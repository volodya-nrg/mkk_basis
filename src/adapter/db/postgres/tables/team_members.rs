use sqlx::{Pool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::adapter::db::{RepositoryError, models::TeamMember, postgres::table_basic::TableBasic};

#[derive(Clone)]
pub struct TeamMembers {
    pool: Pool<Postgres>,
    table_basic: TableBasic,
}

impl TeamMembers {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            table_basic: TableBasic {
                name: "team_members".to_string(),
                fields: vec![
                    "team_id".to_string(),
                    "user_id".to_string(),
                    "created_at".to_string(),
                ],
            },
        }
    }
    pub async fn all(&self) -> Result<Vec<TeamMember>, RepositoryError> {
        Ok(QueryBuilder::new(format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        ))
        .build_query_as()
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::FailedToQuery)?)
    }
    pub async fn one(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMember, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE team_id=$1 AND user_id=$2",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build_query_as()
            .bind(team_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::FailedToQuery)?
            .ok_or(RepositoryError::NotFoundRow)
    }
    pub async fn create(&self, item: TeamMember) -> Result<(), RepositoryError> {
        let query = format!(
            "INSERT INTO {} (team_id, user_id) VALUES ($1,$2)",
            self.table_basic.name,
        );
        QueryBuilder::new(query)
            .build()
            .bind(item.team_id)
            .bind(item.user_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::FailedToInsert)
            .map(|_| ())
    }
    pub async fn delete(&self, team_id: Uuid, user_id: Uuid) -> Result<(), RepositoryError> {
        let query = format!(
            "DELETE FROM {} WHERE team_id=$1 AND user_id=$2",
            self.table_basic.name
        );
        QueryBuilder::new(query)
            .build()
            .bind(team_id)
            .bind(user_id)
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
