use crate::adapter::db::models::Team;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub struct Teams {
    pool: Pool<Postgres>,
}

impl Teams {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn get_all(&self) -> Result<Vec<Team>, String> {
        Ok(vec![])
    }

    pub fn get_one(&self) -> Result<Team, String> {
        Ok(Team {
            team_id: Default::default(),
            name: "".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
        })
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
