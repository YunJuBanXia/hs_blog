use std::sync::LazyLock;

use axum::{extract::FromRequestParts, http::request::Parts};
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, decode, encode};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;


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


pub fn decode_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token, 
        &DECODING_KEY, 
        &jsonwebtoken::Validation::new(Algorithm::RS256)
    )?;

    Ok(token_data.claims)
}


#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i32,
}


impl<T> FromRequestParts<T> for AuthenticatedUser
where
    T: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts<'a, 'b>(
        parts: &'a mut Parts,
        _state: &'b T
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::InvalidCredentials("Missing authorization header".to_string()))?;

        if !header.starts_with("Bearer ") {
            return Err(AppError::InvalidToken);
        }

        // 提取 token 并验证
        let token = &header[7..];  // 去掉 "Bearer " 前缀
        let token_data = decode_token(token)
            .map_err(|_| AppError::InvalidToken)?;

        Ok(AuthenticatedUser {
            user_id: token_data.sub,
        })
    }
}




