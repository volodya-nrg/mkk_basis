use crate::adapter::db::RepositoryError;
use crate::adapter::db::models::Task;
use crate::adapter::db::postgres::table_basic::TableBasic;
use sqlx::{Pool, Postgres, QueryBuilder, Row};
use std::fmt::{self, Formatter};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Status {
    Start,
    Todo,
    Done,
    Cancelled,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Status::Start => "start",
            Status::Todo => "todo",
            Status::Done => "done",
            Status::Cancelled => "cancelled",
        };
        write!(f, "{}", s)
    }
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
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Task>, i64), RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
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
            .map_err(RepositoryError::FailedToQuery)?;
        let total: (i64,) =
            QueryBuilder::new(format!("SELECT COUNT(*) FROM {}", self.table_basic.name)) // возвращает такой же диапазон как и i64
                .build_query_as()
                .fetch_one(&self.pool)
                .await
                .map_err(RepositoryError::FailedToCount)?;

        Ok((items, total.0))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Task, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE task_id=$1",
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
    pub async fn create(&self, item: Task) -> Result<Uuid, RepositoryError> {
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
            .map_err(RepositoryError::FailedToInsert)?
            .get(0);

        Ok(result)
    }
    pub async fn update(&self, item: Task) -> Result<(), RepositoryError> {
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
            .map_err(RepositoryError::FailedToUpdate)?;
        let amount_updated_rows = result.rows_affected();

        if amount_updated_rows != 1 {
            return Err(RepositoryError::ExpectedOneRow(amount_updated_rows));
        }

        Ok(())
    }
    #[allow(dead_code)]
    pub async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError> {
        let query = format!("DELETE FROM {} WHERE task_id=$1", self.table_basic.name);
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
