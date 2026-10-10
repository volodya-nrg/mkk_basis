use http::StatusCode;

use crate::adapter::db::errors::RepositoryError;

#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    // отдельная ошибка для транспортного уровня
    #[error("transport error: {status_code} - {public_err}")]
    Transport {
        status_code: StatusCode,
        public_err: String,
        internal_err: Option<String>,
    },
    // Репозитории могут самолично отдать NotFoundRow, поэтому явно проверяем в бизнес-слое это.
    // Если так не сделать, придется везде завязаться на UseCaseError.
    #[error("item not found")]
    ItemNotFound, // тут обобщенная запись нужна (user, team или др.)
    #[error(transparent)]
    Internal(anyhow::Error),
}
impl From<RepositoryError> for UseCaseError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::NotFoundRow => Self::ItemNotFound,
            other => Self::Internal(other.into()),
        }
    }
}
