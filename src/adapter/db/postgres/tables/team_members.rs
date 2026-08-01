use sqlx::{Pool, Postgres};
use super::super::super::models::TeamMember;
use super::super::table_basic::TableBasic;
use uuid::Uuid;

pub struct TeamsMembers<'p> {
    pool: &'p Pool<Postgres>,
    table_basic: TableBasic,
}
impl<'p> TeamsMembers<'p> {
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
    pub fn list(&self, limit: i32, offset: i32) -> Result<(Vec<TeamMember>, u32), String> {
        let items = vec![];
        let total: u32 = 0;
        Ok((items, total))
    }
    pub fn one(&self, item_id: Uuid) -> Result<TeamMember, String> {
        Ok(TeamMember {
            team_id: Default::default(),
            user_id: Default::default(),
            created_at: Default::default(),
        })
    }
    pub fn create(&self, item: TeamMember) -> Result<Uuid, String> {
        Ok(Uuid::new_v4())
    }
    pub fn update(&self, item: TeamMember) -> Result<(), String> {
        Ok(())
    }
    pub fn delete(&self, item_id: Uuid) -> Result<(), String> {
        Ok(())
    }
}
