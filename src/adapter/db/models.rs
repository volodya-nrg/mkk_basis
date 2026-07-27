use crate::usecase::models as ucModels;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Task {
    pub task_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl Task {
    pub fn to_uc(&self) -> ucModels::Task {
        ucModels::Task {
            task_id: self.task_id,
            name: self.name.to_string(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

pub struct Team {
    pub team_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl Team {
    pub fn to_uc(&self) -> ucModels::Team {
        ucModels::Team {
            team_id: self.team_id,
            name: self.name.to_string(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
