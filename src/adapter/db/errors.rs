// thiserror так же реализует и стандартную ошибку (см. ниже). Debug необходим.
// После этого нашу ошибку можно будет связывать в цепочку с другими ошибками из std-библиотеки.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("sqlx error: {0}")]
    FailedSQLX(#[from] sqlx::Error), // обобщаем все к одному
    #[error("not found row")]
    NotFoundRow,
    #[error("expected one row, but has {0}")]
    ExpectedOneRow(u64),
    #[error("wrong database")]
    WrongDatabase,
}
// impl std::error::Error for RepositoryError {}
