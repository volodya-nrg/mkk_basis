use uuid::Uuid;

trait TeamsRepository {
    fn get_all(&self) -> Result<Vec<Uuid>, String>;
    fn get_one(&self) -> Result<Uuid, String>;
    fn create(&self) -> Result<Uuid, String>;
    fn update(&self) -> Result<(), String>;
    fn delete(&self) -> Result<(), String>;
}

pub struct Teams<T: TeamsRepository> {
    teams_repo: T,
}

impl<T: TeamsRepository> Teams<T> {
    pub fn new(teams_repo: T) -> Self {
        Self { teams_repo }
    }
    pub fn get_all(&self) -> Result<Vec<Uuid>, String> {
        Ok(vec![Uuid::new_v4()])
    }
    pub fn get_one(&self) -> Result<Uuid, String> {
        Ok(Uuid::new_v4())
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
