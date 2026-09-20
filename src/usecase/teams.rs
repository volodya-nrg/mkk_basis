use http::StatusCode;
use uuid::Uuid;

use crate::{
    adapter::db::postgres::{
        tables::{
            team_members::TeamMembers as DBTeamMembers, teams::Teams as DBTeams,
            users::Role as UserRole,
        },
        transactor::Transactor,
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
    pub const fn new(
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
        Ok(self
            .transactor
            .in_transaction::<_, _, UseCaseError>(async |tx| {
                let list = self.teams_repo.list(tx, limit, offset).await?;
                Ok((
                    list.0.into_iter().map(mapper::team_db_to_team_uc).collect(),
                    list.1,
                ))
            })
            .await?)
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Team, UseCaseError> {
        let mut db_conn = self.transactor.conn().await?;
        Ok(mapper::team_db_to_team_uc(
            self.teams_repo.one(&mut db_conn, item_id).await?,
        ))
    }
    pub async fn create(&self, team: Team) -> Result<Uuid, UseCaseError> {
        let mut db_conn = self.transactor.conn().await?;
        Ok(self
            .teams_repo
            .create(&mut db_conn, mapper::team_uc_to_team_db(team))
            .await?)
    }
    pub async fn update(&self, team: Team) -> Result<(), UseCaseError> {
        let mut db_conn = self.transactor.conn().await?;
        Ok(self
            .teams_repo
            .update(&mut db_conn, mapper::team_uc_to_team_db(team))
            .await?)
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        let mut db_conn = self.transactor.conn().await?;
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
        let mut db_conn = self.transactor.conn().await?;
        let is_has_access = if let Some(role) = profile_role
            && role == UserRole::Admin.to_string()
        {
            true
        } else {
            self.teams_repo
                .one(&mut db_conn, team_id)
                .await?
                .created_by
                == profile_id
        };

        if !is_has_access {
            return Err(UseCaseError::Transport {
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
