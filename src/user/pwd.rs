use serde::{Serialize, Deserialize};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password(String);  // 存储哈希后的密码


impl Password {
    pub fn new(raw: String) -> Self {
        Self(Self::hash(raw))
    }


    pub fn verify(&self, raw: String) -> bool {
        let salt = SaltString::from_b64(self.0.as_str()).unwrap();
        let hash = Argon2::default()
            .hash_password(raw.as_bytes(), &salt)
            .expect("argon2 hashing should succeed with default parameters");
        hash.to_string() == self.0
    }


    fn hash(raw: String) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(raw.as_bytes(), &salt)
            .expect("argon2 hashing should succeed with default parameters");

        hash.to_string()
    }
}