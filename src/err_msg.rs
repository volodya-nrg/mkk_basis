use std::fmt;

use super::consts;

pub enum ErrMsg {
    BadFileData,
    EmailAlreadyConfirm,
    EmailNotBeEmpty,
    EmailNotCorrect,
    LoginOrPasswordNotCorrect,
    NeedAcceptAgreement,
    NeedAcceptPrivacyPolicy,
    NoAccessTeamMemberOnly,
    NoRules,
    NotCorrectMultipartForm,
    NotCorrectVerifyEmailCode,
    NotFoundItem,
    NotFoundUser,
    PasswordIsShort,
    PasswordsNotEquals,
    TokenExpired,
    TokenIsNotRefresh,
    TokenNotValid,
    UndefinedTypeImage,
    VerifyCodeNotBeEmpty,
    VerifyYourEmail,
}
impl fmt::Display for ErrMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrMsg::EmailNotCorrect => write!(f, "е-мэйл не корректен"),
            ErrMsg::EmailAlreadyConfirm => write!(f, "е-мэйл уже подтверждён"),
            ErrMsg::EmailNotBeEmpty => write!(f, "отсутствует е-мэйл"),
            ErrMsg::VerifyYourEmail => write!(f, "е-мэйл необходимо верифицировать"),
            ErrMsg::VerifyCodeNotBeEmpty => write!(f, "проверочный код для е-мэйла отсутствует"),
            ErrMsg::PasswordsNotEquals => write!(f, "пароли не равны"),
            ErrMsg::PasswordIsShort => write!(
                f,
                "пароль слишком короткий, нужно более или равно {}",
                consts::MIN_PASSWORD_LEN
            ),
            ErrMsg::NotFoundUser => write!(f, "такой пользователь не найден"),
            ErrMsg::NotFoundItem => write!(f, "запись не найдена"),
            ErrMsg::LoginOrPasswordNotCorrect => write!(f, "логин или пароль не верные"),
            ErrMsg::NeedAcceptAgreement => write!(f, "необходимо принять условия оферты"),
            ErrMsg::NeedAcceptPrivacyPolicy => {
                write!(f, "необходимо принять политику конфиденциальности")
            }
            ErrMsg::NotCorrectVerifyEmailCode => write!(f, "проверочный код е-мэйла не верный"),
            ErrMsg::TokenExpired => write!(f, "токен просрочен"),
            ErrMsg::TokenNotValid => write!(f, "токен не действителен"),
            ErrMsg::TokenIsNotRefresh => write!(f, "токен не является токеном обновления"),
            ErrMsg::NoRules => write!(f, "у вас нет прав на данное действие"),
            ErrMsg::NoAccessTeamMemberOnly => write!(
                f,
                "у вас нет доступа к данному действию, только для участника команды"
            ),
            ErrMsg::NotCorrectMultipartForm => write!(f, "ошибка в обработки формы"),
            ErrMsg::BadFileData => write!(f, "не верные данные файла"),
            ErrMsg::UndefinedTypeImage => write!(f, "не известный тип изображения"),
        }
    }
}
