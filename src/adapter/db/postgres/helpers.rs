use super::super::errors::RepositoryError;

pub fn expect_one_row(rows: u64) -> Result<(), RepositoryError> {
    (rows == 1)
        .then_some(())
        .ok_or(RepositoryError::ExpectedOneRow(rows))
}
