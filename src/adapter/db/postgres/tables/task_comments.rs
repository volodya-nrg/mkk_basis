use sqlx::{AssertSqlSafe, Pool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    models::{List, TaskComment},
    postgres::table_basic::TableBasic,
};

#[derive(Clone)]
pub struct TaskComments {
    table_basic: TableBasic,
}

impl TaskComments {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            table_basic: TableBasic {
                pool,
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
    ) -> Result<List<TaskComment>, RepositoryError> {
        let mut query_common = format!(
            "SELECT {} FROM {}",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        let mut query_count = format!("SELECT COUNT(*) as count FROM {}", self.table_basic.name);
        let mut params: Vec<(String, String)> = vec![];

        params.push((
            format!("task_id=${}::uuid", params.len() + 1),
            task_id.to_string(),
        ));

        if !params.is_empty() {
            let fields = params
                .iter()
                .map(|(k, _)| k.to_string())
                .collect::<Vec<String>>()
                .join(" AND ");

            let where_str = format!(" WHERE {}", fields);
            query_common += where_str.as_str();
            query_count += where_str.as_str();
        }

        let mut tx = self
            .table_basic
            .pool
            .begin()
            .await
            .map_err(RepositoryError::TransactionError)?;
        let total = self
            .table_basic
            .count(&mut tx, Some(query_count), params.clone())
            .await?;

        query_common.push_str(" ORDER BY created_at DESC");

        if limit > -1 {
            query_common += format!(" LIMIT ${}::bigint", params.len() + 1).as_str();
            params.push(("".to_string(), limit.to_string()));
        }
        if offset > -1 {
            query_common += format!(" OFFSET ${}::bigint", params.len() + 1).as_str();
            params.push(("".to_string(), offset.to_string()));
        }

        let mut prepare_common = sqlx::query_as::<_, TaskComment>(AssertSqlSafe(query_common));
        for (_, v) in params.iter() {
            prepare_common = prepare_common.bind(v);
        }

        let items = prepare_common
            .fetch_all(&mut *tx)
            .await
            .map_err(RepositoryError::FailedToQuery)?;

        tx.commit()
            .await
            .map_err(RepositoryError::TransactionError)?;

        Ok(List(items, total))
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
            .fetch_optional(&self.table_basic.pool)
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
            .fetch_one(&self.table_basic.pool)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .try_get(0)
            .map_err(RepositoryError::Common)
    }
    #[allow(dead_code)]
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
            .execute(&self.table_basic.pool)
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
            .execute(&self.table_basic.pool)
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
