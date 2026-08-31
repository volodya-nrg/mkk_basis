use uuid::Uuid;

use crate::usecase::models::{
    Task, TaskComment, TaskLimitOffsetFilter, TaskHistory, Team, User, UserCreate, UserUpdate,
};

use super::models::{
    RequestTask, RequestTaskLimitOffsetFilter, RequestTeam, RequestUserCreate, RequestUserUpdate,
    ResponseTask, ResponseTaskComment, ResponseTaskHistory, ResponseTeam, ResponseUser,
};

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

pub fn team_tr_to_team_uc(req: RequestTeam) -> Team {
    Team {
        team_id: Uuid::nil(),
        name: req.name,
        created_by: Uuid::nil(),
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

pub fn task_limit_offset_filter_tr_to_task_limit_offset_filter_uc(req: RequestTaskLimitOffsetFilter) -> TaskLimitOffsetFilter {
    TaskLimitOffsetFilter {
        limit: req.limit,
        offset: req.offset,
        team_id: req.team_id,
        assignee_id: req.assignee_id,
        status: req.status,
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
        avatar: item.avatar,
        role: item.role,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

pub fn user_create_tr_to_user_create_uc(item: RequestUserCreate) -> UserCreate {
    UserCreate {
        email: item.email,
        password: item.password,
        name: item.name,
        email_code: None,
        role: item.role,
        avatar: item.avatar,
    }
}

pub fn user_tr_update_to_user_uc_update(item: RequestUserUpdate) -> UserUpdate {
    UserUpdate {
        user_id: Default::default(),
        email: item.email,
        password: item.password,
        name: item.name,
        email_code: None,
        role: item.role,
        avatar: item.avatar,
        is_remove_avatar: item.is_remove_avatar,
    }
}

pub fn task_comment_tr_to_task_comment_uc(
    msg: String,
    task_id: Uuid,
    user_id: Uuid,
) -> TaskComment {
    TaskComment {
        task_comment_id: Default::default(),
        task_id,
        user_id,
        msg,
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}

pub fn task_comment_uc_to_task_comment_tr(item: TaskComment) -> ResponseTaskComment {
    ResponseTaskComment {
        task_comment_id: item.task_comment_id,
        task_id: item.task_id,
        user_id: item.user_id,
        msg: item.msg,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
