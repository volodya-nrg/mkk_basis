use crate::adapter::db::postgres::teams::Teams as TeamsRepo;
use crate::usecase::models::Team;
use uuid::Uuid;

pub struct Teams {
    teams_repo: TeamsRepo,
}

impl Teams {
    pub fn new(teams_repo: TeamsRepo) -> Self {
        Self { teams_repo }
    }
    pub fn get_all(&self) -> Result<Vec<Team>, String> {
        let items = self
            .teams_repo
            .get_all()
            .map_err(|e| format!("failed to get items: {e}"))?;
        Ok(items.iter().map(|i| i.to_uc()).collect())
    }
    pub fn get_one(&self) -> Result<Team, String> {
        let item = self
            .teams_repo
            .get_one()
            .map_err(|e| format!("failed to get item: {e}"))?;
        Ok(item.to_uc())
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
