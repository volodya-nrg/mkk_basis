use async_trait::async_trait;
use sqlx::{Postgres, Transaction as SQLXTransaction};

use crate::adapter::db::{
    errors::RepositoryError,
    storage::{AnyConnection, Transaction},
};

pub struct Transactor {
    pub tx: SQLXTransaction<'static, Postgres>,
}

impl Transactor {
    pub const fn new(tx: SQLXTransaction<'static, Postgres>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl Transaction for Transactor {
    async fn get_conn(&mut self) -> Result<AnyConnection<'_>, RepositoryError> {
        Ok(AnyConnection::Postgres(&mut self.tx))
    }
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
        Ok(self
            .tx
            // .borrow()
            .commit()
            .await?)
        // match self.tx {
        //     AnyTransaction::Postgres(tx) => tx.commit().await.map_err(RepositoryError::Common),
        //     AnyTransaction::Sqlite(tx) => tx.commit().await.map_err(RepositoryError::Common),
        // }
    }
    async fn rollback(self: Box<Self>) -> Result<(), RepositoryError> {
        Ok(self
            .tx
            // .borrow()
            .rollback()
            .await?)
        // match self.tx {
        //     AnyTransaction::Postgres(tx) => tx.rollback().await.map_err(RepositoryError::Common),
        //     AnyTransaction::Sqlite(tx) => tx.rollback().await.map_err(RepositoryError::Common),
        // }
    }
}
