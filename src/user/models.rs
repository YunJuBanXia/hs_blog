use anyhow::Result;
use axum::extract::State;
use regex::Regex;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
use crate::user::pwd::Password;
use chrono::{DateTime, Utc};
use validator::{Validate, ValidationError};
use lazy_static::lazy_static;


lazy_static!(
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[\w.-]+@[\w.-]+\.\w+$").unwrap();
);


#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Validate)]
pub struct User {
    pub id: i32,

    #[validate(custom(function = "User::validate_username"))]
    pub username: String,  // 全小写, 不允许重复

    #[validate(email)]
    pub email: String,  // 全小写, 不允许重复

    pub password_hash: String,  // 存储哈希后的密码
    pub created_at: DateTime<Utc>,
    pub is_active: bool,  // 用户注销不删除数据, 而是将该字段标记为 false
    pub is_admin: bool,   // 是否管理员用户
}


impl User {
    pub fn new(id: i32, username: String, email: String, raw_password: String) -> Self {
        let pwd = Password::new(raw_password);
        let password_hash = pwd.0.clone();
        let created_at = Utc::now();
        Self { id, username, email, password_hash, created_at, is_active: true, is_admin: false }
    }


    pub fn validate_username(username: &String) -> Result<(), ValidationError> {
        if !USERNAME_REGEX.is_match(&username) {
            return Err(ValidationError::new("Username can only contain letters, numbers, and underscores"));
        }
        
        if username.len() < 3 || username.len() > 20 {
            return Err(ValidationError::new("Username must be between 3 and 20 characters long"));
        }

        Ok(())
    }

    
    pub async fn set_password(&mut self, raw_password: String, State(pool): State<PgPool>) -> Result<()> {
        let pwd = Password::new(raw_password);
        self.password_hash = pwd.0.clone();

        let result = sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            self.password_hash,
            self.id
        ).execute(&pool).await;

        if let Err(e) = result {
            return Err(anyhow::anyhow!("Failed to update user password: {}", e));
        }
        Ok(())
    }


    pub fn check_password(&self, raw_pwd: String) -> bool {
        let pwd_obj = Password(self.password_hash.clone());
        pwd_obj.verify(raw_pwd)
    }


    pub async fn from_id(id: i32, State(pool): State<PgPool>) -> Result<Self> {
        let result = sqlx::query_as!(
            User,
            "SELECT id, username, email, password_hash, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\", is_active, is_admin FROM users WHERE id = $1",
            id
        ).fetch_one(&pool).await;
        
        match result {
            Ok(user) => Ok(user),
            Err(_) => {
                Err(anyhow::anyhow!("Failed to fetch user by ID"))
            }
        }
    }


    pub async fn save(&self, State(pool): State<PgPool>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET username = $1, email = $2, password_hash = $3, is_active = $4, is_admin = $5 WHERE id = $6",
            self.username,
            self.email,
            self.password_hash,
            self.is_active,
            self.is_admin,
            self.id
        ).execute(&pool).await?;

        Ok(())
    }
}