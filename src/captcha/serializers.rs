use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct CaptchaResponse {
    pub captcha_id: String,
    pub image_base64: String,  // base64 编码的图片数据
}


