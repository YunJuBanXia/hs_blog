use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, encode};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};


lazy_static!(
    static ref ENCODING_KEY: EncodingKey = {
        let kp = std::fs::read_to_string("private_key_pkcs8.pem").expect("Failed to read private key");
        EncodingKey::from_rsa_pem(kp.as_bytes()).expect("Failed to create encoding key from PEM")
    };

    static ref DECODING_KEY: DecodingKey = {
        let kp = std::fs::read_to_string("public_key.pem").expect("Failed to read public key");
        DecodingKey::from_rsa_pem(kp.as_bytes()).expect("Failed to create decoding key from PEM")
    };
);


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