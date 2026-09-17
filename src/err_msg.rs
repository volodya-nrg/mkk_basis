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
            Self::EmailNotCorrect => write!(f, "е-мэйл не корректен"),
            Self::EmailAlreadyConfirm => write!(f, "е-мэйл уже подтверждён"),
            Self::EmailNotBeEmpty => write!(f, "отсутствует е-мэйл"),
            Self::VerifyYourEmail => write!(f, "е-мэйл необходимо верифицировать"),
            Self::VerifyCodeNotBeEmpty => write!(f, "проверочный код для е-мэйла отсутствует"),
            Self::PasswordsNotEquals => write!(f, "пароли не равны"),
            Self::PasswordIsShort => write!(
                f,
                "пароль слишком короткий, нужно более или равно {}",
                consts::MIN_PASSWORD_LEN
            ),
            Self::NotFoundUser => write!(f, "такой пользователь не найден"),
            Self::NotFoundItem => write!(f, "запись не найдена"),
            Self::LoginOrPasswordNotCorrect => write!(f, "логин или пароль не верные"),
            Self::NeedAcceptAgreement => write!(f, "необходимо принять условия оферты"),
            Self::NeedAcceptPrivacyPolicy => {
                write!(f, "необходимо принять политику конфиденциальности")
            }
            Self::NotCorrectVerifyEmailCode => write!(f, "проверочный код е-мэйла не верный"),
            Self::TokenExpired => write!(f, "токен просрочен"),
            Self::TokenNotValid => write!(f, "токен не действителен"),
            Self::TokenIsNotRefresh => write!(f, "токен не является токеном обновления"),
            Self::NoRules => write!(f, "у вас нет прав на данное действие"),
            Self::NoAccessTeamMemberOnly => write!(
                f,
                "у вас нет доступа к данному действию, только для участника команды"
            ),
            Self::NotCorrectMultipartForm => write!(f, "ошибка в обработки формы"),
            Self::BadFileData => write!(f, "не верные данные файла"),
            Self::UndefinedTypeImage => write!(f, "не известный тип изображения"),
        }
    }
}
