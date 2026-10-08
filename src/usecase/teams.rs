use anyhow::Context;
use http::StatusCode;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    adapter::db::{postgres::tables::users::Role as UserRole, storage::Storage},
    app_errors::AppErr,
};

use super::{
    errors::UseCaseError,
    mapper,
    models::{Team, TeamMember},
};

#[derive(Clone)] // clone из-за axum
pub struct Teams {
    storage: Arc<dyn Storage>,
}

impl Teams {
    pub const fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> anyhow::Result<(Vec<Team>, i64)> {
        let mut tx = self
            .storage
            .begin()
            .await
            .context("failed to create tx-begin")?;
        let mut conn = tx.get_conn().await.context("failed to get tx-conn")?;
        let list = self
            .storage
            .teams()
            .list(&mut conn, limit, offset)
            .await
            .context("failed to get list teams")?;

        tx.commit().await.context("failed to tx-commit")?;
        Ok((
            list.0.into_iter().map(mapper::team_db_to_team_uc).collect(),
            list.1,
        ))
    }
    pub async fn one(&self, item_id: Uuid) -> anyhow::Result<Team> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        Ok(mapper::team_db_to_team_uc(
            self.storage
                .teams()
                .one(&mut conn.as_mut(), item_id)
                .await
                .context("failed to get team")?,
        ))
    }
    pub async fn create(&self, team: Team) -> anyhow::Result<Uuid> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        self.storage
            .teams()
            .create(&mut conn.as_mut(), mapper::team_uc_to_team_db(team))
            .await
            .context("failed to create team")
    }
    pub async fn update(&self, team: Team) -> anyhow::Result<()> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        self.storage
            .teams()
            .update(&mut conn.as_mut(), mapper::team_uc_to_team_db(team))
            .await
            .context("failed to update team")
    }
    pub async fn delete(&self, item_id: Uuid) -> anyhow::Result<()> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        self.storage
            .teams()
            .delete(&mut conn.as_mut(), item_id)
            .await
            .context("failed to delete team")
    }
    // пригласить может только owner или admin
    pub async fn invite(
        &self,
        profile_id: Uuid,
        profile_role: Option<String>,
        team_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<()> {
        let is_has_access = if let Some(role) = profile_role
            && role == UserRole::Admin.to_string()
        {
            true
        } else {
            let mut conn = self
                .storage
                .get_conn()
                .await
                .context("failed to get db-conn")?;
            self.storage
                .teams()
                .one(&mut conn.as_mut(), team_id)
                .await
                .context("failed to get team")?
                .created_by
                == profile_id
        };

        if !is_has_access {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::FORBIDDEN,
                public_err: AppErr::NoRules.to_string(),
                internal_err: None,
            }
            .into());
        }

        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        self.storage
            .team_members()
            .create(
                &mut conn.as_mut(),
                mapper::team_member_uc_to_team_member_db(TeamMember {
                    team_id,
                    user_id,
                    created_at: Default::default(),
                }),
            )
            .await
            .context("failed to create team-member")
    }
}
