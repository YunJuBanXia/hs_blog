use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use lettre::{AsyncSmtpTransport, Tokio1Executor, transport::smtp::authentication::Credentials};
use rand::random;
use serde::{Deserialize, Serialize};


lazy_static!(
    pub static ref SMTP_TRANSPORT: AsyncSmtpTransport<Tokio1Executor> = {
        let smtp_server: String = dotenvy::var("SMTP_SERVER").unwrap();
        let smtp_port: u16 = dotenvy::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse::<u16>().unwrap();
        let smtp_username: String = dotenvy::var("SMTP_USERNAME").unwrap();
        let smtp_password: String = dotenvy::var("SMTP_PASSWORD").unwrap();

        AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_server)
            .unwrap()
            .port(smtp_port)
            .credentials(Credentials::new(smtp_username, smtp_password))
            .build()
    };

    static ref EMAIL_VERIFICATION_CODE_LENGTH: usize = dotenvy::var("EMAIL_VERIFICATION_CODE_LENGTH").unwrap().parse::<usize>().unwrap();
    static ref CODE_CHARSET: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
);


#[derive(Debug, Clone)]
pub struct EmailVerificationCode {
    pub id: i32,
    pub email: String,
    pub code: String,
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