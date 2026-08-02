use crate::adapter::db::errors::Error;
use crate::adapter::db::models::TeamMember;
use crate::adapter::db::postgres::table_basic::TableBasic;
use sqlx::{Pool, Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Debug)]
pub struct TeamMembers<'p> {
    pool: &'p Pool<Postgres>,
    table_basic: TableBasic,
}
impl<'p> TeamMembers<'p> {
    pub fn new(pool: &'p Pool<Postgres>) -> Self {
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
    pub async fn all(&self) -> Result<Vec<TeamMember>, String> {
        let items: Vec<TeamMember> = QueryBuilder::new(format!(
            "SELECT {} FROM {} ORDER BY created_at DESC",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        ))
        .build_query_as()
        .fetch_all(self.pool)
        .await
        .map_err(|e| format!("failed to query: {e}"))?;

        Ok(items)
    }
    pub async fn one(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMember, Error> {
        let query = format!(
            "SELECT {} FROM {} WHERE team_id=$1 AND user_id=$2",
            self.table_basic.fields.join(","),
            self.table_basic.name,
        );
        let opt = QueryBuilder::new(query)
            .build_query_as()
            .bind(team_id)
            .bind(user_id)
            .fetch_optional(self.pool)
            .await
            .map_err(|e| Error::Any(format!("failed to query: {e}")))?;
        match opt {
            Some(v) => Ok(v),
            None => Err(Error::NotFound),
        }
    }
    pub async fn create(&self, item: TeamMember) -> Result<(), String> {
        let query = format!(
            "INSERT INTO {} (team_id, user_id) VALUES ($1,$2)",
            self.table_basic.name,
        );

        QueryBuilder::new(query)
            .build()
            .bind(item.team_id)
            .bind(item.user_id)
            .execute(self.pool)
            .await
            .map_err(|e| format!("failed to insert: {e}"))?;
        Ok(())
    }
    pub async fn delete(&self, team_id: Uuid, user_id: Uuid) -> Result<(), String> {
        let query = format!(
            "DELETE FROM {} WHERE team_id=$1 AND user_id=$2",
            self.table_basic.name
        );
        let result = QueryBuilder::new(query)
            .build()
            .bind(team_id)
            .bind(user_id)
            .execute(self.pool)
            .await
            .map_err(|e| format!("failed to delete: {e}"))?;

        let amount_updated_rows = result.rows_affected();
        if amount_updated_rows != 1 {
            let err_msg = format!(
                "expected delete one row, but delete {}",
                amount_updated_rows
            )
            .to_string();
            return Err(err_msg);
        }

        Ok(())
    }
}
