use sqlx::{Pool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    models::{List, Team},
    postgres::transactor::{TransactionError, Transactor},
    traits::NameAndFields,
};

#[derive(Clone)] // из-за axum-state
pub struct Teams {
    pool: Pool<Postgres>,
    transactor: Transactor,
}
impl NameAndFields for Teams {
    fn get_name(&self) -> &str {
        "teams"
    }
    fn get_fields(&self) -> &[&str] {
        &["team_id", "name", "created_by", "created_at", "updated_at"]
    }
}
impl Teams {
    pub fn new(pool: Pool<Postgres>, transactor: Transactor) -> Self {
        Self { pool, transactor }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<List<Team>, RepositoryError> {
        let mut common_builder = QueryBuilder::new(format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.get_fields().join(","),
            self.get_name(),
        ));
        let mut count_builder =
            QueryBuilder::new(format!("SELECT COUNT(*) FROM {}", self.get_name()));

        if limit > -1 {
            common_builder.push(" LIMIT ");
            common_builder.push_bind(limit);
        }
        if offset > -1 {
            common_builder.push(" OFFSET ");
            common_builder.push_bind(offset);
        }

        self.transactor
            .execute(async |tx| {
                let items: Vec<Team> = common_builder
                    .build_query_as()
                    .fetch_all(tx.as_mut())
                    .await
                    .map_err(RepositoryError::FailedToQuery)?;
                let total = count_builder
                    .build_query_scalar()
                    .fetch_one(tx.as_mut())
                    .await
                    .map_err(RepositoryError::FailedToCount)?;

                Ok(List(items, total))
            })
            .await
            .map_err(|e| match e {
                TransactionError::Database(sqlx_err) => RepositoryError::Common(sqlx_err),
                TransactionError::Operation(repo_err) => repo_err,
            })
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Team, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE team_id=$1",
            self.get_fields().join(","),
            self.get_name(),
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
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.created_by)
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .try_get(0)
            .map_err(RepositoryError::Common)
    }
    pub async fn update(&self, item: Team) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET name=$1 WHERE team_id=$2", // создателя не меняем
            self.get_name(),
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
        let query = format!("DELETE FROM {} WHERE team_id=$1", self.get_name());
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
