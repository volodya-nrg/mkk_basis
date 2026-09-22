use super::consts;

#[derive(Debug, thiserror::Error)]
pub enum ErrMsg {
    #[error("не верные данные файла")]
    BadFileData,
    #[error("е-мэйл уже подтверждён")]
    EmailAlreadyConfirm,
    #[error("отсутствует е-мэйл")]
    EmailNotBeEmpty,
    #[error("е-мэйл не корректен")]
    EmailNotCorrect,
    #[error("логин или пароль не верные")]
    LoginOrPasswordNotCorrect,
    #[error("необходимо принять условия оферты")]
    NeedAcceptAgreement,
    #[error("необходимо принять политику конфиденциальности")]
    NeedAcceptPrivacyPolicy,
    #[error("у вас нет доступа к данному действию, только для участника команды")]
    NoAccessTeamMemberOnly,
    #[error("у вас нет прав на данное действие")]
    NoRules,
    #[error("ошибка в обработки формы")]
    NotCorrectMultipartForm,
    #[error("проверочный код е-мэйла не верный")]
    NotCorrectVerifyEmailCode,
    #[error("запись не найдена")]
    NotFoundItem,
    #[error("пользователь не найден")]
    NotFoundUser,
    #[error(
        "пароль слишком короткий, нужно более или равно {0}",
        consts::MIN_PASSWORD_LEN
    )]
    PasswordIsShort,
    #[error("пароли не равны")]
    PasswordsNotEquals,
    #[error("токен просрочен")]
    TokenExpired,
    #[error("токен не является токеном обновления")]
    TokenIsNotRefresh,
    #[error("токен не действителен")]
    TokenNotValid,
    #[error("не известный тип изображения")]
    UndefinedTypeImage,
    #[error("проверочный код для е-мэйла отсутствует")]
    VerifyCodeNotBeEmpty,
    #[error("е-мэйл необходимо верифицировать")]
    VerifyYourEmail,
}

// После этого нашу ошибку можно будет связывать в цепочку с другими ошибками из стандартной библиотеки.
// Debug необходим.
// impl std::error::Error for ErrMsg {}
