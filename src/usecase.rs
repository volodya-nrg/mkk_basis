pub mod models;
pub mod tasks;
pub mod teams;

use crate::adapter::db::postgres::Postgres;
use tasks::Tasks;
use teams::Teams;

pub struct UseCase {
    pub tasks: Tasks,
    pub teams: Teams,
}

impl UseCase {
    pub fn new(postgres: Postgres) -> Self {
        Self {
            tasks: Tasks::new(postgres.tbl_tasks),
            teams: Teams::new(postgres.tbl_teams),
        }
    }
}
