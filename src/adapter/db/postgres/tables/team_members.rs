use crate::adapter::db::RepositoryError;
use crate::adapter::db::models::TeamMember;
use crate::adapter::db::postgres::table_basic::TableBasic;
use sqlx::{Pool, Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Clone)]
pub struct TeamMembers {
    pool: Pool<Postgres>,
    table_basic: TableBasic,
}

#[allow(dead_code)]
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
        let items: Vec<TeamMember> = QueryBuilder::new(format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        ))
        .build_query_as()
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::FailedToQuery)?;

        Ok(items)
    }
    pub async fn one(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMember, RepositoryError> {
        let query = format!(
            "SELECT {} FROM {} WHERE team_id=$1 AND user_id=$2",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        let opt = QueryBuilder::new(query)
            .build_query_as()
            .bind(team_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::FailedToQuery)?;
        match opt {
            Some(v) => Ok(v),
            None => Err(RepositoryError::NotFoundRow),
        }
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
            .map_err(RepositoryError::FailedToInsert)?;
        Ok(())
    }
    pub async fn delete(&self, team_id: Uuid, user_id: Uuid) -> Result<(), RepositoryError> {
        let query = format!(
            "DELETE FROM {} WHERE team_id=$1 AND user_id=$2",
            self.table_basic.name
        );
        let result = QueryBuilder::new(query)
            .build()
            .bind(team_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::FailedToDelete)?;
        let amount_updated_rows = result.rows_affected();
        
        if amount_updated_rows != 1 {
            return Err(RepositoryError::ExpectedOneRow(amount_updated_rows));
        }

        Ok(())
    }
}
