use axum::{Router, routing::{get, post}};
use sqlx::{Pool, Postgres};
use crate::user::handlers;

pub fn router() -> Router<Pool<Postgres>> {
    Router::new()
        .route("users", get(handlers::list_users))
        .route("users/{id}", get(handlers::get_user))
        .route("users/register", post(handlers::register))
        .route("users/login", post(handlers::login))
        .route("users/refresh_token", post(handlers::refresh_token))
}