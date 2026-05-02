use axum::extract::State;
use captcha_rs::CaptchaBuilder;
use chrono::Utc;
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use uuid::Uuid;

use crate::captcha::serializers::CaptchaResponse;


pub async fn generate_captcha(State(pool): State<PgPool>) -> Result<CaptchaResponse, anyhow::Error> {
    // 生成验证码图片和答案
    let captcha = CaptchaBuilder::new()
        .length(5)
        .width(220)
        .height(100)
        .dark_mode(false)
        .complexity(5)
        .compression(40)
        .distortion(3)
        .build();

    let answer = &captcha.text;
    
    // 将图片数据转换为 base64 编码字符串
    let image_base64 = captcha.to_base64();

    // 生成唯一验证码 ID
    let captcha_id = Uuid::new_v4().to_string();

    // 对 answer 进行哈希
    let mut hasher = Sha256::new();
    hasher.update(answer.as_bytes());
    let hashed_answer: String = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();

    // 将 captcha_id 和 hashed_answer 存进数据库
    let captcha_expire_seconds = dotenvy::var("CAPTCHA_EXPIRATION_SECONDS").unwrap_or_else(|_| "300".to_string()).parse::<i64>().unwrap_or(300);
    let expires_at = Utc::now() + chrono::Duration::seconds(captcha_expire_seconds);

    sqlx::query!(
        "INSERT INTO image_captchas (id, answer_hash, expires_at) VALUES ($1, $2, $3)",
        captcha_id,
        hashed_answer,
        expires_at
    )
    .fetch_one(&pool)
    .await?;

    Ok(CaptchaResponse {
        captcha_id: captcha_id,
        image_base64,
    })
}


/// 验证用户提交的验证码答案是否正确.
/// Ok(true) 表示验证通过;
/// Ok(false) 表示验证失败, 包括验证码过期或答案错误;
/// Err(e) 表示发生内部错误, 例如数据库查询失败;
pub async fn verify_captcha(State(pool): State<PgPool>, captcha_id: String, answer: String) -> Result<bool, anyhow::Error> {
    // 从数据库中查询对应的 hashed_answer 和 expires_at
    let record = sqlx::query!(
        "SELECT answer_hash, expires_at FROM image_captchas WHERE id = $1",
        captcha_id
    )
    .fetch_optional(&pool)
    .await?;

    if let Some(record) = record {
        // 检查验证码是否过期
        if Utc::now() > record.expires_at {
            return Ok(false);
        }

        // 对用户输入的 answer 进行哈希
        let mut hasher = Sha256::new();
        hasher.update(answer.as_bytes());
        let hashed_input: String = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();

        // 比较哈希值
        Ok(hashed_input == record.answer_hash)
    } else {
        // 没有找到对应的验证码记录
        Err(anyhow::anyhow!("Captcha ID {} does not exist", captcha_id))
    }
}