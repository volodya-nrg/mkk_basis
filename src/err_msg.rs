use super::consts;

pub enum ErrMsg {
    EmailNotCorrect,
    EmailAlreadyConfirm,
    PasswordsNotEquals,
    PasswordIsShort,
    NotFoundUser,
    LoginOrPasswordNotCorrect,
    NeedAcceptAgreement,
    NeedAcceptPrivacyPolicy,
    NotCorrectVerifyEmailCode,
}
impl ErrMsg {
    pub fn as_str(&self) -> String {
        match self {
            ErrMsg::EmailNotCorrect => "е-мэйл не корректен".to_string(),
            ErrMsg::EmailAlreadyConfirm => "е-мэйл уже подтверждён".to_string(),
            ErrMsg::PasswordsNotEquals => "пароли не равны".to_string(),
            ErrMsg::PasswordIsShort => {
                format!(
                    "пароль слишком короткий, нужно более или равно {}",
                    consts::MIN_PASSWORD_LEN
                )
            }
            ErrMsg::NotFoundUser => "такой пользователь не найден".to_string(),
            ErrMsg::LoginOrPasswordNotCorrect => "логин или пароль не верные".to_string(),
            ErrMsg::NeedAcceptAgreement => "необходимо принять условия оферты".to_string(),
            ErrMsg::NeedAcceptPrivacyPolicy => {
                "необходимо принять политику конфиденциальности".to_string()
            }
            ErrMsg::NotCorrectVerifyEmailCode => "проверочный код е-мэйла не верный".to_string(),
        }
    }
}
