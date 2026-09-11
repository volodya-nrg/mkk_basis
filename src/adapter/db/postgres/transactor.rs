use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Pool, Postgres, Transaction};

#[derive(Debug, thiserror::Error)]
pub enum TransactionError<E> {
    #[error("transaction error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("operation error: {0}")]
    Operation(E),
}

#[derive(Clone)]
pub struct Transactor {
    pool: Pool<Postgres>,
}
impl Transactor {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
    pub async fn in_transaction<F, T, E>(&self, f: F) -> Result<T, TransactionError<E>>
    where
        F: AsyncFnOnce(&mut PgConnection) -> Result<T, E>,
    {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;
        let result = f(&mut *tx).await.map_err(TransactionError::Operation)?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn conn(&self) -> Result<PoolConnection<Postgres>, sqlx::Error> {
        let conn = self.pool.acquire().await?;
        Ok(conn)
    }
}
