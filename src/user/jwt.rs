use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};


pub static ENCODING_KEY: LazyLock<EncodingKey> = LazyLock::new(|| {
    let kp = std::fs::read_to_string("private_key_pkcs8.pem").expect("Failed to read private key");
    EncodingKey::from_rsa_pem(kp.as_bytes()).expect("Failed to create encoding key from PEM")
});
pub static DECODING_KEY: LazyLock<DecodingKey> = LazyLock::new(|| {
    let kp = std::fs::read_to_string("public_key.pem").expect("Failed to read public key");
    DecodingKey::from_rsa_pem(kp.as_bytes()).expect("Failed to create decoding key from PEM")
});
pub static JWT_ACCESS_TOKEN_EXPIRATION_HOURS: LazyLock<i64> = LazyLock::new(|| {
    dotenvy::var("JWT_ACCESS_TOKEN_EXPIRATION_HOURS").unwrap_or_else(|_| "2".to_string()).parse::<i64>().unwrap_or(2)
});
pub static JWT_REFRESH_TOKEN_EXPIRATION_HOURS: LazyLock<i64> = LazyLock::new(|| {
    dotenvy::var("JWT_REFRESH_TOKEN_EXPIRATION_HOURS").unwrap_or_else(|_| "24".to_string()).parse::<i64>().unwrap_or(24)
});


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,   // 用户 ID, 数据库主键
    pub exp: usize,   // 过期时间戳
    pub iat: usize,   // 签发时间戳
}


pub fn generate_token(user_id: i32, expires_at: DateTime<Utc>, issued_at: DateTime<Utc>) -> Result<String, jsonwebtoken::errors::Error> {
    let header = Header::new(Algorithm::RS256);
    let claims = Claims {
        sub: user_id,
        exp: expires_at.timestamp() as usize,
        iat: issued_at.timestamp() as usize,
    };

    encode(&header, &claims, &ENCODING_KEY)
}