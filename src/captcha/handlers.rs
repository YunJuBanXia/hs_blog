use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;

use crate::captcha::serializers::{CaptchaError, CaptchaResponse, VerifyCaptchaSerializer};


pub async fn get_captcha(State(pool): State<PgPool>) -> impl IntoResponse {
    // 获取 CaptchaResponse 结构体, 包含 captcha_id 和 captcha_image
    let result = CaptchaResponse::generate_captcha(State(pool)).await;

    match result {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate captcha").into_response()
    }
}


pub async fn verify_captcha(State(pool): State<PgPool>, Json(serializer): Json<VerifyCaptchaSerializer>) -> impl IntoResponse {
    let result = VerifyCaptchaSerializer::verify(State(pool), Json(serializer)).await;

    match result {
        Ok(_) => (StatusCode::OK, "Captcha verified").into_response(),
        Err(e) => match e {
            CaptchaError::NotFound => (StatusCode::NOT_FOUND, "Captcha not found").into_response(),
            CaptchaError::Expired => (StatusCode::BAD_REQUEST, "Captcha expired").into_response(),
            CaptchaError::AlreadyUsed => (StatusCode::BAD_REQUEST, "Captcha already used").into_response(),
            CaptchaError::WrongAnswer => (StatusCode::BAD_REQUEST, "Wrong captcha answer").into_response(),
            CaptchaError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
        }   
    }
}

