use async_trait::async_trait;
use sqlx::{QueryBuilder, Row};
use std::fmt;
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    internal::Table,
    models::{List, User},
    postgres::helpers::expect_one_row,
    storage::{AnyConnection, UsersTable},
};

pub enum Role {
    Admin,
    Null, // при обновлении пользователя нужно иметь возможность выставить как-то в NULL
}
impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Role::*;
        let s = match self {
            Admin => "admin",
            Null => "null",
        };
        write!(f, "{}", s)
    }
}

pub struct Users {}

impl Users {
    fn get_valid_role(&self, role: Option<String>) -> Option<String> {
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

impl Table for Users {
    fn get_table_name(&self) -> &str {
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

#[async_trait]
impl UsersTable for Users {
    async fn list(
        &self,
        conn: &mut AnyConnection<'_>,
        limit: i32,
        offset: i32,
    ) -> Result<List<User>, RepositoryError> {
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

        let items: Vec<User> = common_builder
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
    ) -> Result<User, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE user_id=$1",
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
    async fn by_email(
        &self,
        conn: &mut AnyConnection<'_>,
        email: &str,
    ) -> Result<User, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE email=$1",
            self.get_fields().join(","),
            self.get_table_name(),
        );
        QueryBuilder::new(query)
            .build_query_as()
            .bind(email)
            .fetch_optional(conn.as_postgres_mut()?)
            .await?
            .ok_or(RepositoryError::NotFoundRow)
    }
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: User,
    ) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (email, password, name, email_code, avatar, role) VALUES ($1,$2,$3,$4,$5,$6::user_role_enum) RETURNING user_id",
            self.get_table_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(&item.email)
            .bind(&item.password)
            .bind(&item.name)
            .bind(&item.email_code)
            .bind(&item.avatar)
            .bind(self.get_valid_role(item.role))
            .fetch_one(conn.as_postgres_mut()?)
            .await?
            .try_get(0)
            .map_err(|e| e.into())
    }
    async fn update(
        &self,
        conn: &mut AnyConnection<'_>,
        item: User,
    ) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET email=$1, password=$2, name=$3, email_code=$4, avatar=$5, role=$6::user_role_enum WHERE user_id=$7",
            self.get_table_name(),
        );
        let amount_rows = QueryBuilder::new(query)
            .build()
            .bind(&item.email)
            .bind(&item.password)
            .bind(&item.name)
            .bind(&item.email_code)
            .bind(&item.avatar)
            .bind(self.get_valid_role(item.role))
            .bind(item.user_id)
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
        let query = format!("DELETE FROM {} WHERE user_id=$1", self.get_table_name());
        let amount_rows = QueryBuilder::new(query)
            .build()
            .bind(item_id)
            .execute(conn.as_postgres_mut()?)
            .await?
            .rows_affected();

        expect_one_row(amount_rows)
    }
}
