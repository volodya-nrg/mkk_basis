use crate::adapter::db::models::Task;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct Tasks {
    pool: Pool<Postgres>,
}

impl Tasks {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn get_all(&self) -> Result<Vec<Task>, String> {
        println!("--- postgres tasks get_all");
        Ok(vec![])
    }

    pub fn get_one(&self) -> Result<Task, String> {
        println!("--- postgres tasks get_one");
        Ok(Task {
            task_id: Default::default(),
            name: "".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
        })
    }

    pub fn create(&self) -> Result<Uuid, String> {
        println!("--- postgres tasks create");
        Ok(Uuid::new_v4())
    }

    pub fn update(&self) -> Result<(), String> {
        println!("--- postgres tasks update");
        Ok(())
    }

    pub fn delete(&self) -> Result<(), String> {
        println!("--- postgres tasks delete");
        Ok(())
    }
}
