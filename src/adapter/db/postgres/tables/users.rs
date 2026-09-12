use sqlx::{QueryBuilder, Row};
use std::fmt;
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    models::{List, User},
    traits::NameAndFields,
};

pub enum Role {
    Admin,
    #[allow(dead_code)]
    Moder,
    Null, // при обновлении пользователя нужно иметь возможность выставить как-то в NULL
}
impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Role::Admin => "admin",
            Role::Moder => "moder",
            Role::Null => "null",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Default)]
pub struct Users {}
impl NameAndFields for Users {
    fn get_name(&self) -> &str {
        "users"
    }
    fn get_fields(&self) -> &[&str] {
        &[
            "user_id",
            "email",
            "password",
            "name",
            "email_code",
            "avatar",
            "role::text as role",
            "created_at",
            "updated_at",
        ]
    }
}
impl Users {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn list(
        &self,
        executor: &mut sqlx::PgConnection,
        limit: i32,
        offset: i32,
    ) -> Result<List<User>, RepositoryError> {
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

        let items: Vec<User> = common_builder
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
    pub async fn one(
        &self,
        executor: &mut sqlx::PgConnection,
        item_id: &Uuid,
    ) -> Result<User, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE user_id=$1",
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
    pub async fn by_email(
        &self,
        executor: &mut sqlx::PgConnection,
        email: &str, // чтение, но не владение
    ) -> Result<User, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE email=$1",
            self.get_fields().join(","),
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build_query_as()
            .bind(email)
            .fetch_optional(executor)
            .await
            .map_err(RepositoryError::FailedToQuery)?
            .ok_or(RepositoryError::NotFoundRow)
    }
    pub async fn create(
        &self,
        executor: &mut sqlx::PgConnection,
        item: User,
    ) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (email, password, name, email_code, avatar, role) VALUES ($1,$2,$3,$4,$5,$6::user_role_enum) RETURNING user_id",
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(&item.email)
            .bind(&item.password)
            .bind(&item.name)
            .bind(&item.email_code)
            .bind(&item.avatar)
            .bind(self.get_valid_role(&item.role))
            .fetch_one(executor)
            .await
            .map_err(RepositoryError::FailedToInsert)?
            .try_get(0)
            .map_err(RepositoryError::Common)
    }
    pub async fn update(
        &self,
        executor: &mut sqlx::PgConnection,
        item: User,
    ) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET email=$1, password=$2, name=$3, email_code=$4, avatar=$5, role=$6::user_role_enum WHERE user_id=$7",
            self.get_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(&item.email)
            .bind(&item.password)
            .bind(&item.name)
            .bind(&item.email_code)
            .bind(&item.avatar)
            .bind(self.get_valid_role(&item.role))
            .bind(item.user_id)
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
    pub async fn delete(
        &self,
        executor: &mut sqlx::PgConnection,
        item_id: &Uuid,
    ) -> Result<(), RepositoryError> {
        let query = format!("DELETE FROM {} WHERE user_id=$1", self.get_name());
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
    fn get_valid_role(&self, role: &Option<String>) -> Option<String> {
        let role_loc = role.clone();
        if let Some(v) = role
            && *v == Role::Null.to_string()
        {
            None
        } else {
            role_loc
        }
    }
}
