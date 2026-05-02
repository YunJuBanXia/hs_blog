use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;

use crate::captcha::serializers::{CaptchaResponse, VerifyCaptchaRequest};


pub async fn get_captcha(State(pool): State<PgPool>) -> impl IntoResponse {
    // 获取 CaptchaResponse 结构体, 包含 captcha_id 和 captcha_image
    let result = CaptchaResponse::generate_captcha(State(pool)).await;

    match result {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate captcha").into_response()
    }
}


pub async fn verify_captcha(State(pool): State<PgPool>, Json(request): Json<crate::captcha::serializers::VerifyCaptchaRequest>) -> impl IntoResponse {
    let result = VerifyCaptchaRequest::verify(State(pool), Json(request)).await;

    match result {
        Ok(true) => (StatusCode::OK, "Captcha verified").into_response(),
        Ok(false) => (StatusCode::BAD_REQUEST, "Invalid captcha").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to verify captcha").into_response()
    }
}

