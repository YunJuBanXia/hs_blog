use axum::{Extension, Json, extract::Path, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use crate::user::{pwd::Password, serializers::{UserRegisterSerializer, UserResponse, Validate}};

pub async fn list_users(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
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


pub async fn get_user(Path(id): Path<i32>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
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


pub async fn list_users_paged(Path((page, page_size)): Path<(i64, i64)>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
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


pub async fn user_register(Json(serializer): Json<UserRegisterSerializer>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    // 验证输入数据
    match serializer.validate(Extension(pool.clone())).await {
        Ok(true) => {
            // 验证通过, 创建新用户
            let new_user = sqlx::query!(
                "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id",
                serializer.username.to_lowercase(),
                serializer.email.to_lowercase(),
                Password::new(serializer.raw_password).0
            )
            .fetch_one(&pool)
            .await;
            
            match new_user {
                Ok(record) => (StatusCode::CREATED, Json(UserResponse {
                    id: record.id,
                    username: serializer.username.to_lowercase(),
                    email: serializer.email.to_lowercase(),
                    created_at: chrono::Utc::now(),
                })).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user").into_response()
            }
        },
        Ok(false) => (StatusCode::BAD_REQUEST, "Invalid input data").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Validation error").into_response()
    }
}