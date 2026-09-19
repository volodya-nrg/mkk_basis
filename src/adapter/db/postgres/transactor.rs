use sqlx::pool::PoolConnection;
use sqlx::{AssertSqlSafe, PgConnection, Pool, Postgres};

// thiserror тут не нужен, т.к. берутся непосредственно их данные
#[derive(Debug)]
pub enum TransactionError<E> {
    Database(sqlx::Error), // Database(#[from] sqlx::Error),
    Operation(E),
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted, // default
    RepeatableRead,
    Serializable,
    None,
}
impl IsolationLevel {
    const fn as_sql(&self) -> &'static str {
        match self {
            Self::ReadUncommitted => " ISOLATION LEVEL READ UNCOMMITTED",
            Self::ReadCommitted => " ISOLATION LEVEL READ COMMITTED",
            Self::RepeatableRead => " ISOLATION LEVEL REPEATABLE READ",
            Self::Serializable => " ISOLATION LEVEL SERIALIZABLE",
            Self::None => "",
        }
    }
}

#[derive(Clone)]
pub struct Transactor {
    pool: Pool<Postgres>,
    level: IsolationLevel,
}

impl Transactor {
    pub const fn new(pool: Pool<Postgres>, level: IsolationLevel) -> Self {
        Self { pool, level }
    }
    pub async fn in_transaction<F, T, E>(&self, f: F) -> Result<T, TransactionError<E>>
    where
        F: AsyncFnOnce(&mut PgConnection) -> Result<T, E>,
    {
        let mut tx = self
            .pool
            .begin_with(AssertSqlSafe(format!("BEGIN{}", self.level.as_sql())))
            .await
            .map_err(TransactionError::Database)?;
        let result = f(&mut *tx).await.map_err(TransactionError::Operation)?;

        tx.commit()
            .await
            .map_err(|e| TransactionError::Database(e))?;

        Ok(result)
    }
    pub async fn conn(&self) -> Result<PoolConnection<Postgres>, sqlx::Error> {
        self.pool.acquire().await
    }
}
