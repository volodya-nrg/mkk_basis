use crate::adapter::db::RepositoryError;
use crate::adapter::db::models::Team;
use crate::adapter::db::postgres::table_basic::TableBasic;
use sqlx::{Pool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

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
        let mut query = QueryBuilder::new(format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        ));

        if limit > -1 {
            query.push(" LIMIT ");
            query.push_bind(limit);
        }
        if offset > -1 {
            query.push(" OFFSET ");
            query.push_bind(offset);
        }

        let items: Vec<Team> = query
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::FailedToQuery)?;
        let total: (i64,) =
            QueryBuilder::new(format!("SELECT COUNT(*) FROM {}", self.table_basic.name)) // возвращает такой же диапазон как и i64
                .build_query_as()
                .fetch_one(&self.pool)
                .await
                .map_err(RepositoryError::FailedToCount)?;

        Ok((items, total.0))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Team, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE team_id=$1",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        let opt = QueryBuilder::new(query)
            .build_query_as()
            .bind(item_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::FailedToQuery)?;
        match opt {
            Some(v) => Ok(v),
            None => Err(RepositoryError::NotFoundRow),
        }
    }
    pub async fn create(&self, item: Team) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (name, created_by) VALUES ($1,$2) RETURNING team_id",
            self.table_basic.name,
        );
        let result = QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.created_by)
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .get(0);

        Ok(result)
    }
    #[allow(dead_code)]
    pub async fn update(&self, item: Team) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET name=$1, created_by=$2 WHERE team_id=$3",
            self.table_basic.name,
        );
        let result = QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.created_by)
            .bind(item.team_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::FailedToUpdate)?;
        let amount_updated_rows = result.rows_affected();

        if amount_updated_rows != 1 {
            return Err(RepositoryError::ExpectedOneRow(amount_updated_rows));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError> {
        let query = format!("DELETE FROM {} WHERE team_id=$1", self.table_basic.name);
        let result = QueryBuilder::new(query)
            .build()
            .bind(item_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::FailedToDelete)?;
        let amount_updated_rows = result.rows_affected();

        if amount_updated_rows != 1 {
            return Err(RepositoryError::ExpectedOneRow(amount_updated_rows));
        }

        Ok(())
    }
}
