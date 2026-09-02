use sqlx::{AssertSqlSafe, Pool, Postgres, QueryBuilder, Row};
use std::fmt::{self, Formatter};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    models::{Task, TaskData},
    postgres::table_basic::TableBasic,
    transactor::Transactor,
};

#[derive(Debug, EnumIter, PartialEq)]
pub enum Status {
    Start,
    Todo,
    Done,
    Cancelled,
}
impl Status {
    fn contains_value(v: String) -> bool {
        Status::iter().any(|s| s.to_string() == v)
    }
}
// можно поставить заклинание Display, но тогда будет начинаться с большой буквы, поэтому пишем сами как надо
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
    #[allow(dead_code)]
    transactor: Transactor,
    table_basic: TableBasic,
}
impl Tasks {
    pub fn new(pool: Pool<Postgres>, transactor: Transactor) -> Self {
        Self {
            pool,
            transactor,
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
    pub async fn list(&self, data: TaskData) -> Result<(Vec<Task>, i64), RepositoryError> {
        let mut query_common = format!(
            "SELECT {} FROM {}",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        let mut query_count = format!("SELECT COUNT(*) as count FROM {}", self.table_basic.name);
        let mut params: Vec<(String, String)> = vec![];

        if let Some(team_id) = data.team_id {
            params.push((
                format!("team_id=${}::uuid", params.len() + 1),
                team_id.to_string(),
            ));
        }
        if let Some(assignee_id) = data.assignee_id {
            params.push((
                format!("assignee_id=${}::uuid", params.len() + 1),
                assignee_id.to_string(),
            ));
        }
        if let Some(status) = data.status
            && Status::contains_value(status.clone())
        {
            params.push((
                format!("status=${}::task_status_enum", params.len() + 1),
                status,
            ));
        }
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

        let mut prepare_count = sqlx::query_scalar(AssertSqlSafe(query_count));
        for (_, v) in params.iter() {
            prepare_count = prepare_count.bind(v);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(RepositoryError::TransactionError)?;
        let count = prepare_count
            .fetch_one(&mut *tx)
            .await
            .map_err(RepositoryError::FailedToCount)?;

        if data.limit > -1 {
            query_common += format!(" LIMIT ${}::bigint", params.len() + 1).as_str();
            params.push(("".to_string(), data.limit.to_string()));
        }
        if data.offset > -1 {
            query_common += format!(" OFFSET ${}::bigint", params.len() + 1).as_str();
            params.push(("".to_string(), data.offset.to_string()));
        }

        let mut prepare_common = sqlx::query_as::<_, Task>(AssertSqlSafe(query_common));
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

        Ok((items, count))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Task, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE task_id=$1",
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
    pub async fn create(&self, item: Task) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (name, description, created_by, team_id, assignee_id, status) VALUES ($1,$2,$3,$4,$5,$6::task_status_enum) RETURNING task_id",
            self.table_basic.name,
        );
        QueryBuilder::new(query)
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
            .try_get(0)
            .map_err(RepositoryError::Common)
    }
    pub async fn update(&self, item: Task) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET name=$1, description=$2, created_by=$3, team_id=$4, assignee_id=$5, status=$6::task_status_enum WHERE task_id=$7",
            self.table_basic.name,
        );
        QueryBuilder::new(query)
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
    pub async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError> {
        let query = format!("DELETE FROM {} WHERE task_id=$1", self.table_basic.name);
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
