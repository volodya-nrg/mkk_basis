use crate::adapter::db::models::{
    Task as DBTask, TaskComment as DBTaskComment, TaskHistory as DBTaskHistory, Team as DBTeam,
    TeamMember as DBTeamMember, User as DBUser,
};

use super::models::{Task, TaskComment, TaskHistory, Team, TeamMember, User};

pub fn task_db_to_task_uc(item: DBTask) -> Task {
    Task {
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

pub fn task_uc_to_task_db(item: Task) -> DBTask {
    DBTask {
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

pub fn task_history_db_to_task_history_uc(item: DBTaskHistory) -> TaskHistory {
    TaskHistory {
        task_history_id: item.task_history_id,
        task_id: item.task_id,
        user_id: item.user_id,
        msg: item.msg,
        created_at: item.created_at,
    }
}

pub fn task_history_uc_to_task_history_db(item: TaskHistory) -> DBTaskHistory {
    DBTaskHistory {
        task_history_id: item.task_history_id,
        task_id: item.task_id,
        user_id: item.user_id,
        msg: item.msg,
        created_at: item.created_at,
    }
}

pub fn team_db_to_team_uc(item: DBTeam) -> Team {
    Team {
        team_id: item.team_id,
        name: item.name,
        created_by: item.created_by,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

pub fn team_uc_to_team_db(item: Team) -> DBTeam {
    DBTeam {
        team_id: item.team_id,
        name: item.name,
        created_by: item.created_by,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

pub fn team_member_uc_to_team_member_db(item: TeamMember) -> DBTeamMember {
    DBTeamMember {
        team_id: item.team_id,
        user_id: item.user_id,
        created_at: item.created_at,
    }
}

pub fn user_db_to_user_uc(item: DBUser) -> User {
    User {
        user_id: item.user_id,
        name: item.name,
        email: item.email,
        password: item.password,
        email_code: item.email_code,
        avatar: item.avatar,
        role: item.role,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

pub fn task_comment_db_to_task_comment_uc(item: DBTaskComment) -> TaskComment {
    TaskComment {
        task_comment_id: item.task_comment_id,
        task_id: item.task_id,
        user_id: item.user_id,
        msg: item.msg,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

pub fn task_comment_uc_to_task_comment_db(item: TaskComment) -> DBTaskComment {
    DBTaskComment {
        task_comment_id: item.task_comment_id,
        task_id: item.task_id,
        user_id: item.user_id,
        msg: item.msg,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
