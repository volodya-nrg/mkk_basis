use sqlx::{Pool, Postgres};
use super::super::super::models::TaskHistory;
use super::super::table_basic::TableBasic;
use uuid::Uuid;

#[derive(Debug)]
pub struct TaskHistories<'p> {
    pool: &'p Pool<Postgres>,
    table_basic: TableBasic,
}
impl<'p> TaskHistories<'p> {
    pub fn new(pool: &'p Pool<Postgres>) -> Self {
        Self {
            pool,
            table_basic: TableBasic {
                name: "task_histories".to_string(),
                fields: vec![
                    "task_history_id".to_string(),
                    "task_id".to_string(),
                    "user_id".to_string(),
                    "msg".to_string(),
                    "created_at".to_string(),
                ],
            },
        }
    }
    pub fn list(&self, limit: i32, offset: i32) -> Result<(Vec<TaskHistory>, u32), String> {
        let items = vec![];
        let total: u32 = 0;
        Ok((items, total))
    }
    pub fn one(&self, item_id: Uuid) -> Result<TaskHistory, String> {
        Ok(TaskHistory {
            task_history_id: Default::default(),
            task_id: Default::default(),
            user_id: Default::default(),
            msg: "".to_string(),
            created_at: Default::default(),
        })
    }
    pub fn create(&self, item: TaskHistory) -> Result<Uuid, String> {
        Ok(Uuid::new_v4())
    }
    pub fn update(&self, item: TaskHistory) -> Result<(), String> {
        Ok(())
    }
    pub fn delete(&self, item_id: Uuid) -> Result<(), String> {
        Ok(())
    }
}
