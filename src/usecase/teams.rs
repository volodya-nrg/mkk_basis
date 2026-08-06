use crate::adapter::db::postgres::tables::{
    team_members::TeamMembers as TeamMembersRepo, teams::Teams as TeamsRepo,
};
use crate::usecase::{mapper, models::*};
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
    pub async fn get_list(&self, limit: i32, offset: i32) -> Result<(Vec<Team>, i64), String> {
        let (items, total) = self
            .teams_repo
            .list(limit, offset)
            .await
            .map_err(|e| format!("failed to get items: {e}"))?;
        Ok((
            items
                .into_iter()
                .map(|item| mapper::team_db_to_team_uc(item))
                .collect(),
            total,
        ))
    }
    pub async fn create(&self, team: Team) -> Result<Uuid, String> {
        let new_uuid = self
            .teams_repo
            .create(mapper::team_uc_to_team_db(team))
            .await
            .map_err(|e| format!("failed to create: {e}"))?;
        Ok(new_uuid)
    }
    pub async fn invite(&self, team_id: Uuid, user_id: Uuid) -> Result<(), String> {
        self.team_members_repo
            .create(mapper::team_member_uc_to_team_member_db(TeamMember {
                team_id,
                user_id,
                created_at: Default::default(),
            }))
            .await
            .map_err(|e| format!("failed to create: {e}"))?;
        Ok(())
    }
}
