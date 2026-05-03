use anyhow::Result;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{captcha::serializers::{CaptchaError, VerifyCaptchaSerializer}, user::models::User};


#[derive(Debug, Serialize)]
pub struct UserResponse {
    // 使用时直接使用 Json(response) 包装即可, 不需要手动构造 Response 对象
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegisterSerializer {
    pub username: String,
    pub email: String,
    pub raw_password: String,
}


pub enum UserRegisterError {
    InvalidUsername,
    InvalidEmail,
    InvalidPassword,
    UsernameAlreadyExists,
    EmailAlreadyExists,
    DatabaseError(sqlx::Error),
}


impl UserRegisterSerializer {
    pub async fn validate(&self, State(pool): State<PgPool>) -> Result<(), UserRegisterError> {
        // 验证用户名: 将输入转为小写, 长度范围 3-50, 只能包含字母, 数字, 下划线, 以及是否与已有用户冲突
        let username = self.username.to_lowercase();
        if username.len() < 3 || username.len() > 50 {
            return Err(UserRegisterError::InvalidUsername);
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(UserRegisterError::InvalidUsername);
        }
        // 数据库查询, 检查用户名是否已存在
        let is_user_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)",
            username
        ).fetch_one(&pool).await;
        if let Err(e) = is_user_exists {
            return Err(UserRegisterError::DatabaseError(e));
        }
        // 排除查询错误, 直接unwrap
        // 若结果为 None, 则表示查询成功但没有匹配的用户, 即用户名可用;
        // 若结果为 Some(true), 则表示用户名已存在;
        // 若结果为 Some(false), 则表示用户名不存在, 但查询成功
        let is_user_exists = is_user_exists.unwrap().unwrap_or(false);

        if is_user_exists {
            return Err(UserRegisterError::UsernameAlreadyExists);
        }


        // 验证邮箱: 将输入转为小写, 使用正则表达式检查格式, 以及是否与已有用户冲突
        let email = self.email.to_lowercase();
        let email_regex = regex::Regex::new(r"^[\w.-]+@[\w.-]+\.\w+$").unwrap();
        if !email_regex.is_match(&email) {
            return Err(UserRegisterError::InvalidEmail);
        }
        // 数据库查询, 检查邮箱是否已存在
        let is_email_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
            email
        ).fetch_one(&pool).await;

        if let Err(e) = is_email_exists {
            return Err(UserRegisterError::DatabaseError(e));
        }

        let is_email_exists = is_email_exists.unwrap().unwrap_or(false);

        if is_email_exists {
            return Err(UserRegisterError::EmailAlreadyExists);
        }


        // 验证密码: 长度至少 8, 包含大写字母, 小写字母, 数字, 和下划线 其中两者或以上
        let raw_pwd = &self.raw_password;
        if raw_pwd.len() < 8 {
            return Err(UserRegisterError::InvalidPassword);
        }
        let has_upper = raw_pwd.chars().any(|c| c.is_uppercase());
        let has_lower = raw_pwd.chars().any(|c| c.is_lowercase());
        let has_digit = raw_pwd.chars().any(|c| c.is_ascii_digit());
        let has_special = raw_pwd.chars().any(|c| "_".contains(c));
        let is_valid_password = [has_upper, has_lower, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count() >= 2;
        if !is_valid_password {
            return Err(UserRegisterError::InvalidPassword);
        }
        
        // 所有验证通过
        Ok(())
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginSerializer {
    pub certificate: String,  // 用户名或邮箱
    pub raw_password: String,
    pub captcha_id: String,
    pub answer: String,
}


pub enum UserLoginError {
    InvalidCertificate,
    WrongPassword,
    UserBanned,
    CaptchaVerificationFailed(CaptchaError),
    DatabaseError(sqlx::Error),
}


impl UserLoginSerializer {
    pub async fn validate(&self, State(pool): State<PgPool>) -> Result<(), UserLoginError> {
        // 验证码校验
        let serializer = VerifyCaptchaSerializer {
            captcha_id: self.captcha_id.to_owned(),
            answer: self.answer.to_owned(),
        };
        if let Err(e) = VerifyCaptchaSerializer::verify(State(pool.clone()), Json(serializer)).await {
            return Err(UserLoginError::CaptchaVerificationFailed(e));
        }

        // 验证用户名/邮箱
        let certificate = self.certificate.to_lowercase();
        if !(certificate.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '@' || c == '.')) {
            // 输入的用户名/邮箱格式不合法, 直接返回错误, 不进行数据库查询
            return Err(UserLoginError::InvalidCertificate);
        }
        let record = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE username = $1 OR email = $1",
            certificate
        )
        .fetch_optional(&pool)
        .await;
        
        if let Err(e) = record {
            return Err(UserLoginError::DatabaseError(e));
        }
        
        match record.unwrap() {
            Some(user) => {
                if !user.check_password(self.raw_password.to_owned()) {
                    Err(UserLoginError::WrongPassword)
                } else if !user.is_active {
                    Err(UserLoginError::UserBanned)
                } else {
                    Ok(())
                }
            }
            None => Err(UserLoginError::InvalidCertificate),
        }
    }
}