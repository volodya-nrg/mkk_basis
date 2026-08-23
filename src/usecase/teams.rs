use super::{
    UseCaseError, mapper,
    models::{Team, TeamMember},
};
use crate::adapter::{
    db::RepositoryError,
    db::postgres::tables::{
        team_members::TeamMembers as TeamMembersRepo, teams::Teams as TeamsRepo,
    },
};
use crate::usecase::models::Task;
use http::StatusCode;
use uuid::Uuid;

#[derive(Clone)] // из-за axum-state
pub struct Teams {
    teams_repo: TeamsRepo,
    team_members_repo: TeamMembersRepo,
}

impl Teams {
    pub fn new(teams_repo: TeamsRepo, team_members_repo: TeamMembersRepo) -> Self {
        Self {
            teams_repo,
            team_members_repo,
        }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Team>, i64), UseCaseError> {
        let (items, total) = self
            .teams_repo
            .list(limit, offset)
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to get items: {e}")))?;
        Ok((
            items.into_iter().map(mapper::team_db_to_team_uc).collect(),
            total,
        ))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Team, UseCaseError> {
        let team_db = self.teams_repo.one(item_id).await.map_err(|e| match e {
            RepositoryError::NotFoundRow => UseCaseError::ForTransport {
                status_code: StatusCode::NOT_FOUND,
                public_err: "item not found".to_string(),
                internal_err: None,
            },
            other => UseCaseError::Common(other.to_string()),
        })?;
        Ok(mapper::team_db_to_team_uc(team_db))
    }
    pub async fn create(&self, team: Team) -> Result<Uuid, UseCaseError> {
        let new_uuid = self
            .teams_repo
            .create(mapper::team_uc_to_team_db(team))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create: {e}")))?;
        Ok(new_uuid)
    }
    pub async fn update(&self, team: Team) -> Result<(), UseCaseError> {
        self.teams_repo
            .update(mapper::team_uc_to_team_db(team))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to update: {e}")))?;
        Ok(())
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        self.teams_repo
            .delete(item_id)
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to delete: {e}")))?;
        Ok(())
    }
    // invite. Пригласить может только owner или admin.
    pub async fn invite(&self, team_id: Uuid, user_id: Uuid) -> Result<(), UseCaseError> {
        // TODO надо сделать авторизацию, чтоб можно было узнать кто приглашает
        self.team_members_repo
            .create(mapper::team_member_uc_to_team_member_db(TeamMember {
                team_id,
                user_id,
                created_at: Default::default(),
            }))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create: {e}")))?;
        Ok(())
    }
}
