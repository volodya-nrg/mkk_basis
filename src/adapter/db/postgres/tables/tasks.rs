use crate::adapter::db::errors::Error;
use crate::adapter::db::models::Task;
use crate::adapter::db::postgres::table_basic::TableBasic;
use sqlx::{Pool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

#[derive(Debug)]
pub enum Status {
    Start,
    Todo,
    Done,
    Cancelled,
}

#[derive(Clone)]
pub struct Tasks {
    pool: Pool<Postgres>,
    table_basic: TableBasic,
}
impl Tasks {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            table_basic: TableBasic {
                name: "tasks".to_string(),
                fields: vec![
                    "task_id".to_string(),
                    "name".to_string(),
                    "description".to_string(),
                    "created_by".to_string(),
                    "team_id".to_string(),
                    "assignee_id".to_string(),
                    "status::text as status".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                ],
            },
        }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Task>, i64), String> {
        let query = format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        // let query = query.replace(",status,", ",status::text as status,");

        let mut query_builder = QueryBuilder::new(query);

        if limit > -1 {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit);
        }
        if offset > -1 {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset);
        }

        let items: Vec<Task> = query_builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("failed to query: {e}"))?;
        let total: (i64,) =
            QueryBuilder::new(format!("SELECT COUNT(*) FROM {}", self.table_basic.name)) // возвращает такой же диапазон как и i64
                .build_query_as()
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("failed to count: {e}"))?;

        Ok((items, total.0))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Task, Error> {
        let query = format!(
            "SELECT {} FROM {} WHERE task_id=$1",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        // let query = query.replace(",status,", ",status::text as status,");
        let opt = QueryBuilder::new(query)
            .build_query_as()
            .bind(item_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Any(format!("failed to query: {e}")))?;

        match opt {
            Some(v) => Ok(v),
            None => Err(Error::NotFound),
        }
    }
    pub async fn create(&self, item: Task) -> Result<Uuid, String> {
        let query = format!(
            "INSERT INTO {} (name, description, created_by, team_id, assignee_id, status) VALUES ($1,$2,$3,$4,$5,$6::task_status_enum) RETURNING task_id",
            self.table_basic.name,
        );
        let result = QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.description)
            .bind(item.created_by)
            .bind(item.team_id)
            .bind(item.assignee_id)
            .bind(item.status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("failed to insert: {e}"))?
            .get(0);

        Ok(result)
    }
    pub async fn update(&self, item: Task) -> Result<(), String> {
        let query = format!(
            "UPDATE {} SET name=$1, description=$2, created_by=$3, team_id=$4, assignee_id=$5, status=$6::task_status_enum WHERE task_id=$7",
            self.table_basic.name,
        );
        let result = QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.description)
            .bind(item.created_by)
            .bind(item.team_id)
            .bind(item.assignee_id)
            .bind(item.status)
            .bind(item.task_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("failed to update: {e}"))?;
        let amount_updated_rows = result.rows_affected();

        if amount_updated_rows != 1 {
            let err_msg = format!(
                "expected update one row, but update {}",
                amount_updated_rows
            )
            .to_string();
            return Err(err_msg);
        }

        Ok(())
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), String> {
        let query = format!("DELETE FROM {} WHERE task_id=$1", self.table_basic.name);
        let result = QueryBuilder::new(query)
            .build()
            .bind(item_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("failed to delete: {e}"))?;
        let amount_updated_rows = result.rows_affected();

        if amount_updated_rows != 1 {
            let err_msg = format!(
                "expected delete one row, but delete {}",
                amount_updated_rows
            )
            .to_string();
            return Err(err_msg);
        }

        Ok(())
    }
}
