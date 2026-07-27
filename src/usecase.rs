mod tasks;
mod teams;
pub mod models;

use tasks::Tasks;
use teams::Teams;

pub struct UseCase {
    tasks: Tasks,
    teams: Teams,
}

impl UseCase {
    pub fn new() -> Self {
        Self {
            tasks: Tasks::new(),
            teams: Teams::new(),
        }
    }
}
