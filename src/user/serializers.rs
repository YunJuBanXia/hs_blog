use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use validator::Validate;

use crate::{user::{models::User, pwd::Password}};


#[derive(Debug, Serialize, Validate)]
pub struct UserResponse {
    // 使用时直接使用 Json(response) 包装即可, 不需要手动构造 Response 对象
    pub id: i32,

    #[validate(custom(function = "User::validate_username"))]
    pub username: String,

    #[validate(email)]
    pub email: String,
    pub created_at: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserRegisterSerializer {
    #[validate(custom(function = "User::validate_username"))]
    pub username: String,

    #[validate(email)]
    pub email: String,

    #[validate(custom(function = "Password::validate_raw_password"))]
    pub raw_password: String,

    pub email_verification_code: String,  // 用户注册时需要提供邮箱验证码, 以验证邮箱的有效性
}


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserLoginSerializer {
    #[validate(custom(function = "User::validate_certificate"))]
    pub certificate: String,  // 用户名或邮箱
    
    #[validate(custom(function = "Password::validate_raw_password"))]
    pub raw_password: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}