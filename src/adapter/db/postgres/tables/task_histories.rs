use async_trait::async_trait;
use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    internal::Table,
    models::{List, TaskHistory},
    postgres::helpers::expect_one_row,
    storage::{AnyConnection, TaskHistoriesTable},
};

pub struct TaskHistories {}

impl Table for TaskHistories {
    fn get_table_name(&self) -> &str {
        "task_histories"
    }
    fn get_fields(&self) -> &[&str] {
        &["task_history_id", "task_id", "user_id", "msg", "created_at"]
    }
}

#[async_trait]
impl TaskHistoriesTable for TaskHistories {
    async fn list(
        &self,
        conn: &mut AnyConnection<'_>,
        limit: i32,
        offset: i32,
    ) -> Result<List<TaskHistory>, RepositoryError> {
        let pg = conn.as_postgres_mut()?;
        let mut common_builder = QueryBuilder::new(format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.get_fields().join(","),
            self.get_table_name(),
        ));
        let mut count_builder =
            QueryBuilder::new(format!("SELECT COUNT(*) FROM {}", self.get_table_name()));

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
            .fetch_all(pg.as_mut())
            .await?;
        let total = count_builder
            .build_query_scalar()
            .fetch_one(pg.as_mut())
            .await?;

        Ok(List(items, total))
    }
    async fn one(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<TaskHistory, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE task_history_id=$1",
            self.get_fields().join(","),
            self.get_table_name(),
        );
        QueryBuilder::new(query)
            .build_query_as()
            .bind(item_id)
            .fetch_optional(conn.as_postgres_mut()?)
            .await?
            .ok_or(RepositoryError::NotFoundRow)
    }
    async fn by_task_id(
        &self,
        conn: &mut AnyConnection<'_>,
        task_id: Uuid,
    ) -> Result<Vec<TaskHistory>, RepositoryError> {
        QueryBuilder::new(format!(
            "SELECT {} FROM {} WHERE task_id=$1 ORDER BY created_at DESC",
            self.get_fields().join(","),
            self.get_table_name(),
        ))
        .build_query_as()
        .bind(task_id)
        .fetch_all(conn.as_postgres_mut()?)
        .await
        .map_err(|e| e.into())
    }
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: TaskHistory,
    ) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (task_id, user_id, msg) VALUES ($1,$2,$3) RETURNING task_history_id",
            self.get_table_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(&item.msg)
            .fetch_one(conn.as_postgres_mut()?)
            .await?
            .try_get(0)
            .map_err(|e| e.into())
    }
    async fn update(
        &self,
        conn: &mut AnyConnection<'_>,
        item: TaskHistory,
    ) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET task_id=$1, user_id=$2, msg=$3 WHERE task_history_id=$4",
            self.get_table_name(),
        );
        let amount_rows = QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(&item.msg)
            .bind(item.task_history_id)
            .execute(conn.as_postgres_mut()?)
            .await?
            .rows_affected();

        expect_one_row(amount_rows)
    }
    async fn delete(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<(), RepositoryError> {
        let query = format!(
            "DELETE FROM {} WHERE task_history_id=$1",
            self.get_table_name()
        );
        let amount_rows = QueryBuilder::new(query)
            .build()
            .bind(item_id)
            .execute(conn.as_postgres_mut()?)
            .await?
            .rows_affected();

        expect_one_row(amount_rows)
    }
}
