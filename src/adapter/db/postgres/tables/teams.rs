use super::super::super::models::Team;
use super::super::table_basic::TableBasic;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct Teams<'p> {
    pool: &'p Pool<Postgres>,
    table_basic: TableBasic,
}
impl<'p> Teams<'p> {
    pub fn new(pool: &'p Pool<Postgres>) -> Self {
        Self {
            pool,
            table_basic: TableBasic {
                name: "teams".to_string(),
                fields: vec![
                    "team_id".to_string(),
                    "name".to_string(),
                    "created_by".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                ],
            },
        }
    }
    pub fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Team>, u32), String> {
        let items = vec![];
        let total: u32 = 0;
        Ok((items, total))
    }
    pub fn one(&self, item_id: Uuid) -> Result<Team, String> {
        Ok(Team {
            team_id: Default::default(),
            name: "".to_string(),
            created_by: Default::default(),
            created_at: Default::default(),
            updated_at: Default::default(),
        })
    }
    pub fn create(&self, item: Team) -> Result<Uuid, String> {
        Ok(Uuid::new_v4())
    }
    pub fn update(&self, item: Team) -> Result<(), String> {
        Ok(())
    }
    pub fn delete(&self, item_id: Uuid) -> Result<(), String> {
        Ok(())
    }
}
