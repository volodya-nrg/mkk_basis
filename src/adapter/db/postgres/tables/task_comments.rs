use sqlx::{AssertSqlSafe, QueryBuilder, Row};
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    models::{List, TaskComment},
    traits::NameAndFields,
};

#[derive(Clone, Default)]
pub struct TaskComments {}
impl NameAndFields for TaskComments {
    fn get_name(&self) -> &str {
        "task_comments"
    }
    fn get_fields(&self) -> &[&str] {
        &[
            "task_comment_id",
            "task_id",
            "user_id",
            "msg",
            "created_at",
            "updated_at",
        ]
    }
}
impl TaskComments {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn list(
        &self,
        executor: &mut sqlx::PgConnection, // везде стоит это, Executor не подходит, тк нужно executor иногда использовать несколько раз
        task_id: &Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<List<TaskComment>, RepositoryError> {
        let mut query_common = format!(
            "SELECT {} FROM {}",
            self.get_fields().join(","),
            self.get_name(),
        );
        let mut query_count = format!("SELECT COUNT(*) as count FROM {}", self.get_name());
        let mut params: Vec<(String, String)> = vec![];

        params.push((
            format!("task_id=${}::uuid", params.len() + 1),
            task_id.to_string(),
        ));

        if !params.is_empty() {
            let fields = params
                .iter()
                .map(|(field_name, _)| field_name.to_string())
                .collect::<Vec<String>>()
                .join(" AND ");

            let where_str = format!(" WHERE {}", fields);
            query_common += where_str.as_str();
            query_count += where_str.as_str();
        }

        let mut prepare_count = sqlx::query_scalar(AssertSqlSafe(query_count));
        let params_copy = params.clone();
        for (_, v) in params_copy.iter() {
            prepare_count = prepare_count.bind(v);
        }

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
            .fetch_all(executor.as_mut())
            .await
            .map_err(RepositoryError::FailedToQuery)?;
        let total = prepare_count
            .fetch_one(executor.as_mut())
            .await
            .map_err(RepositoryError::FailedToCount)?;

        Ok(List(items, total))
    }
    pub async fn one(
        &self,
        executor: &mut sqlx::PgConnection,
        item_id: &Uuid,
    ) -> Result<TaskComment, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE task_comment_id=$1",
            self.get_fields().join(","),
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build_query_as()
            .bind(item_id)
            .fetch_optional(executor)
            .await
            .map_err(RepositoryError::FailedToQuery)?
            .ok_or(RepositoryError::NotFoundRow)
    }
    pub async fn create(
        &self,
        executor: &mut sqlx::PgConnection,
        item: TaskComment,
    ) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (task_id, user_id, msg) VALUES ($1,$2,$3) RETURNING task_comment_id",
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(&item.msg)
            .fetch_one(executor)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .try_get(0)
            .map_err(RepositoryError::Common)
    }
    #[allow(dead_code)]
    pub async fn update(
        &self,
        executor: &mut sqlx::PgConnection,
        item: TaskComment,
    ) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET task_id=$1, user_id=$2, msg=$3 WHERE task_comment_id=$4",
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(&item.msg)
            .bind(item.task_comment_id)
            .execute(executor)
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
    pub async fn delete(
        &self,
        executor: &mut sqlx::PgConnection,
        item_id: &Uuid,
    ) -> Result<(), RepositoryError> {
        let query = format!("DELETE FROM {} WHERE task_comment_id=$1", self.get_name());
        QueryBuilder::new(query)
            .build()
            .bind(item_id)
            .execute(executor)
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
