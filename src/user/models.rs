use anyhow::Result;
use regex::Regex;
use serde::{Serialize, Deserialize};
use crate::user::pwd::Password;
use chrono::{DateTime, Utc};
use validator::{Validate, ValidationError};
use lazy_static::lazy_static;


lazy_static!(
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[\w.-]+@[\w.-]+\.\w+$").unwrap();
);


/// User 模型, 对应数据库中的 users 表
/// 其实这只是一个由于与数据库对应的类型, 实际并不使用这个类型进行数据交互
/// 这个类型主要用于提供与用户相关的业务逻辑方法
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Validate)]
pub struct User {
    pub id: i32,
    pub username: String,  // 全小写, 不允许重复
    pub email: String,  // 全小写, 不允许重复
    pub password_hash: String,  // 存储哈希后的密码
    pub created_at: DateTime<Utc>,
    pub is_active: bool,  // 用户注销不删除数据, 而是将该字段标记为 false
    pub is_admin: bool,   // 是否管理员用户
}


impl User {
    pub fn validate_username(username: &String) -> Result<(), ValidationError> {
        if !USERNAME_REGEX.is_match(&username) {
            return Err(ValidationError::new("Username can only contain letters, numbers, and underscores"));
        }
        
        if username.len() < 3 || username.len() > 20 {
            return Err(ValidationError::new("Username must be between 3 and 20 characters long"));
        }

        Ok(())
    }


    pub fn check_password(&self, raw_pwd: String) -> bool {
        let pwd_obj = Password(self.password_hash.clone());
        pwd_obj.verify(raw_pwd)
    }
}