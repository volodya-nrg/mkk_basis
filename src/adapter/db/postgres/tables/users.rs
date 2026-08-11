use crate::adapter::db::RepositoryError;
use crate::adapter::db::models::User;
use crate::adapter::db::postgres::table_basic::TableBasic;
use sqlx::{Pool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct Users {
    pool: Pool<Postgres>,
    table_basic: TableBasic,
}
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
                    "email_is_confirmed".to_string(),
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
            .map_err(|e| RepositoryError::FailedToQuery(e))?;
        let total: (i64,) =
            QueryBuilder::new(format!("SELECT COUNT(*) FROM {}", self.table_basic.name)) // возвращает такой же диапазон как и i64
                .build_query_as()
                .fetch_one(&self.pool)
                .await
                .map_err(|e| RepositoryError::FailedToCount(e))?;

        Ok((items, total.0))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<User, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE user_id=$1",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        let opt = QueryBuilder::new(query)
            .build_query_as()
            .bind(item_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::FailedToQuery(e))?;
        match opt {
            Some(v) => Ok(v),
            None => Err(RepositoryError::NotFoundRow),
        }
    }
    pub async fn get_by_email(&self, email: String) -> Result<User, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE email=$1",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        let opt = QueryBuilder::new(query)
            .build_query_as()
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::FailedToQuery(e))?;
        match opt {
            Some(v) => Ok(v),
            None => Err(RepositoryError::NotFoundRow),
        }
    }
    pub async fn create(&self, item: User) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (name, email, password, email_is_confirmed) VALUES ($1,$2,$3,$4) RETURNING user_id",
            self.table_basic.name,
        );
        let result = QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.email)
            .bind(item.password)
            .bind(item.email_is_confirmed)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::FailedToInsert(e))?
            .get(0);

        Ok(result)
    }
    pub async fn update(&self, item: User) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET name=$1, email=$2, password=$3, email_is_confirmed=$4 WHERE user_id=$5",
            self.table_basic.name,
        );
        let result = QueryBuilder::new(query)
            .build()
            .bind(item.name)
            .bind(item.email)
            .bind(item.password)
            .bind(item.email_is_confirmed)
            .bind(item.user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::FailedToUpdate(e))?;
        let amount_updated_rows = result.rows_affected();

        if amount_updated_rows != 1 {
            return Err(RepositoryError::ExpectedOneRow(amount_updated_rows));
        }

        Ok(())
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError> {
        let query = format!("DELETE FROM {} WHERE user_id=$1", self.table_basic.name);
        let result = QueryBuilder::new(query)
            .build()
            .bind(item_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::FailedToDelete(e))?;
        let amount_updated_rows = result.rows_affected();

        if amount_updated_rows != 1 {
            return Err(RepositoryError::ExpectedOneRow(amount_updated_rows));
        }

        Ok(())
    }
}
