use fake::Dummy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Dummy)]
pub struct RequestRegister {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub is_agree: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseRegister {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub is_agree: bool,
}
