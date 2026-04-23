use axum::{Extension, Json, extract::Path, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use crate::user::models::{UserResponse};

pub async fn get_users(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
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


pub async fn get_user_by_id(Path(id): Path<i32>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
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


pub async fn get_users_paged(Path((page, page_size)): Path<(i64, i64)>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let offset = (page - 1) * page_size;
    let users: Vec<UserResponse> = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, email, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\" FROM users ORDER BY id LIMIT $1 OFFSET $2",
        page_size,
        offset
    )
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch paged users");
    
    Json(users)
}