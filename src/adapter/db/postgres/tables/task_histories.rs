use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    models::{List, TaskHistory},
    traits::NameAndFields,
};

#[derive(Clone, Default)]
pub struct TaskHistories {}
impl NameAndFields for TaskHistories {
    fn get_name(&self) -> &str {
        "task_histories"
    }
    fn get_fields(&self) -> &[&str] {
        &["task_history_id", "task_id", "user_id", "msg", "created_at"]
    }
}
impl TaskHistories {
    pub fn new() -> Self {
        Self {}
    }
    #[allow(dead_code)]
    pub async fn list(
        &self,
        executor: &mut sqlx::PgConnection,
        limit: i32,
        offset: i32,
    ) -> Result<List<TaskHistory>, RepositoryError> {
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

        let items: Vec<TaskHistory> = common_builder
            .build_query_as()
            .fetch_all(executor.as_mut())
            .await
            .map_err(RepositoryError::FailedToQuery)?;
        let total = count_builder
            .build_query_scalar()
            .fetch_one(executor.as_mut())
            .await
            .map_err(RepositoryError::FailedToCount)?;

        Ok(List(items, total))
    }
    #[allow(dead_code)]
    pub async fn one(
        &self,
        executor: &mut sqlx::PgConnection,
        item_id: Uuid,
    ) -> Result<TaskHistory, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE task_history_id=$1",
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
    pub async fn by_task_id(
        &self,
        executor: &mut sqlx::PgConnection,
        task_id: Uuid,
    ) -> Result<Vec<TaskHistory>, RepositoryError> {
        QueryBuilder::new(format!(
            "SELECT {} FROM {} WHERE task_id=$1 ORDER BY created_at DESC",
            self.get_fields().join(","),
            self.get_name(),
        ))
        .build_query_as()
        .bind(task_id)
        .fetch_all(executor)
        .await
        .map_err(RepositoryError::FailedToQuery)
    }
    pub async fn create(
        &self,
        executor: &mut sqlx::PgConnection,
        item: TaskHistory,
    ) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (task_id, user_id, msg) VALUES ($1,$2,$3) RETURNING task_history_id",
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(item.msg)
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
        item: TaskHistory,
    ) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET task_id=$1, user_id=$2, msg=$3 WHERE task_history_id=$4",
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(item.msg)
            .bind(item.task_history_id)
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
    #[allow(dead_code)]
    pub async fn delete(
        &self,
        executor: &mut sqlx::PgConnection,
        item_id: Uuid,
    ) -> Result<(), RepositoryError> {
        let query = format!("DELETE FROM {} WHERE task_history_id=$1", self.get_name());
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
