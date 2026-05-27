use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use lettre::{AsyncSmtpTransport, Tokio1Executor, transport::smtp::authentication::Credentials};
use rand::random;
use serde::{Deserialize, Serialize};


pub static SMTP_TRANSPORT: LazyLock<AsyncSmtpTransport<Tokio1Executor>> = LazyLock::new(|| {
    let smtp_server: String = dotenvy::var("SMTP_SERVER").unwrap();
    let smtp_port: u16 = dotenvy::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse::<u16>().unwrap();
    let smtp_username: String = dotenvy::var("SMTP_USERNAME").unwrap();
    let smtp_password: String = dotenvy::var("SMTP_PASSWORD").unwrap();

    AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_server)
        .unwrap()
        .port(smtp_port)
        .credentials(Credentials::new(smtp_username, smtp_password))
        .build()
});
pub static EMAIL_VERIFICATION_CODE_LENGTH: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("EMAIL_VERIFICATION_CODE_LENGTH").unwrap().parse::<usize>().unwrap()
});
pub static CODE_CHARSET: LazyLock<Vec<char>> = LazyLock::new(|| {
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect()
});


#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct EmailVerificationCode {
    pub id: i32,
    pub email: String,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


impl EmailVerificationCode {
    pub fn generate_code() -> String {
        let code: String = (0..*EMAIL_VERIFICATION_CODE_LENGTH)
            .map(|_| {
                let n = random::<u8>() % 36;
                CODE_CHARSET[n as usize]
            })
            .collect();
        code.to_uppercase()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code() {
        let code = EmailVerificationCode::generate_code();
        println!("Generated code: {}", code);
        assert_eq!(code.len(), *EMAIL_VERIFICATION_CODE_LENGTH);
        assert!(code.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }
}