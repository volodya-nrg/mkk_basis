use sqlx::{Pool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

use crate::adapter::db::{RepositoryError, models::TaskComment, postgres::table_basic::TableBasic};

#[derive(Clone)]
pub struct TaskComments {
    pool: Pool<Postgres>,
    table_basic: TableBasic,
}

impl TaskComments {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            table_basic: TableBasic {
                name: "task_comments".to_string(),
                fields: vec![
                    "task_comment_id".to_string(),
                    "task_id".to_string(),
                    "user_id".to_string(),
                    "msg".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                ],
            },
        }
    }
    pub async fn list(
        &self,
        task_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<TaskComment>, i64), RepositoryError> {
        let mut common_builder = QueryBuilder::new(format!(
            "SELECT {} FROM {} WHERE task_id=",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        ));
        let mut count_builder = QueryBuilder::new(format!(
            "SELECT COUNT(*) FROM {} WHERE task_id=$1",
            self.table_basic.name
        ));

        common_builder.push_bind(task_id);
        common_builder.push(" ORDER BY created_at DESC");

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
        let items: Vec<TaskComment> = common_builder
            .build_query_as()
            .fetch_all(&mut *tx)
            .await
            .map_err(RepositoryError::FailedToQuery)?;
        let total = count_builder
            .build_query_scalar()
            .bind(task_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(RepositoryError::FailedToCount)?;

        tx.commit()
            .await
            .map_err(RepositoryError::TransactionError)?;

        Ok((items, total))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<TaskComment, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE task_comment_id=$1",
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
    pub async fn create(&self, item: TaskComment) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (task_id, user_id, msg) VALUES ($1,$2,$3) RETURNING task_comment_id",
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(item.msg)
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .try_get(0)
            .map_err(|e| RepositoryError::Common(e))
    }
    pub async fn update(&self, item: TaskComment) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET task_id=$1, user_id=$2, msg=$3 WHERE task_comment_id=$4",
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(item.msg)
            .bind(item.task_comment_id)
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
        let query = format!(
            "DELETE FROM {} WHERE task_comment_id=$1",
            self.table_basic.name
        );
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
