use serde::{Serialize, Deserialize};

use crate::user::pwd::Password;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: u32,
    name: String,
    email: String,
    password: Password,
}

impl User {
    
}