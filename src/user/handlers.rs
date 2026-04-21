use axum::{Extension, extract::Path, response::{IntoResponse, Response}};
use sqlx::PgPool;
use crate::user::models::User;

pub async fn get_users() -> String {
    todo!()
}

pub async fn get_user_by_id(Path(id): Path<i32>, Extension(pool): Extension<PgPool>) -> Response {
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, email, password_hash, created_at as \"created_at!: chrono::DateTime<chrono::Utc>\" FROM users WHERE id = $1",
        id
    )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user by ID");
    
    user.into_response()
}