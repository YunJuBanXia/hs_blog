use axum::{Extension, Json, extract::Path, response::IntoResponse};
use sqlx::PgPool;
use crate::user::models::User;


pub async fn get_users(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    // 查询所有用户
    // 应尽量不使用这一函数, 而应该使用下面的 get_users_paged 来分页查询用户, 以避免一次性加载过多数据
    // 且这一函数会暴露密码哈希
    let users: Vec<User> = sqlx::query_as!(
        User,
        "SELECT id, username, email, password_hash, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\" FROM users"
    )
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch users");
    
    Json(users)
}

pub async fn get_user_by_id(Path(id): Path<i32>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, email, password_hash, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\" FROM users WHERE id = $1",
        id
    )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user by ID");
    
    user
}