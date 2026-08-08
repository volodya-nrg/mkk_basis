use super::consts;

pub enum ErrMsg {
    EmailNotCorrect,
    PasswordsNotEquals,
    AcceptAgree,
    PasswordIsShort,
}
impl ErrMsg {
    pub fn as_str(&self) -> String {
        match self {
            ErrMsg::EmailNotCorrect => "е-мэйл не корректен".to_string(),
            ErrMsg::PasswordsNotEquals => "пароли не равны".to_string(),
            ErrMsg::AcceptAgree => "примите соглашение".to_string(),
            ErrMsg::PasswordIsShort => {
                format!(
                    "пароль слишком короткий, нужно более или равно {}",
                    consts::MIN_PASSWORD_LEN
                )
            }
            _ => "undefined message".to_string(),
        }
    }
}
