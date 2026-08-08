use fake::{Fake, Faker};
use mkk_basis::transport::models::*;
use reqwest::{Client as ReqwestClient, Response};
use std::time::Duration;
use uuid::Uuid;

pub struct Client {
    addr: String,
    client: ReqwestClient,
}

impl Client {
    pub fn new(addr: String) -> Self {
        /*
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("my-app/1.0")
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_static("Bearer token123")
        );
        */
        Self {
            addr,
            client: ReqwestClient::builder()
                .timeout(Duration::from_secs(10))
                .connect_timeout(Duration::from_secs(5))
                .user_agent("my-rust-client/1.0")
                .build()
                .unwrap(),
        }
    }
    async fn parse_response(&self, resp: Response) -> Result<(u16, String), String> {
        let status_code = resp.status().as_u16();
        let result = resp
            .text()
            .await
            .map_err(|e| format!("failed to read body: {:?}", e))?;
        Ok((status_code, result))
    }

    // etc
    pub async fn index(&self, cb: fn(Result<(u16, String), String>)) -> &Self {
        let result = self
            .client
            .get(&self.addr)
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => match self.parse_response(v).await {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };

        cb(result);
        self
    }
    pub async fn healthz(&self, cb: fn(Result<(u16, String), String>)) -> &Self {
        let result = self
            .client
            .get(format!("{}/healthz", self.addr))
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => match self.parse_response(v).await {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };

        cb(result);
        self
    }
    pub async fn page404(&self, cb: fn(Result<(u16, String), String>)) -> &Self {
        let random_string: String = Faker.fake();
        let result = self
            .client
            .get(format!("{}/{}", self.addr, random_string))
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => match self.parse_response(v).await {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };

        cb(result);
        self
    }
    pub async fn get_file(
        &self,
        url_filepath: String,
        cb: fn(Result<(u16, String), String>),
    ) -> &Self {
        let url_filepath = url_filepath
            .strip_prefix('/')
            .unwrap_or(url_filepath.as_str());
        let result = self
            .client
            .get(format!("{}/{}", self.addr, url_filepath))
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => match self.parse_response(v).await {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };

        cb(result);
        self
    }

    // auth
    pub async fn register(
        &self,
        req: RequestRegister,
        cb: fn(Result<(u16, String), String>),
    ) -> &Self {
        let result = self
            .client
            .post(format!("{}/api/v1/register", self.addr))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => match self.parse_response(v).await {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };

        cb(result);
        self
    }
    pub async fn login(&self, req: RequestLogin, cb: fn(Result<(u16, String), String>)) -> &Self {
        let result = self
            .client
            .post(format!("{}/api/v1/login", self.addr))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => match self.parse_response(v).await {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };

        cb(result);
        self
    }
    pub async fn logout(&self, cb: fn(Result<(u16, String), String>)) -> &Self {
        let result = self
            .client
            .post(format!("{}/api/v1/logout", self.addr))
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => match self.parse_response(v).await {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };

        cb(result);
        self
    }

    // teams
    pub async fn teams_list(
        &self,
        limit: i32,
        offset: i32,
        filter: String,
        cb: fn(Result<ResponseTeamsList, String>),
    ) -> &Self {
        let result = self
            .client
            .get(format!("{}/api/v1/teams", self.addr))
            .json(&RequestLimitOffsetFilter {
                limit,
                offset,
                filter,
            })
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => v,
            Err(e) => {
                cb(Err(e));
                return self;
            }
        };
        let result = result
            .json()
            .await
            .map_err(|e| format!("failed convert to json: {:?}", e));

        cb(result);
        self
    }
    pub async fn teams_create(
        &self,
        req: RequestTeamCreate,
        cb: fn(Result<ResponseTeam, String>),
    ) -> &Self {
        let result = self
            .client
            .post(format!("{}/api/v1/teams", self.addr))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => v,
            Err(e) => {
                cb(Err(e));
                return self;
            }
        };
        let result = result
            .json()
            .await
            .map_err(|e| format!("failed convert to json: {:?}", e));

        cb(result);
        self
    }
    pub async fn teams_invite(
        &self,
        team_id: Uuid,
        req: RequestTeamInvite,
        cb: fn(Result<Response, String>),
    ) -> &Self {
        let result = self
            .client
            .post(format!("{}/api/v1/teams/{}/invite", self.addr, team_id))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));

        cb(result);
        self
    }

    // tasks
    pub async fn tasks_list(
        &self,
        limit: i32,
        offset: i32,
        filter: String,
        cb: fn(Result<ResponseTasksList, String>),
    ) -> &Self {
        let result = self
            .client
            .get(format!("{}/api/v1/tasks", self.addr))
            .json(&RequestLimitOffsetFilter {
                limit,
                offset,
                filter,
            })
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => v,
            Err(e) => {
                cb(Err(e));
                return self;
            }
        };
        let result = result
            .json()
            .await
            .map_err(|e| format!("failed convert to json: {:?}", e));

        cb(result);
        self
    }
    pub async fn tasks_create(
        &self,
        req: RequestTask,
        cb: fn(Result<ResponseTask, String>),
    ) -> &Self {
        let result = self
            .client
            .post(format!("{}/api/v1/tasks", self.addr))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => v,
            Err(e) => {
                cb(Err(e));
                return self;
            }
        };
        let result = result
            .json()
            .await
            .map_err(|e| format!("failed convert to json: {:?}", e));

        cb(result);
        self
    }
    pub async fn tasks_update(
        &self,
        task_id: Uuid,
        req: RequestTask,
        cb: fn(Result<ResponseTask, String>),
    ) -> &Self {
        let result = self
            .client
            .put(format!("{}/api/v1/tasks/{}", self.addr, task_id))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => v,
            Err(e) => {
                cb(Err(e));
                return self;
            }
        };
        let result = result
            .json()
            .await
            .map_err(|e| format!("failed convert to json: {:?}", e));

        cb(result);
        self
    }
    pub async fn tasks_history(
        &self,
        task_id: Uuid,
        cb: fn(Result<ResponseTaskHistories, String>),
    ) -> &Self {
        let result = self
            .client
            .get(format!("{}/api/v1/tasks/{}/history", self.addr, task_id))
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));
        let result = match result {
            Ok(v) => v,
            Err(e) => {
                cb(Err(e));
                return self;
            }
        };
        let result = result
            .json()
            .await
            .map_err(|e| format!("failed convert to json: {:?}", e));

        cb(result);
        self
    }
}
