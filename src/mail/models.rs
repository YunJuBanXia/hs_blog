use chrono::{DateTime, Utc};


#[derive(Debug, Clone)]
pub struct EmailVerificationCode {
    pub id: i32,
    pub email: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}