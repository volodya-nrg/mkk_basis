use sqlx::{Pool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

use crate::adapter::db::{RepositoryError, models::User, postgres::table_basic::TableBasic};

#[derive(Clone)]
pub struct Users {
    pool: Pool<Postgres>,
    table_basic: TableBasic,
}

#[allow(dead_code)]
impl Users {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            table_basic: TableBasic {
                name: "users".to_string(),
                fields: vec![
                    "user_id".to_string(),
                    "name".to_string(),
                    "email".to_string(),
                    "password".to_string(),
                    "email_code".to_string(),
                    "avatar".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                ],
            },
        }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<User>, i64), RepositoryError> {
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

        let items: Vec<User> = query
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
    pub async fn one(&self, item_id: Uuid) -> Result<User, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE user_id=$1",
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
    pub async fn by_email(&self, email: String) -> Result<User, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE email=$1",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build_query_as()
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::FailedToQuery)?
            .ok_or(RepositoryError::NotFoundRow)
    }
    pub async fn create(&self, item: User) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (name, email, password, email_code, avatar) VALUES ($1,$2,$3,$4,$5) RETURNING user_id",
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.email)
            .bind(item.password)
            .bind(item.email_code)
            .bind(item.avatar)
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .try_get(0)
            .map_err(|e| RepositoryError::Common(e))
    }
    pub async fn update(&self, item: User) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET name=$1, email=$2, password=$3, email_code=$4, avatar=$5 WHERE user_id=$6",
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.email)
            .bind(item.password)
            .bind(item.email_code)
            .bind(item.avatar)
            .bind(item.user_id)
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
    pub async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError> {
        let query = format!("DELETE FROM {} WHERE user_id=$1", self.table_basic.name);
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
