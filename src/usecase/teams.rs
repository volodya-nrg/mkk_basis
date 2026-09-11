use http::StatusCode;
use uuid::Uuid;

use crate::{
    adapter::db::postgres::{
        tables::{
            team_members::TeamMembers as DBTeamMembers, teams::Teams as DBTeams,
            users::Role as UserRole,
        },
        transactor::{TransactionError, Transactor},
    },
    err_msg::ErrMsg,
};

use super::{
    UseCaseError, mapper,
    models::{Team, TeamMember},
};

#[derive(Clone)] // из-за axum-state
pub struct Teams {
    transactor: Transactor,
    teams_repo: DBTeams,
    team_members_repo: DBTeamMembers,
}

impl Teams {
    pub fn new(
        transactor: Transactor,
        teams_repo: DBTeams,
        team_members_repo: DBTeamMembers,
    ) -> Self {
        Self {
            transactor,
            teams_repo,
            team_members_repo,
        }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Team>, i64), UseCaseError> {
        self.transactor
            .in_transaction(async |tx| {
                self.teams_repo
                    .list(tx, limit, offset)
                    .await
                    .map_err(|e| UseCaseError::Common(format!("failed to get items: {e}")))
                    .map(|list| {
                        (
                            list.0.into_iter().map(mapper::team_db_to_team_uc).collect(),
                            list.1,
                        )
                    })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Database(sqlx_err) => UseCaseError::Common(sqlx_err.to_string()),
                TransactionError::Operation(use_case_err) => use_case_err,
            })
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Team, UseCaseError> {
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        Ok(mapper::team_db_to_team_uc(
            self.teams_repo.one(&mut db_conn, item_id).await?,
        ))
    }
    pub async fn create(&self, team: Team) -> Result<Uuid, UseCaseError> {
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        Ok(self
            .teams_repo
            .create(&mut db_conn, mapper::team_uc_to_team_db(team))
            .await?)
    }
    pub async fn update(&self, team: Team) -> Result<(), UseCaseError> {
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        Ok(self
            .teams_repo
            .update(&mut db_conn, mapper::team_uc_to_team_db(team))
            .await?)
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        Ok(self.teams_repo.delete(&mut db_conn, item_id).await?)
    }
    // пригласить может только owner или admin
    pub async fn invite(
        &self,
        profile_id: Uuid,
        profile_role: Option<String>,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), UseCaseError> {
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;

        let mut is_has_access = false;

        if let Some(role) = profile_role
            && role == UserRole::Admin.to_string()
        {
            is_has_access = true;
        } else {
            let team = self.teams_repo.one(&mut db_conn, team_id).await?;
            if team.created_by == profile_id {
                is_has_access = true;
            }
        }

        if !is_has_access {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::FORBIDDEN,
                public_err: ErrMsg::NoRules.to_string(),
                internal_err: None,
            });
        }

        Ok(self
            .team_members_repo
            .create(
                &mut db_conn,
                mapper::team_member_uc_to_team_member_db(TeamMember {
                    team_id,
                    user_id,
                    created_at: Default::default(),
                }),
            )
            .await?)
    }
}
