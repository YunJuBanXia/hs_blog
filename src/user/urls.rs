use axum::{Router, routing::get};
use crate::user::handlers;

pub fn router() -> Router {
    Router::new()
        .route("users", get(handlers::list_users))
        .route("users/{id}", get(handlers::get_user))
}