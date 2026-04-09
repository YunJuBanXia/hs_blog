use serde::{Serialize, Deserialize};
use argon2::{
    Argon2, PasswordHasher, PasswordVerifier, password_hash::{SaltString, rand_core::OsRng}
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password(String);  // 存储哈希后的密码


impl Password {
    pub fn new(raw: String) -> Self {
        // 密码一创建就进行哈希, Password 对象永远存储哈希后的密码
        Self(Self::hash(raw))
    }


    pub fn verify(&self, raw: String) -> bool {
        let parsed_hash = argon2::PasswordHash::new(&self.0)
            .expect("stored password hash should be valid");

        Argon2::default()
            .verify_password(raw.as_bytes(), &parsed_hash)
            .is_ok()
    }


    fn hash(raw: String) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(raw.as_bytes(), &salt)
            .expect("argon2 hashing should succeed with default parameters");

        hash.to_string()
    }
}


// cargo test pwd
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwd_hash_and_verify() {
        let raw = "my_password".to_string();
        let pwd = Password::new(raw.clone());

        assert!(pwd.verify(raw));
    }
}