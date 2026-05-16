use serde::{Deserialize, Serialize};
use validator::Validate;



#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct EmailVerificationSerializer {
    #[validate(email)]
    pub email: String,

    pub code: String,
}