use serde::{Serialize, Deserialize};
use crate::user::pwd::Password;
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: u32,
    name: String,
    email: String,
    password: Password,
    created_at: DateTime<Utc>,
}


impl User {
    pub fn new(id: u32, name: String, email: String, raw_password: String) -> Self {
        let password = Password::new(raw_password);
        let created_at = Utc::now();
        Self { id, name, email, password, created_at }
    }

    
    pub fn set_password(&mut self, raw_password: String) {
        self.password = Password::new(raw_password);
    }


    pub fn check_password(&self, raw_pwd: String) -> bool {
        self.password.verify(raw_pwd)
    }
}