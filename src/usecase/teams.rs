use http::StatusCode;
use std::sync::Arc;
use uuid::Uuid;

use super::{
    UseCaseError, mapper,
    models::{Team, TeamMember},
};

use crate::{
    adapter::db::{postgres::tables::users::Role as UserRole, storage::Storage},
    app_errors::AppErr,
};

#[derive(Clone)] // clone из-за axum
pub struct Teams {
    storage: Arc<dyn Storage>,
}

impl Teams {
    pub const fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Team>, i64), UseCaseError> {
        let mut tx = self.storage.begin().await?;
        let mut conn = tx.get_conn().await?;
        let list = self.storage.teams().list(&mut conn, limit, offset).await?;

        tx.commit().await?;
        Ok((
            list.0.into_iter().map(mapper::team_db_to_team_uc).collect(),
            list.1,
        ))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Team, UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        Ok(mapper::team_db_to_team_uc(
            self.storage
                .teams()
                .one(&mut conn.as_mut(), item_id)
                .await?,
        ))
    }
    pub async fn create(&self, team: Team) -> Result<Uuid, UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        Ok(self
            .storage
            .teams()
            .create(&mut conn.as_mut(), mapper::team_uc_to_team_db(team))
            .await?)
    }
    pub async fn update(&self, team: Team) -> Result<(), UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        Ok(self
            .storage
            .teams()
            .update(&mut conn.as_mut(), mapper::team_uc_to_team_db(team))
            .await?)
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        Ok(self
            .storage
            .teams()
            .delete(&mut conn.as_mut(), item_id)
            .await?)
    }
    // пригласить может только owner или admin
    pub async fn invite(
        &self,
        profile_id: Uuid,
        profile_role: Option<String>,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), UseCaseError> {
        let is_has_access = if let Some(role) = profile_role
            && role == UserRole::Admin.to_string()
        {
            true
        } else {
            let mut conn = self.storage.get_conn().await?;
            self.storage
                .teams()
                .one(&mut conn.as_mut(), team_id)
                .await?
                .created_by
                == profile_id
        };

        if !is_has_access {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::FORBIDDEN,
                public_err: AppErr::NoRules.to_string(),
                internal_err: None,
            });
        }

        let mut conn = self.storage.get_conn().await?;
        Ok(self
            .storage
            .team_members()
            .create(
                &mut conn.as_mut(),
                mapper::team_member_uc_to_team_member_db(TeamMember {
                    team_id,
                    user_id,
                    created_at: Default::default(),
                }),
            )
            .await?)
    }
}
