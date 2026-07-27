mod tasks;
mod users;

use tasks::Tasks;
use users::Users;

pub struct Postgres {
    tbl_users: Users,
    tbl_tasks: Tasks,
}

impl Postgres {
    pub fn new(dsn: &String) -> Self {
        
        
        Self {
            tbl_users: Users::new(),
            tbl_tasks: Tasks::new(),
        }
    }
}
