use axum::{Extension, Json, extract::Path, response::IntoResponse};
use sqlx::PgPool;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}


pub async fn get_users(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    // 查询所有用户
    // 应尽量不使用这一函数, 而应该使用下面的 get_users_paged 来分页查询用户, 以避免一次性加载过多数据
    let users: Vec<UserResponse> = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, email, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\" FROM users"
    )
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch users");
    
    Json(users)
}


pub async fn get_user_by_id(Path(id): Path<i32>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let user = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, email, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\" FROM users WHERE id = $1",
        id
    )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user by ID");
    
    Json(user)
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