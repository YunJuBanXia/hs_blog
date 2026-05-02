use anyhow::Result;
use axum::extract::State;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
use crate::user::pwd::Password;
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    id: i32,
    username: String,  // 全小写, 不允许重复
    email: String,  // 全小写, 不允许重复
    password_hash: String,  // 存储哈希后的密码
    created_at: DateTime<Utc>,
    is_active: bool,  // 用户注销不删除数据, 而是将该字段标记为 false
    is_admin: bool,   // 是否管理员用户
    is_email_varified: bool,  // 邮箱是否已验证
    email_varificated_at: Option<DateTime<Utc>>,  // 邮箱验证时间
}


impl User {
    pub fn new(id: i32, username: String, email: String, raw_password: String) -> Self {
        let pwd = Password::new(raw_password);
        let password_hash = pwd.0.clone();
        let created_at = Utc::now();
        Self { id, username, email, password_hash, created_at, is_active: true, is_admin: false, is_email_varified: false, email_varificated_at: None }
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
            "SELECT id, username, email, password_hash, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\", is_active, is_admin, is_email_varified, email_varificated_at as \"email_varificated_at!: chrono::DateTime<chrono::Utc>\" FROM users WHERE id = $1",
            id
        ).fetch_one(&pool).await;
        
        match result {
            Ok(user) => Ok(user),
            Err(_) => {
                Err(anyhow::anyhow!("Failed to fetch user by ID"))
            }
        }
    }
}


