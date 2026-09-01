use super::consts;

pub enum ErrMsg {
    EmailNotCorrect,
    EmailAlreadyConfirm,
    EmailNotBeEmpty,
    VerifyYourEmail,
    VerifyCodeNotBeEmpty,
    PasswordsNotEquals,
    PasswordIsShort,
    NotFoundUser,
    NotFoundItem,
    LoginOrPasswordNotCorrect,
    NeedAcceptAgreement,
    NeedAcceptPrivacyPolicy,
    NotCorrectVerifyEmailCode,
    TokenExpired,
    TokenNotValid,
    TokenIsNotRefresh,
    NoRules,
    NoAccessTeamMemberOnly,
    NotCorrectMultipartForm,
    BadFileData,
    UndefinedTypeImage,
}
impl ErrMsg {
    pub fn as_str(&self) -> String {
        match self {
            ErrMsg::EmailNotCorrect => "е-мэйл не корректен".to_string(),
            ErrMsg::EmailAlreadyConfirm => "е-мэйл уже подтверждён".to_string(),
            ErrMsg::EmailNotBeEmpty => "отсутствует е-мэйл".to_string(),
            ErrMsg::VerifyYourEmail => "е-мэйл необходимо верифицировать".to_string(),
            ErrMsg::VerifyCodeNotBeEmpty => "проверочный код для е-мэйла отсутствует".to_string(),
            ErrMsg::PasswordsNotEquals => "пароли не равны".to_string(),
            ErrMsg::PasswordIsShort => {
                format!(
                    "пароль слишком короткий, нужно более или равно {}",
                    consts::MIN_PASSWORD_LEN
                )
            }
            ErrMsg::NotFoundUser => "такой пользователь не найден".to_string(),
            ErrMsg::NotFoundItem => "запись не найдена".to_string(),
            ErrMsg::LoginOrPasswordNotCorrect => "логин или пароль не верные".to_string(),
            ErrMsg::NeedAcceptAgreement => "необходимо принять условия оферты".to_string(),
            ErrMsg::NeedAcceptPrivacyPolicy => {
                "необходимо принять политику конфиденциальности".to_string()
            }
            ErrMsg::NotCorrectVerifyEmailCode => "проверочный код е-мэйла не верный".to_string(),
            ErrMsg::TokenExpired => "токен просрочен".to_string(),
            ErrMsg::TokenNotValid => "токен не действителен".to_string(),
            ErrMsg::TokenIsNotRefresh => "токен не является токеном обновления".to_string(),
            ErrMsg::NoRules => "у вас нет прав на данное действие".to_string(),
            ErrMsg::NoAccessTeamMemberOnly => {
                "у вас нет доступа к данному действию, только для участника команды".to_string()
            }
            ErrMsg::NotCorrectMultipartForm => "ошибка в обработки формы".to_string(),
            ErrMsg::BadFileData => "не верные данные файла".to_string(),
            ErrMsg::UndefinedTypeImage => "не известный тип изображения".to_string(),
        }
    }
}
