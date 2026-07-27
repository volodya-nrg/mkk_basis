mod login;
mod register;
mod tasks;
mod teams;

use login::Login;
use register::Register;
use tasks::Tasks;
use teams::Teams;

pub struct Handlers {
    login: Login,
    register: Register,
    tasks: Tasks,
    teams: Teams,
}

impl Handlers {
    pub fn new() -> Self {
        Self {
            login: Login::new(),
            register: Register::new(),
            tasks: Tasks::new(),
            teams: Teams::new(),
        }
    }
}
