use http::StatusCode;

// другие лишние преобразования уберем, оставим только метку для транспорта
#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    #[error("transport error: {status_code} - {public_err}")]
    Transport {
        status_code: StatusCode,
        public_err: String,
        internal_err: Option<String>,
    },
    #[error("user not found")]
    UserNotFound,
}
