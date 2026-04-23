use axum::response::{IntoResponse, Response};
use serde::{Serialize, Deserialize};
use crate::user::pwd::Password;
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    id: i32,
    username: String,
    email: String,
    password_hash: String,  // 存储哈希后的密码
    created_at: DateTime<Utc>,
    is_active: bool,  // 用户注销不删除数据, 而是将该字段标记为 false
    is_admin: bool,   // 是否管理员用户
}


impl User {
    pub fn new(id: i32, username: String, email: String, raw_password: String) -> Self {
        let pwd = Password::new(raw_password);
        let password_hash = pwd.0.clone();
        let created_at = Utc::now();
        Self { id, username, email, password_hash, created_at, is_active: true, is_admin: false }
    }

    
    pub fn set_password(&mut self, raw_password: String) {
        let pwd = Password::new(raw_password);
        self.password_hash = pwd.0.clone();
    }


    pub fn check_password(&self, raw_pwd: String) -> bool {
        let pwd_obj = Password(self.password_hash.clone());
        pwd_obj.verify(raw_pwd)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}