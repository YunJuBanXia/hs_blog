use axum::{Router, routing::{get, post}};
use sqlx::{Pool, Postgres};

use crate::captcha::handlers::{get_captcha, verify_captcha};


pub fn router() -> Router<Pool<Postgres>> {
    Router::new()
        .route("captchas", get(get_captcha))
        .route("captchas/verify", post(verify_captcha))
}


