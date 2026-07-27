use crate::adapter::db::models as dbModels;
use crate::usecase::models::Task;
use uuid::Uuid;

pub struct Tasks<T: TasksRepository> {
    tasks_repo: T,
}

impl<T: TasksRepository> Tasks<T> {
    pub fn new(tasks_repo: T) -> Self {
        Self { tasks_repo }
    }
    pub fn get_all(&self) -> Result<Vec<Task>, String> {
        match self.tasks_repo.get_all() {
            Ok(items) => Ok(items.iter().map(|i| i.to_uc()).collect()),
            Err(e) => Err(format!("failed to get items from db: {e}")),
        }
    }
    pub fn get_one(&self) -> Result<Task, String> {
        let item = match self.tasks_repo.get_one() {
            Ok(item) => item.to_uc(),
            Err(e) => return Err(format!("failed to get item from db: {e}")),
        };
        Ok(item)
    }
    pub fn create(&self) -> Result<Uuid, String> {
        Ok(Uuid::new_v4())
    }
    pub fn update(&self) -> Result<(), String> {
        Ok(())
    }
    pub fn delete(&self) -> Result<(), String> {
        Ok(())
    }
}

pub trait TasksRepository {
    fn get_all(&self) -> Result<Vec<dbModels::Task>, String>;
    fn get_one(&self) -> Result<dbModels::Task, String>;
    fn create(&self) -> Result<Uuid, String>;
    fn update(&self) -> Result<(), String>;
    fn delete(&self) -> Result<(), String>;
}
