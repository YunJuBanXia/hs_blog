use axum::response::{IntoResponse, Response};
use serde::{Serialize, Deserialize};
use crate::user::pwd::Password;
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}


impl User {
    pub fn new(id: i32, username: String, email: String, raw_password: String) -> Self {
        let pwd = Password::new(raw_password);
        let password_hash = pwd.0.clone();
        let created_at = Utc::now();
        Self { id, username, email, password_hash, created_at }
    }

    
    pub fn set_password(&mut self, raw_password: String) {
        let pwd = Password::new(raw_password);
        self.password_hash = pwd.0.clone();
    }


    pub fn check_password(&self, raw_pwd: String) -> bool {
        let pwd_obj = Password(self.password_hash.clone());
        pwd_obj.verify(raw_pwd)
    }
}


impl IntoResponse for User {
    fn into_response(self) -> Response {
        // 响应时不包含密码的哈希
        let user_info = serde_json::json!({
            "id": self.id,
            "username": self.username,
            "email": self.email,
            "created_at": self.created_at,
        });
        axum::Json(user_info).into_response()
    }
}