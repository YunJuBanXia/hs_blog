use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password(String);  // 存储哈希后的密码

impl Password {
    pub fn new(raw: String) -> Self {
        todo!("Hash the password and return a Password instance")
    }

    pub fn verify(&self, raw: String) -> bool {
        todo!("Check if the raw password matches the hashed password")
    }

    fn hash(raw: String) -> String {
        // bcrypt or argon2
        todo!("Hash the password using a secure algorithm")
    }
}