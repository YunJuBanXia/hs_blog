use axum::{Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use crate::user::{pwd::Password, serializers::{UserRegisterError, UserRegisterSerializer, UserResponse}};


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


pub async fn user_register(Json(serializer): Json<UserRegisterSerializer>, State(pool): State<PgPool>) -> impl IntoResponse {
    // 在调用该handler前, 应该先通过 crate::captcha::handlers::verify_captcha 来验证用户提交的验证码, 以防止恶意注册
    // 验证输入数据
    match serializer.validate(State(pool.clone())).await {
        Ok(_) => {
            // 验证通过, 创建新用户
            match sqlx::query!(
                "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)",
                serializer.username.to_lowercase(),
                serializer.email.to_lowercase(),
                Password::new(serializer.raw_password).0
            ).execute(&pool).await {
                Ok(_) => (StatusCode::CREATED, "User registered successfully").into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user").into_response()
            }
        }
        Err(e) => {
            // 验证失败, 返回相应的错误信息
            match e {
                UserRegisterError::InvalidUsername => (StatusCode::BAD_REQUEST, "Invalid username").into_response(),
                UserRegisterError::InvalidEmail => (StatusCode::BAD_REQUEST, "Invalid email").into_response(),
                UserRegisterError::InvalidPassword => (StatusCode::BAD_REQUEST, "Invalid password").into_response(),
                UserRegisterError::UsernameAlreadyExists => (StatusCode::BAD_REQUEST, "Username already exists").into_response(),
                UserRegisterError::EmailAlreadyExists => (StatusCode::BAD_REQUEST, "Email already exists").into_response(),
                UserRegisterError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
            }
        }
    }
}


pub async fn user_login() -> impl IntoResponse {
    // TODO: 实现用户登录功能, 包括验证用户名/邮箱和密码, 以及生成 JWT token 等
    (StatusCode::NOT_IMPLEMENTED, "Login functionality not implemented yet").into_response()
}