use axum::{Router, routing::get};
use sqlx::{Pool, Postgres};
use crate::user::handlers;

pub fn router() -> Router<Pool<Postgres>> {
    Router::new()
        .route("users", get(handlers::list_users))
        .route("users/{id}", get(handlers::get_user))
}