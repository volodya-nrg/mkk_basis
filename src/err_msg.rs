use super::consts;

pub enum ErrMsg {
    EmailNotCorrect,
    PasswordsNotEquals,
    PasswordIsShort,
    NotFoundUser,
    LoginOrPasswordNotCorrect,
}
impl ErrMsg {
    pub fn as_str(&self) -> String {
        match self {
            ErrMsg::EmailNotCorrect => "е-мэйл не корректен".to_string(),
            ErrMsg::PasswordsNotEquals => "пароли не равны".to_string(),
            ErrMsg::PasswordIsShort => {
                format!(
                    "пароль слишком короткий, нужно более или равно {}",
                    consts::MIN_PASSWORD_LEN
                )
            }
            ErrMsg::NotFoundUser => "такой пользователь не найден".to_string(),
            ErrMsg::LoginOrPasswordNotCorrect => "логин или пароль не верные".to_string(),
        }
    }
}
