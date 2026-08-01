use sqlx::{Pool, Postgres};
use super::super::super::models::TaskComment;
use super::super::table_basic::TableBasic;
use uuid::Uuid;

#[derive(Debug)]
pub struct TaskComments<'p> {
    pool: &'p Pool<Postgres>,
    table_basic: TableBasic,
}
impl<'p> TaskComments<'p> {
    pub fn new(pool: &'p Pool<Postgres>) -> Self {
        Self {
            pool,
            table_basic: TableBasic {
                name: "task_comments".to_string(),
                fields: vec![
                    "task_comment_id".to_string(),
                    "task_id".to_string(),
                    "user_id".to_string(),
                    "msg".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                ],
            },
        }
    }
    pub fn list(&self, limit: i32, offset: i32) -> Result<(Vec<TaskComment>, u32), String> {
        let items = vec![];
        let total: u32 = 0;
        Ok((items, total))
    }
    pub fn one(&self, item_id: Uuid) -> Result<TaskComment, String> {
        Ok(TaskComment {
            task_comment_id: Default::default(),
            task_id: Default::default(),
            user_id: Default::default(),
            msg: "".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
        })
    }
    pub fn create(&self, item: TaskComment) -> Result<Uuid, String> {
        Ok(Uuid::new_v4())
    }
    pub fn update(&self, item: TaskComment) -> Result<(), String> {
        Ok(())
    }
    pub fn delete(&self, item_id: Uuid) -> Result<(), String> {
        Ok(())
    }
}
