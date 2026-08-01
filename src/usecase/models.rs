// use crate::adapter::db::models as dbModels;
use chrono::Utc;
use uuid::Uuid;

pub struct Task {
    pub task_id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}
// impl Task {
//     pub fn to_db(&self) -> dbModels::Task {
//         dbModels::Task {
//             task_id: self.task_id,
//             name: self.name.to_string(),
//             created_at: self.created_at,
//             updated_at: self.updated_at,
//         }
//     }
// }

pub struct Team {
    pub team_id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}
// impl Team {
//     pub fn to_db(&self) -> dbModels::Team {
//         dbModels::Team {
//             team_id: self.team_id,
//             name: self.name.to_string(),
//             created_at: self.created_at,
//             updated_at: self.updated_at,
//         }
//     }
// }
