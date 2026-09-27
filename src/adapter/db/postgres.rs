pub mod tables;
pub mod transactor;

use crate::adapter::db::postgres::transactor::Transactor;
use std::sync::Arc;
use tables::{
    task_comments::TaskComments, task_histories::TaskHistories, tasks::Tasks,
    team_members::TeamMembers, teams::Teams, users::Users,
};

// Структура Postgres - аккамулятор для таблиц, для удобного вызова конкретной таблицы.
// Таблицы могут использоваться несколько раз в том или ином бизнес-слое, поэтому на него должен быть
// указатель. Роутер требует многопоточность, поэтому применяется Arc.
// Transactor содержится внутри только для удобства.
pub struct Postgres {
    pub transactor: Arc<Transactor>,
    pub tbl_users: Arc<Users>,
    pub tbl_teams: Arc<Teams>,
    pub tbl_team_members: Arc<TeamMembers>,
    pub tbl_tasks: Arc<Tasks>,
    pub tbl_task_histories: Arc<TaskHistories>,
    pub tbl_task_comments: Arc<TaskComments>,
}

impl Postgres {
    // Transactor используется в нескольких местах: в тесте и внутри безнес-логики. Поэтому тоже Arc.
    pub fn new(transactor: Arc<Transactor>) -> Self {
        Self {
            transactor,
            tbl_users: Arc::new(Users::new()),
            tbl_teams: Arc::new(Teams::new()),
            tbl_team_members: Arc::new(TeamMembers::new()),
            tbl_tasks: Arc::new(Tasks::new()),
            tbl_task_histories: Arc::new(TaskHistories::new()),
            tbl_task_comments: Arc::new(TaskComments::new()),
        }
    }
}
