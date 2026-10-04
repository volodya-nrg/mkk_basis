use async_trait::async_trait;
use sqlx::{AssertSqlSafe, QueryBuilder, Row};
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    internal::Table,
    models::{List, Task, TaskData},
    storage::{AnyConnection, TasksTable},
};

#[derive(Debug, EnumIter, PartialEq, Eq)]
pub enum Status {
    Start,
    Todo,
    Done,
    Cancelled,
}
impl Status {
    fn contains_value(v: String) -> bool {
        Self::iter().any(|s| s.to_string() == v)
    }
}
// если указать trait Display, тогда статусы будут в том же регистре как и написаны. Пишем по своему.
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Status::*;
        let s = match self {
            Start => "start",
            Todo => "todo",
            Done => "done",
            Cancelled => "cancelled",
        };
        write!(f, "{}", s)
    }
}

pub struct Tasks {}

impl Table for Tasks {
    fn get_name(&self) -> &str {
        "tasks"
    }
    fn get_fields(&self) -> &[&str] {
        &[
            "task_id",
            "name",
            "description",
            "created_by",
            "team_id",
            "assignee_id",
            "status::text as status",
            "created_at",
            "updated_at",
        ]
    }
}

#[async_trait]
impl TasksTable for Tasks {
    async fn list(
        &self,
        conn: &mut AnyConnection<'_>,
        data: TaskData,
    ) -> Result<List<Task>, RepositoryError> {
        let AnyConnection::Postgres(pg) = conn else {
            return Err(RepositoryError::WrongDatabase);
        };
        let mut query_common = format!(
            "SELECT {} FROM {}",
            self.get_fields().join(","),
            self.get_name(),
        );
        let mut query_count = format!("SELECT COUNT(*) as count FROM {}", self.get_name());
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
        if let Some(status) = &data.status
            && Status::contains_value(status.clone())
        {
            params.push((
                format!("status=${}::task_status_enum", params.len() + 1),
                status.clone(),
            ));
        }
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
        for (_, v) in params_copy {
            prepare_count = prepare_count.bind(v);
        }

        query_common.push_str(" ORDER BY created_at DESC");

        if data.limit > -1 {
            query_common += format!(" LIMIT ${}::bigint", params.len() + 1).as_str();
            params.push(("".to_string(), data.limit.to_string()));
        }
        if data.offset > -1 {
            query_common += format!(" OFFSET ${}::bigint", params.len() + 1).as_str();
            params.push(("".to_string(), data.offset.to_string()));
        }

        let mut prepare_common = sqlx::query_as::<_, Task>(AssertSqlSafe(query_common));
        for (_, v) in params {
            prepare_common = prepare_common.bind(v);
        }

        let items = prepare_common
            .fetch_all(&mut **pg)
            .await
            .map_err(RepositoryError::FailedToQuery)?;
        let total = prepare_count
            .fetch_one(&mut **pg)
            .await
            .map_err(RepositoryError::FailedToCount)?;

        Ok(List(items, total))
    }
    async fn one(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<Task, RepositoryError> {
        let AnyConnection::Postgres(pg) = conn else {
            return Err(RepositoryError::WrongDatabase);
        };
        let query = format!(
            "SELECT {} FROM {} WHERE task_id=$1",
            self.get_fields().join(","),
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build_query_as()
            .bind(item_id)
            .fetch_optional(&mut **pg)
            .await
            .map_err(RepositoryError::FailedToQuery)?
            .ok_or(RepositoryError::NotFoundRow)
    }
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: Task,
    ) -> Result<Uuid, RepositoryError> {
        let AnyConnection::Postgres(pg) = conn else {
            return Err(RepositoryError::WrongDatabase);
        };
        let query = format!(
            "INSERT INTO {} (name, description, created_by, team_id, assignee_id, status) VALUES ($1,$2,$3,$4,$5,$6::task_status_enum) RETURNING task_id",
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(&item.name)
            .bind(&item.description)
            .bind(item.created_by)
            .bind(item.team_id)
            .bind(item.assignee_id)
            .bind(&item.status)
            .fetch_one(&mut **pg)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .try_get(0)
            .map_err(RepositoryError::Common)
    }
    async fn update(
        &self,
        conn: &mut AnyConnection<'_>,
        item: Task,
    ) -> Result<(), RepositoryError> {
        let AnyConnection::Postgres(pg) = conn else {
            return Err(RepositoryError::WrongDatabase);
        };
        let query = format!(
            "UPDATE {} SET name=$1, description=$2, created_by=$3, team_id=$4, assignee_id=$5, status=$6::task_status_enum WHERE task_id=$7",
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(&item.name)
            .bind(&item.description)
            .bind(item.created_by)
            .bind(item.team_id)
            .bind(item.assignee_id)
            .bind(&item.status)
            .bind(item.task_id)
            .execute(&mut **pg)
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
    async fn delete(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<(), RepositoryError> {
        let AnyConnection::Postgres(pg) = conn else {
            return Err(RepositoryError::WrongDatabase);
        };
        let query = format!("DELETE FROM {} WHERE task_id=$1", self.get_name());
        QueryBuilder::new(query)
            .build()
            .bind(item_id)
            .execute(&mut **pg)
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
