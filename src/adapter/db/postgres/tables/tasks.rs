use sqlx::{Pool, Postgres};
use super::super::super::models::Task;
use super::super::table_basic::TableBasic;
use uuid::Uuid;

#[derive(Debug)]
pub struct Tasks<'p> {
    pool: &'p Pool<Postgres>,
    table_basic: TableBasic,
}
impl<'p> Tasks<'p> {
    pub fn new(pool: &'p Pool<Postgres>) -> Self {
        Self {
            pool,
            table_basic: TableBasic {
                name: "tasks".to_string(),
                fields: vec![
                    "task_id".to_string(),
                    "name".to_string(),
                    "description".to_string(),
                    "created_by".to_string(),
                    "team_id".to_string(),
                    "assignee_id".to_string(),
                    "status".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                ],
            },
        }
    }
    pub fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Task>, u32), String> {
        let items = vec![];
        let total: u32 = 0;
        Ok((items, total))
    }
    pub fn one(&self, item_id: Uuid) -> Result<Task, String> {
        Ok(Task {
            task_id: Default::default(),
            name: "".to_string(),
            description: None,
            created_by: Default::default(),
            team_id: Default::default(),
            assignee_id: None,
            status: "".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
        })
    }
    pub fn create(&self, item: Task) -> Result<Uuid, String> {
        Ok(Uuid::new_v4())
    }
    pub fn update(&self, item: Task) -> Result<(), String> {
        Ok(())
    }
    pub fn delete(&self, item_id: Uuid) -> Result<(), String> {
        Ok(())
    }
}
