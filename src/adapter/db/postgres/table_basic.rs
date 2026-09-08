use sqlx::{AssertSqlSafe, Pool, Postgres, Transaction};

use crate::adapter::db::errors::RepositoryError;

#[derive(Clone)] // из-за axum-state
pub struct TableBasic {
    pub pool: Pool<Postgres>,
    pub name: String,
    pub fields: Vec<String>,
}
impl TableBasic {
    pub async fn count(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        query: Option<String>,
        params: Vec<(String, String)>,
    ) -> Result<i64, RepositoryError> {
        let mut query_loc = format!("SELECT COUNT(*) FROM {}", self.name);

        if let Some(q) = query {
            query_loc = q;
        }

        let mut builder = sqlx::query_scalar(AssertSqlSafe(query_loc));
        for (_, v) in params.iter() {
            builder = builder.bind(v);
        }

        builder
            .fetch_one(tx.as_mut())
            .await
            .map_err(RepositoryError::FailedToCount)
    }
}
