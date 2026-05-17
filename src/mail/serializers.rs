use serde::{Deserialize, Serialize};
use validator::Validate;


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SendVerificationEmailSerializer {
    #[validate(email)]
    pub email: String,
}
