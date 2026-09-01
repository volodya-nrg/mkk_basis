pub mod models;
pub mod postgres;
pub mod transactor;

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum RepositoryError {
    #[error("failed to query: {0}")]
    FailedToQuery(sqlx::Error),
    #[error("failed to count: {0}")]
    FailedToCount(sqlx::Error),
    #[error("failed to insert: {0}")]
    FailedToInsert(sqlx::Error),
    #[error("failed to update: {0}")]
    FailedToUpdate(sqlx::Error),
    #[error("failed to delete: {0}")]
    FailedToDelete(sqlx::Error),

    #[error("failed to tx: {0}")]
    TransactionError(sqlx::Error),
    #[error("not found row")]
    NotFoundRow,
    #[error("expected one row, but has {0}")]
    ExpectedOneRow(u64),
    #[error("{0}")]
    Common(sqlx::Error),
}
