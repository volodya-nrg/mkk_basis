use async_trait::async_trait;
use sqlx::{AssertSqlSafe, QueryBuilder, Row};
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    internal::Table,
    models::{List, TaskComment},
    postgres::helpers::expect_one_row,
    storage::{AnyConnection, TaskCommentsTable},
};

/*
Чтобы from не делать, можно короче написать с помощью thiserror.
Эта аннотация говорит thiserror, что нужно сгенерировать соответствующую реализацию трэйта From.
#[derive(Debug, Error)]
enum PurchaseError {
    #[error("Nested servation error: (0)")]
    ReservationFailed(#[from] ReserveError)
    #[error("Nested shipping error: (0)")]
    ShippingFailed(#[from] ShipmentError)
}
*/
/*
    пример как два раза получить мутабильную ссылку
    fn main() {
        let mut a = 5;
        unsafe {
            let r1: &mut i32 = &mut a; // первая мутабельная ссылка
            let ptr: *mut i32 = r1 as *mut i32; // мутабельный указатель
            let r2: &mut i32 = ptr.as_mut().unwrap(); // указатель во вторую ссылку
            inc(r1);
            inc(r2);
        }
        println!("{a}"); // 7
    }

    fn inc(a: &mut i32) {
        *a = *a + 1;
    }
*/

pub struct TaskComments {}

impl Table for TaskComments {
    fn get_table_name(&self) -> &str {
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

#[async_trait]
impl TaskCommentsTable for TaskComments {
    async fn list(
        &self,
        conn: &mut AnyConnection<'_>,
        task_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<List<TaskComment>, RepositoryError> {
        // example
        // let AnyConnection::Postgres(pg) = conn else {
        //     return Err(RepositoryError::WrongDatabase);
        // };
        let pg = conn.as_postgres_mut()?;
        let mut query_common = format!(
            "SELECT {} FROM {}",
            self.get_fields().join(","),
            self.get_table_name()
        );
        let mut query_count = format!("SELECT COUNT(*) as count FROM {}", self.get_table_name());
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
        for (_, v) in params_copy {
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
        for (_, v) in params {
            prepare_common = prepare_common.bind(v);
        }

        // &mut **pg
        let items = prepare_common.fetch_all(pg.as_mut()).await?;
        let total = prepare_count.fetch_one(pg.as_mut()).await?;

        Ok(List(items, total))
    }
    async fn one(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<TaskComment, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE task_comment_id=$1",
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
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: TaskComment,
    ) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (task_id, user_id, msg) VALUES ($1,$2,$3) RETURNING task_comment_id",
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
        item: TaskComment,
    ) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET task_id=$1, user_id=$2, msg=$3 WHERE task_comment_id=$4",
            self.get_table_name(),
        );
        // and_then - as map() and flatten()
        let amount_rows = QueryBuilder::new(query)
            .build()
            .bind(item.task_id)
            .bind(item.user_id)
            .bind(&item.msg)
            .bind(item.task_comment_id)
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
            "DELETE FROM {} WHERE task_comment_id=$1",
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
