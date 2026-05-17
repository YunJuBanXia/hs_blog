use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{Duration, Utc};
use lazy_static::lazy_static;
use lettre::{AsyncTransport, Message};
use sqlx::PgPool;
use validator::Validate;

use crate::{errors::AppError, mail::{models::{EmailVerificationCode, SMTP_TRANSPORT}, serializers::SendVerificationEmailSerializer}};


lazy_static!(
    static ref EMAIL_VERIFICATION_CODE_LENGTH: usize = dotenvy::var("EMAIL_VERIFICATION_CODE_LENGTH").unwrap().parse::<usize>().unwrap();
    static ref EMAIL_VERIFICATION_CODE_EXPIRATION_MINUTES: i64 = dotenvy::var("EMAIL_VERIFICATION_CODE_EXPIRATION_MINUTES").unwrap().parse::<i64>().unwrap();
);


pub async fn send_verification_email(
    State(pool): State<PgPool>,
    Json(serializer): Json<SendVerificationEmailSerializer>
) -> Result<impl IntoResponse, AppError> {
    // 验证输入数据合法性(邮箱合法性)
    serializer.validate().map_err(|e| AppError::Validation(e))?;

    // TODO: 限制发送频率
    
    let new_code = EmailVerificationCode::generate_code();
    let expires_at = Utc::now() + Duration::minutes(*EMAIL_VERIFICATION_CODE_EXPIRATION_MINUTES);

    // 将验证码存入数据库
    sqlx::query!(
        r#"
        INSERT INTO email_verification_codes (email, code ,expires_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (email)
        DO UPDATE SET
            code = EXCLUDED.code,
            expires_at = EXCLUDED.expires_at,
            updated_at = CURRENT_TIMESTAMP
        "#,
        serializer.email,
        new_code,
        expires_at
    )
        .execute(&pool)
        .await?;

    // 实际的邮件发送功能
    let email = Message::builder()
        .from("noreply@ban-xia.com".parse().unwrap())
        .to(serializer.email.parse().unwrap())
        .subject("Email Verification")
        .body(format!("Your verification code is: {}", new_code))
        .unwrap();

    SMTP_TRANSPORT.send(email).await?;

    Ok((StatusCode::OK, "Verification email sent successfully").into_response())
}