use uuid::Uuid;

use crate::usecase::models::{
    Task as UCTask, TaskComment as UCTaskComment, TaskData, TaskHistory as UCTaskHistory,
    Team as UCTeam, User as UCUser, UserCreate, UserUpdate,
};

use super::models::{
    RequestTask, RequestTaskData, RequestTeam, RequestUserCreate, RequestUserUpdate, Task,
    TaskComment, TaskHistory, Team, User,
};

pub fn team_uc_to_team_tr(item: UCTeam) -> Team {
    Team {
        team_id: item.team_id.to_string(),
        name: item.name,
        created_by: item.created_by.to_string(),
        created_at: item.created_at.to_string(),
        updated_at: item.updated_at.to_string(),
    }
}

pub fn task_uc_to_task_tr(item: UCTask) -> Task {
    Task {
        task_id: item.task_id.to_string(),
        name: item.name,
        description: item.description,
        created_by: item.created_by.to_string(),
        team_id: item.team_id.to_string(),
        assignee_id: item.assignee_id.map(|v| v.to_string()),
        status: item.status,
        created_at: item.created_at.to_string(),
        updated_at: item.updated_at.to_string(),
    }
}

pub fn team_tr_to_team_uc(req: RequestTeam) -> UCTeam {
    UCTeam {
        team_id: Uuid::nil(),
        name: req.name,
        created_by: Uuid::nil(),
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}

pub fn task_tr_to_task_uc(req: RequestTask) -> Result<UCTask, uuid::Error> {
    let created_by_parsed = Uuid::parse_str(req.created_by.as_str())?;
    let team_id_parsed = Uuid::parse_str(req.team_id.as_str())?;
    let assignee_id_parsed: Option<Uuid> = req
        .assignee_id
        .map(|v| Uuid::parse_str(v.as_str()))
        .transpose()?;

    Ok(UCTask {
        task_id: Uuid::nil(),
        name: req.name,
        description: req.description,
        created_by: created_by_parsed,
        team_id: team_id_parsed,
        assignee_id: assignee_id_parsed,
        status: req.status,
        created_at: Default::default(),
        updated_at: Default::default(),
    })
}

pub fn task_data_tr_to_task_data_uc(req: RequestTaskData) -> Result<TaskData, uuid::Error> {
    let team_id_parsed: Option<Uuid> = req
        .team_id
        .map(|v| Uuid::parse_str(v.as_str()))
        .transpose()?;
    let assignee_id_parsed: Option<Uuid> = req
        .assignee_id
        .map(|v| Uuid::parse_str(v.as_str()))
        .transpose()?;

    Ok(TaskData {
        limit: req.limit,
        offset: req.offset,
        team_id: team_id_parsed,
        assignee_id: assignee_id_parsed,
        status: req.status,
    })
}

pub fn task_history_uc_to_task_history_tr(item: UCTaskHistory) -> TaskHistory {
    TaskHistory {
        task_history_id: item.task_history_id.to_string(),
        task_id: item.task_id.to_string(),
        user_id: item.user_id.to_string(),
        msg: item.msg,
        created_at: item.created_at.to_string(),
    }
}

pub fn user_uc_to_user_tr(item: UCUser) -> User {
    User {
        user_id: item.user_id.to_string(),
        name: item.name,
        email: item.email,
        avatar: item.avatar,
        role: item.role,
        created_at: item.created_at.to_string(),
        updated_at: item.updated_at.to_string(),
    }
}

pub fn user_create_tr_to_user_create_uc(item: RequestUserCreate) -> UserCreate {
    UserCreate {
        email: item.email,
        password: item.password,
        name: item.name.map(|v| v.to_string()),
        email_code: None,
        role: item.role.map(|v| v.to_string()),
        avatar: item.avatar.map(|v| v.to_string()),
    }
}

pub fn user_tr_update_to_user_uc_update(item: RequestUserUpdate) -> UserUpdate {
    UserUpdate {
        user_id: Default::default(),
        email: item.email.map(|v| v.to_string()),
        password: item.password.map(|v| v.to_string()),
        name: item.name.map(|v| v.to_string()),
        email_code: None,
        role: item.role.map(|v| v.to_string()),
        avatar: item.avatar.map(|v| v.to_string()),
        is_remove_avatar: item.is_remove_avatar,
    }
}

pub fn task_comment_tr_to_task_comment_uc(
    msg: String,
    task_id: Uuid,
    user_id: Uuid,
) -> UCTaskComment {
    UCTaskComment {
        task_comment_id: Default::default(),
        task_id,
        user_id,
        msg,
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}

pub fn task_comment_uc_to_task_comment_tr(item: UCTaskComment) -> TaskComment {
    TaskComment {
        task_comment_id: item.task_comment_id.to_string(),
        task_id: item.task_id.to_string(),
        user_id: item.user_id.to_string(),
        msg: item.msg,
        created_at: item.created_at.to_string(),
        updated_at: item.updated_at.to_string(),
    }
}
