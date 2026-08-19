use super::models::{
    RequestTask, RequestTeamCreate, RequestUser, ResponseTask, ResponseTaskHistory, ResponseTeam,
    ResponseUser,
};
use crate::usecase::models::{Task, TaskHistory, Team, User};
use uuid::Uuid;

pub fn team_uc_to_team_tr(item: Team) -> ResponseTeam {
    ResponseTeam {
        team_id: item.team_id,
        name: item.name,
        created_by: item.created_by,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
pub fn task_uc_to_task_tr(item: Task) -> ResponseTask {
    ResponseTask {
        task_id: item.task_id,
        name: item.name,
        description: item.description,
        created_by: item.created_by,
        team_id: item.team_id,
        assignee_id: item.assignee_id,
        status: item.status,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
pub fn team_tr_to_team_uc(req: RequestTeamCreate) -> Team {
    Team {
        team_id: Uuid::nil(),
        name: req.name,
        created_by: req.created_by,
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}
pub fn task_tr_to_task_uc(req: RequestTask) -> Task {
    Task {
        task_id: Uuid::nil(),
        name: req.name,
        description: req.description,
        created_by: req.created_by,
        team_id: req.team_id,
        assignee_id: req.assignee_id,
        status: req.status,
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}
pub fn task_history_uc_to_task_history_tr(item: TaskHistory) -> ResponseTaskHistory {
    ResponseTaskHistory {
        task_history_id: item.task_history_id,
        task_id: item.task_id,
        user_id: item.user_id,
        msg: item.msg,
        created_at: item.created_at,
    }
}

pub fn user_uc_to_user_tr(item: User) -> ResponseUser {
    ResponseUser {
        user_id: item.user_id,
        name: item.name,
        email: item.email,
        email_code: item.email_code,
        avatar: item.avatar,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

pub fn user_tr_to_user_uc(item: RequestUser) -> User {
    User {
        user_id: Uuid::nil(),
        name: item.name,
        email: item.email,
        password: item.password,
        email_code: item.email_code,
        avatar: None,
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}
