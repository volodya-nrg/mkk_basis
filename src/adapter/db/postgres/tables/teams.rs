use sqlx::{Pool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

use crate::adapter::db::{RepositoryError, models::Team, postgres::table_basic::TableBasic};

#[derive(Clone)] // из-за axum-state
pub struct Teams {
    pool: Pool<Postgres>,
    table_basic: TableBasic,
}
impl Teams {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            table_basic: TableBasic {
                name: "teams".to_string(),
                fields: vec![
                    "team_id".to_string(),
                    "name".to_string(),
                    "created_by".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                ],
            },
        }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Team>, i64), RepositoryError> {
        let mut common_builder = QueryBuilder::new(format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        ));
        let mut count_builder =
            QueryBuilder::new(format!("SELECT COUNT(*) FROM {}", self.table_basic.name));

        if limit > -1 {
            common_builder.push(" LIMIT ");
            common_builder.push_bind(limit);
        }
        if offset > -1 {
            common_builder.push(" OFFSET ");
            common_builder.push_bind(offset);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(RepositoryError::TransactionError)?;
        let items: Vec<Team> = common_builder
            .build_query_as()
            .fetch_all(&mut *tx)
            .await
            .map_err(RepositoryError::FailedToQuery)?;
        let total = count_builder
            .build_query_scalar()
            .fetch_one(&mut *tx)
            .await
            .map_err(RepositoryError::FailedToCount)?;

        tx.commit()
            .await
            .map_err(RepositoryError::TransactionError)?;

        Ok((items, total))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Team, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE team_id=$1",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build_query_as()
            .bind(item_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::FailedToQuery)?
            .ok_or(RepositoryError::NotFoundRow)
    }
    pub async fn create(&self, item: Team) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (name, created_by) VALUES ($1,$2) RETURNING team_id",
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.created_by)
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .try_get(0)
            .map_err(|e| RepositoryError::Common(e))
    }
    pub async fn update(&self, item: Team) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET name=$1 WHERE team_id=$2", // создателя не меняем
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.team_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::FailedToUpdate)
            .and_then(|result| {
                let rows = result.rows_affected();
                if rows == 1 {
                    Ok(())
                } else {
                    Err(RepositoryError::ExpectedOneRow(rows))
                }
            })
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError> {
        let query = format!("DELETE FROM {} WHERE team_id=$1", self.table_basic.name);
        QueryBuilder::new(query)
            .build()
            .bind(item_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::FailedToDelete)
            .and_then(|result| {
                let rows = result.rows_affected();
                if rows == 1 {
                    Ok(())
                } else {
                    Err(RepositoryError::ExpectedOneRow(rows))
                }
            })
    }
}
