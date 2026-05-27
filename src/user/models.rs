use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;
use chrono::{DateTime, Utc};
use validator::{ValidateEmail, ValidationError};


pub static USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9_]+$").unwrap()
});
pub static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\w.-]+@[\w.-]+\.\w+$").unwrap()
});


/// User 模型, 对应数据库中的 users 表
/// 其实这只是一个由于与数据库对应的类型, 实际并不使用这个类型进行数据交互
/// 这个类型主要用于提供与用户相关的业务逻辑方法
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


    pub fn validate_certificate(certificate: &String) -> Result<(), ValidationError> {
        if certificate.contains('@') {
            // 可能是邮箱, 调用 validator::ValidateEmail::validate_email 来验证邮箱格式
            if !ValidateEmail::validate_email(certificate) {
                return Err(ValidationError::new("Invalid email format"));
            }
        } else {
            // 可能是用户名, 调用 validate_username 方法
            Self::validate_username(certificate)?;
        }
        Ok(())
    }
}