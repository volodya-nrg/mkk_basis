use sqlx::{Pool, Postgres, QueryBuilder, Row};
use std::fmt;
use std::fmt::Formatter;
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError, models::User, postgres::table_basic::TableBasic,
    transactor::Transactor,
};

pub enum Role {
    Admin,
    #[allow(dead_code)]
    Moder,
    Null, // при обновлении пользователя нужно иметь возможность выставить как-то в NULL
}
impl fmt::Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Role::Admin => "admin",
            Role::Moder => "moder",
            Role::Null => "null",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct Users {
    pool: Pool<Postgres>,
    #[allow(dead_code)]
    transactor: Transactor,
    table_basic: TableBasic,
}

impl Users {
    pub fn new(pool: Pool<Postgres>, transactor: Transactor) -> Self {
        Self {
            pool,
            transactor,
            table_basic: TableBasic {
                name: "users".to_string(),
                fields: vec![
                    "user_id".to_string(),
                    "email".to_string(),
                    "password".to_string(),
                    "name".to_string(),
                    "email_code".to_string(),
                    "avatar".to_string(),
                    "role::text as role".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                ],
            },
        }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<User>, i64), RepositoryError> {
        let mut common_builder = QueryBuilder::new(format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        ));
        let mut count_builder =
            QueryBuilder::new(format!("SELECT COUNT(*) FROM {}", self.table_basic.name));

        if limit > -1 {
            common_builder.push(" LIMIT ");
            common_builder.push_bind(limit);
        }
        if offset > -1 {
            common_builder.push(" OFFSET ");
            common_builder.push_bind(offset);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(RepositoryError::TransactionError)?;
        let items: Vec<User> = common_builder
            .build_query_as()
            .fetch_all(&mut *tx)
            .await
            .map_err(RepositoryError::FailedToQuery)?;
        let total = count_builder
            .build_query_scalar()
            .fetch_one(&mut *tx)
            .await
            .map_err(RepositoryError::FailedToCount)?;
        tx.commit()
            .await
            .map_err(RepositoryError::TransactionError)?;

        Ok((items, total))
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
            "INSERT INTO {} (email, password, name, email_code, avatar, role) VALUES ($1,$2,$3,$4,$5,$6::user_role_enum) RETURNING user_id",
            self.table_basic.name,
        );

        QueryBuilder::new(query)
            .build()
            .bind(item.email)
            .bind(item.password)
            .bind(item.name)
            .bind(item.email_code)
            .bind(item.avatar)
            .bind(self.get_valid_role(item.role))
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .try_get(0)
            .map_err(RepositoryError::Common)
    }
    pub async fn update(&self, item: User) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET email=$1, password=$2, name=$3, email_code=$4, avatar=$5, role=$6::user_role_enum WHERE user_id=$7",
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.email)
            .bind(item.password)
            .bind(item.name)
            .bind(item.email_code)
            .bind(item.avatar)
            .bind(self.get_valid_role(item.role))
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
    fn get_valid_role(&self, role: Option<String>) -> Option<String> {
        let role_loc = role.clone();
        if let Some(v) = role
            && v == Role::Null.to_string()
        {
            None
        } else {
            role_loc
        }
    }
}
