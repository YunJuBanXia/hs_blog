use anyhow::Result;
use async_trait::async_trait;
use axum::Extension;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};


/// 这个 trait 定义了一个 validate 方法, 用于验证请求数据的合法性,
/// 实现这个 trait 的结构体需要提供 validate 方法的具体实现, 来检查字段是否满足特定的条件, 例如用户名和邮箱的格式, 密码的强度等,
/// Ok(true) 表示验证通过, Ok(false) 表示验证失败
/// Err(e) 表示发生内部错误
#[async_trait]
pub trait Validate {
    async fn validate(&self, pool: Extension<sqlx::PgPool>) -> Result<bool, anyhow::Error>;
}


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


#[async_trait]
impl Validate for UserRegisterSerializer {
    async fn validate(&self, pool: Extension<sqlx::PgPool>) -> Result<bool, anyhow::Error> {
        let Extension(pool) = pool;

        // 验证用户名: 将输入转为小写, 长度范围 3-50, 只能包含字母, 数字, 下划线, 以及是否与已有用户冲突
        let username = self.username.to_lowercase();
        if username.len() < 3 || username.len() > 50 {
            return Ok(false);
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Ok(false);
        }
        // 数据库查询, 检查用户名是否已存在
        let is_user_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)",
            username
        ).fetch_one(&pool).await?.unwrap_or(false);
        if !is_user_exists {
            return Ok(false);
        }


        // 验证邮箱: 将输入转为小写, 使用正则表达式检查格式, 以及是否与已有用户冲突
        let email = self.email.to_lowercase();
        let email_regex = regex::Regex::new(r"^[\w.-]+@[\w.-]+\.\w+$")?;
        if !email_regex.is_match(&email) {
            return Ok(false);
        }
        // 数据库查询, 检查邮箱是否已存在
        let is_email_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
            email
        ).fetch_one(&pool).await?.unwrap_or(false);

        if !is_email_exists {
            return Ok(false);
        }


        // 验证密码: 长度至少 8, 包含大写字母, 小写字母, 数字, 和特殊字符 其中两者或以上
        let raw_pwd = &self.raw_password;
        if raw_pwd.len() < 8 {
            return Ok(false);
        }
        let has_upper = raw_pwd.chars().any(|c| c.is_uppercase());
        let has_lower = raw_pwd.chars().any(|c| c.is_lowercase());
        let has_digit = raw_pwd.chars().any(|c| c.is_ascii_digit());
        let has_special = raw_pwd.chars().any(|c| "!@#$%^&*()-+_".contains(c));
        let valid_password = [has_upper, has_lower, has_digit, has_special].iter().filter(|&&x| x).count() >= 2;
        if !valid_password {
            return Ok(false);
        }
        
        // 所有验证通过
        Ok(true)
    }
}