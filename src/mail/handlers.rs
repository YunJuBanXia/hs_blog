use std::sync::LazyLock;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{DateTime, Duration, Utc};
use lettre::{AsyncTransport, Message};
use sqlx::PgPool;
use validator::Validate;

use crate::{errors::AppError, mail::{models::{EmailVerificationCode, SMTP_TRANSPORT}, serializers::SendVerificationEmailSerializer}};


pub static EMAIL_VERIFICATION_CODE_LENGTH: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("EMAIL_VERIFICATION_CODE_LENGTH").unwrap().parse::<usize>().unwrap()
});
pub static EMAIL_VERIFICATION_CODE_EXPIRATION_MINUTES: LazyLock<i64> = LazyLock::new(|| {
    dotenvy::var("EMAIL_VERIFICATION_CODE_EXPIRATION_MINUTES").unwrap().parse::<i64>().unwrap()
});
pub static EMAIL_VERIFICATION_CODE_RATE_LIMIT_MINUTES: LazyLock<i64> = LazyLock::new(|| {
    dotenvy::var("EMAIL_VERIFICATION_CODE_RATE_LIMIT_MINUTES").unwrap().parse::<i64>().unwrap()
});


pub async fn send_verification_email(
    State(pool): State<PgPool>,
    Json(serializer): Json<SendVerificationEmailSerializer>
) -> Result<impl IntoResponse, AppError> {
    // 验证输入数据合法性(邮箱合法性)
    serializer.validate().map_err(|e| AppError::Validation(e))?;

    // 限制发送频率
    // 60 秒内同一邮箱只能发送一次验证码
    let last_updated: Option<DateTime<Utc>> = sqlx::query_scalar!(
        "SELECT updated_at FROM email_verification_codes WHERE email = $1",
        serializer.email
    )
        .fetch_optional(&pool)
        .await?;

    if let Some(last_time) = last_updated {
        let duration = *EMAIL_VERIFICATION_CODE_RATE_LIMIT_MINUTES;
        if Utc::now() < last_time + Duration::minutes(duration) {
            let retry_after = duration - Utc::now().signed_duration_since(last_time).num_minutes();
            return Err(AppError::TooManyRequests(format!("Please wait {} minutes before requesting another code", retry_after)));
        }
    }

    let new_code = EmailVerificationCode::generate_code();
    let expires_at = Utc::now() + Duration::minutes(*EMAIL_VERIFICATION_CODE_EXPIRATION_MINUTES);

    // 将验证码存入数据库
    sqlx::query!(
        r#"
        INSERT INTO email_verification_codes (email, code, expires_at)
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