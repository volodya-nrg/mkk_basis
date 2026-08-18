use crate::adapter::db::{RepositoryError, models::TaskComment, postgres::table_basic::TableBasic};
use sqlx::{Pool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct TaskComments {
    pool: Pool<Postgres>,
    table_basic: TableBasic,
}

#[allow(dead_code)]
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
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<TaskComment>, i64), RepositoryError> {
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

        let items: Vec<TaskComment> = query
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
    pub async fn one(&self, item_id: Uuid) -> Result<TaskComment, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE task_comment_id=$1",
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
    pub async fn create(&self, item: TaskComment) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (task_id, user_id, msg) VALUES ($1,$2,$3) RETURNING task_comment_id",
            self.table_basic.name,
        );
        let result = QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(item.msg)
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .get(0);
        
        Ok(result)
    }
    pub async fn update(&self, item: TaskComment) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET task_id=$1, user_id=$2, msg=$3 WHERE task_comment_id=$4",
            self.table_basic.name,
        );
        let result = QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(item.msg)
            .bind(item.task_comment_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::FailedToUpdate)?;
        let amount_updated_rows = result.rows_affected();

        if amount_updated_rows != 1 {
            return Err(RepositoryError::ExpectedOneRow(amount_updated_rows));
        }

        Ok(())
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError> {
        let query = format!(
            "DELETE FROM {} WHERE task_comment_id=$1",
            self.table_basic.name
        );
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
