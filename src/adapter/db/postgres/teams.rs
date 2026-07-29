use crate::adapter::db::models::Team;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct Teams {
    pool: Pool<Postgres>,
}

impl Teams {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub fn get_all(&self) -> Result<Vec<Team>, String> {
        println!("--- postgres teams get_all");
        Ok(vec![])
    }

    pub fn get_one(&self) -> Result<Team, String> {
        println!("--- postgres teams get_one");
        Ok(Team {
            team_id: Default::default(),
            name: "".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
        })
    }

    pub fn create(&self) -> Result<Uuid, String> {
        println!("--- postgres teams create");
        Ok(Uuid::new_v4())
    }

    pub fn update(&self) -> Result<(), String> {
        println!("--- postgres teams update");
        Ok(())
    }

    pub fn delete(&self) -> Result<(), String> {
        println!("--- postgres teams delete");
        Ok(())
    }
}
