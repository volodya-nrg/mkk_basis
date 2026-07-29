use crate::adapter::db::postgres::tasks::Tasks as TasksRepo;
use crate::usecase::models::Task;
use uuid::Uuid;

#[derive(Clone)]
pub struct Tasks {
    tasks_repo: TasksRepo,
}

impl Tasks {
    pub fn new(tasks_repo: TasksRepo) -> Self {
        Self { tasks_repo }
    }
    pub fn get_all(&self) -> Result<Vec<Task>, String> {
        println!("--- usecase tasks get_all");
        let items = self
            .tasks_repo
            .get_all()
            .map_err(|e| format!("failed to get items: {e}"))?;
        Ok(items.iter().map(|i| i.to_uc()).collect())
    }
    pub fn get_one(&self) -> Result<Task, String> {
        println!("--- usecase tasks get_one");
        let item = self
            .tasks_repo
            .get_one()
            .map_err(|e| format!("failed to get item: {e}"))?;
        Ok(item.to_uc())
    }
    pub fn create(&self) -> Result<Uuid, String> {
        println!("--- usecase tasks create");
        Ok(Uuid::new_v4())
    }
    pub fn update(&self) -> Result<(), String> {
        println!("--- usecase tasks update");
        Ok(())
    }
    pub fn delete(&self) -> Result<(), String> {
        println!("--- usecase tasks delete");
        Ok(())
    }
}
