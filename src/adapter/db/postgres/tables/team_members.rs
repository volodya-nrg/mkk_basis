use async_trait::async_trait;
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    internal::Table,
    models::TeamMember,
    storage::{AnyConnection, TeamMembersTable},
};

pub struct TeamMembers {}

impl Table for TeamMembers {
    fn get_table_name(&self) -> &str {
        "team_members"
    }
    fn get_fields(&self) -> &[&str] {
        &["team_id", "user_id", "created_at"]
    }
}

#[async_trait]
impl TeamMembersTable for TeamMembers {
    async fn all(&self, conn: &mut AnyConnection<'_>) -> Result<Vec<TeamMember>, RepositoryError> {
        let AnyConnection::Postgres(pg) = conn else {
            return Err(RepositoryError::WrongDatabase);
        };
        QueryBuilder::new(format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.get_fields().join(","),
            self.get_table_name(),
        ))
        .build_query_as()
        .fetch_all(&mut **pg)
        .await
        .map_err(RepositoryError::FailedToQuery)
    }
    async fn one(
        &self,
        conn: &mut AnyConnection<'_>,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<TeamMember, RepositoryError> {
        let AnyConnection::Postgres(pg) = conn else {
            return Err(RepositoryError::WrongDatabase);
        };
        let query = format!(
            "SELECT {} FROM {} WHERE team_id=$1 AND user_id=$2",
            self.get_fields().join(","),
            self.get_table_name(),
        );
        QueryBuilder::new(query)
            .build_query_as()
            .bind(team_id)
            .bind(user_id)
            .fetch_optional(&mut **pg)
            .await
            .map_err(RepositoryError::FailedToQuery)?
            .ok_or(RepositoryError::NotFoundRow)
    }
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: TeamMember,
    ) -> Result<(), RepositoryError> {
        let AnyConnection::Postgres(pg) = conn else {
            return Err(RepositoryError::WrongDatabase);
        };
        let query = format!(
            "INSERT INTO {} (team_id, user_id) VALUES ($1,$2)",
            self.get_table_name(),
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.team_id)
            .bind(item.user_id)
            .execute(&mut **pg)
            .await
            .map_err(RepositoryError::FailedToInsert)
            .map(|_| ())
    }
    async fn delete(
        &self,
        conn: &mut AnyConnection<'_>,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), RepositoryError> {
        let AnyConnection::Postgres(pg) = conn else {
            return Err(RepositoryError::WrongDatabase);
        };
        let query = format!(
            "DELETE FROM {} WHERE team_id=$1 AND user_id=$2",
            self.get_table_name()
        );
        QueryBuilder::new(query)
            .build()
            .bind(team_id)
            .bind(user_id)
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
