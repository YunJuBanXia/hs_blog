use axum::{Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use validator::Validate;

use crate::{db::handle_db_error, errors::AppError, user::{jwt::{self, decode_token}, pwd::Password, serializers::{RefreshTokenResponse, RefreshTokenSerializer, UserLoginResponse, UserLoginSerializer, UserRegisterSerializer, UserResponse}}};


pub async fn list_users(State(pool): State<PgPool>) -> impl IntoResponse {
    // 查询所有用户
    // 应尽量不使用这一函数, 而应该使用下面的 get_users_paged 来分页查询用户, 以避免一次性加载过多数据
    let result: Result<Vec<UserResponse>, sqlx::Error> = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, email, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\" FROM users"
    )
        .fetch_all(&pool)
        .await;
    
    match result {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch users").into_response()
    }
}


pub async fn get_user(
    Path(id): Path<i32>,
    State(pool): State<PgPool>
) -> impl IntoResponse {
    let result = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, email, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\" FROM users WHERE id = $1",
        id
    )
        .fetch_optional(&pool)
        .await;
    
    match result {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user").into_response()
    }
}


pub async fn list_users_paged(
    Path((page, page_size)): Path<(i64, i64)>,
    State(pool): State<PgPool>
) -> impl IntoResponse {
    let offset = (page - 1) * page_size;
    let result: Result<Vec<UserResponse>, sqlx::Error> = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, email, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\" FROM users ORDER BY id LIMIT $1 OFFSET $2",
        page_size,
        offset
    )
        .fetch_all(&pool)
        .await;
    
    match result {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch users").into_response()
    }
}


/// 在调用该handler前, 应该先通过 crate::mail::handlers::send_verification_email 发送邮箱验证码, 并将验证码存储在数据库中, 用户注册时需要提供正确的验证码才能完成注册.
#[axum::debug_handler]
pub async fn register(
    State(pool): State<PgPool>,
    Json(serializer): Json<UserRegisterSerializer>
) -> Result<impl IntoResponse, AppError> {
    // 校验邮箱验证码
    let is_email_exist = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
        serializer.email
    )
        .fetch_one(&pool)
        .await?;

    // 数据库查询失败
    if is_email_exist.is_none() {
        return Err(AppError::Database(sqlx::Error::RowNotFound));
    }
    // 邮箱已存在
    if let Some(true) = is_email_exist {
        return Err(AppError::Conflict("email".to_string()));
    }

    // 数据库查询成功, 但邮箱不存在, 可以继续注册
    // 验证邮箱验证码
    let record = sqlx::query!(
        "SELECT code, expires_at FROM email_verification_codes WHERE email = $1",
        serializer.email
    )
        .fetch_optional(&pool)
        .await?;

    match record {
        Some(record) => {
            if record.expires_at < Utc::now() {
                return Err(AppError::Expired("email_verification_code".to_string()));
            }
            if record.code != serializer.email_verification_code {
                return Err(AppError::Invalid("email_verification_code".to_string()));
            }
        }
        None => {
            return Err(AppError::NotFound("email_verification_code".to_string()));
        }
    }

    // 验证输入数据合法性
    if let Err(errors) = serializer.validate() {
        return Err(AppError::Validation(errors));
    }

    // 对密码进行哈希
    let pwd_hash = Password::new(serializer.raw_password).0;
    
    // 执行插入
    sqlx::query!(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)",
        serializer.username,
        serializer.email,
        pwd_hash
    )
        .execute(&pool)
        .await
        .map_err(|e| handle_db_error(e))?;  // 拦截错误并转换为 AppError

    // 注册成功
    Ok(StatusCode::CREATED)
}


#[axum::debug_handler]
pub async fn login(
    State(pool): State<PgPool>, 
    Json(serializer): Json<UserLoginSerializer>
) -> Result<impl IntoResponse, AppError> {
    // 基础格式校验
    serializer.validate().map_err(|e| AppError::Validation(e))?;

    // 数据库检索
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = $1 OR email = $1",
        serializer.certificate
    )
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::InvalidCredentials("login_certificate".to_string()))?;

    // 验证密码
    let is_password_valid = Password::verify(serializer.raw_password, user.password_hash);
    if !is_password_valid {
        return Err(AppError::Invalid("password".to_string()));
    }

    // 生成 JWT token
    let issued_at = Utc::now();
    let access_duration = Duration::hours(*jwt::JWT_ACCESS_TOKEN_EXPIRATION_HOURS);
    let refresh_duration = Duration::hours(*jwt::JWT_REFRESH_TOKEN_EXPIRATION_HOURS);

    let access_token = jwt::generate_token(user.id, issued_at + access_duration, issued_at).map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to generate JWT token: {}", e)))?;
    let refresh_token = jwt::generate_token(user.id, issued_at + refresh_duration, issued_at).map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to generate JWT token: {}", e)))?;

    // 构造响应
    let resp = UserLoginResponse {
        access_token,
        refresh_token,
    };

    Ok((StatusCode::OK, Json(resp)))
}


#[axum::debug_handler]
pub async fn refresh_token(
    Json(serializer): Json<RefreshTokenSerializer>
) -> Result<impl IntoResponse, AppError> {
    // 解密并校验 refresh token
    let token_data = decode_token(&serializer.refresh_token).map_err(|_| AppError::InvalidToken)?;

    // 验证通过, 签发新的 Access Token
    let user_id = token_data.sub;

    let now = Utc::now();
    let access_duration = Duration::hours(*jwt::JWT_ACCESS_TOKEN_EXPIRATION_HOURS);
    
    let new_token = jwt::generate_token(user_id, now + access_duration, now)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to generate new JWT token: {}", e)))?;

    Ok((StatusCode::OK, Json(RefreshTokenResponse { access_token: new_token })))
}