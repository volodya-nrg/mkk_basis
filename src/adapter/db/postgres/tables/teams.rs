use async_trait::async_trait;
use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    internal::Table,
    models::{List, Team},
    postgres::helpers::expect_one_row,
    storage::{AnyConnection, TeamsTable},
};

pub struct Teams {}

impl Table for Teams {
    fn get_table_name(&self) -> &str {
        "teams"
    }
    fn get_fields(&self) -> &[&str] {
        &["team_id", "name", "created_by", "created_at", "updated_at"]
    }
}

#[async_trait]
impl TeamsTable for Teams {
    async fn list(
        &self,
        conn: &mut AnyConnection<'_>,
        limit: i32,
        offset: i32,
    ) -> Result<List<Team>, RepositoryError> {
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

        let items: Vec<Team> = common_builder
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
    ) -> Result<Team, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE team_id=$1",
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
        item: Team,
    ) -> Result<Uuid, RepositoryError> {
        let query = format!(
            "INSERT INTO {} (name, created_by) VALUES ($1,$2) RETURNING team_id",
            self.get_table_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(&item.name)
            .bind(item.created_by)
            .fetch_one(conn.as_postgres_mut()?)
            .await?
            .try_get(0)
            .map_err(|e| e.into())
    }
    async fn update(
        &self,
        conn: &mut AnyConnection<'_>,
        item: Team,
    ) -> Result<(), RepositoryError> {
        let query = format!(
            "UPDATE {} SET name=$1 WHERE team_id=$2", // создателя не меняем
            self.get_table_name(),
        );
        let amount_rows = QueryBuilder::new(query)
            .build()
            .bind(&item.name)
            .bind(item.team_id)
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
        let query = format!("DELETE FROM {} WHERE team_id=$1", self.get_table_name());
        let amount_rows = QueryBuilder::new(query)
            .build()
            .bind(item_id)
            .execute(conn.as_postgres_mut()?)
            .await?
            .rows_affected();

        expect_one_row(amount_rows)
    }
}
