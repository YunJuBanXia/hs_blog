use axum::{Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use validator::Validate;

use crate::{db::handle_db_error, errors::AppError, user::{pwd::Password, serializers::{UserRegisterSerializer, UserResponse}}};


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


pub async fn get_user(Path(id): Path<i32>, State(pool): State<PgPool>) -> impl IntoResponse {
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


pub async fn list_users_paged(Path((page, page_size)): Path<(i64, i64)>, State(pool): State<PgPool>) -> impl IntoResponse {
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


/// 在调用该handler前, 应该先通过 crate::captcha::handlers::verify_captcha 来验证用户提交的验证码, 以防止恶意注册
pub async fn user_register(Json(serializer): Json<UserRegisterSerializer>, State(pool): State<PgPool>) -> Result<impl IntoResponse, AppError> {
    // TODO: 校验邮箱验证码
    let is_email_exist = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
        serializer.email
    )
        .fetch_one(&pool)
        .await?;

    // 数据库查询失败
    if is_email_exist.is_none() {
        return Err(AppError::DatabaseError(sqlx::Error::RowNotFound));
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
            if record.expires_at < chrono::Utc::now() {
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
        return Err(AppError::ValidationError(errors));
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
    Ok((StatusCode::CREATED, "User registered successfully").into_response())
}


pub async fn user_login() -> impl IntoResponse {
    // TODO: 实现用户登录功能, 包括验证用户名/邮箱和密码, 以及生成 JWT token 等
    (StatusCode::NOT_IMPLEMENTED, "Login functionality not implemented yet").into_response()
}