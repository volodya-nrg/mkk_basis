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
    pub async fn index<T>(&self, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
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
    pub async fn healthz<T>(&self, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
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
    pub async fn page404<T>(&self, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
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
    pub async fn get_file<T>(&self, url_filepath: String, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
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
    pub async fn register<T>(&self, req: RequestRegister, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
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
    pub async fn login<T>(&self, req: RequestLogin, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
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
    pub async fn logout<T>(&self, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
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
    pub async fn teams_list<T>(&self, limit: i32, offset: i32, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
        let result = self
            .client
            .get(format!("{}/api/v1/teams", self.addr))
            .json(&RequestLimitOffset { limit, offset })
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
    pub async fn teams_create<T>(&self, req: RequestTeamCreate, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
        let result = self
            .client
            .post(format!("{}/api/v1/teams", self.addr))
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
    pub async fn teams_invite<T>(&self, team_id: Uuid, req: RequestTeamInvite, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>), // + 'static,
    {
        let result = self
            .client
            .post(format!("{}/api/v1/teams/{}/invite", self.addr, team_id))
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

    // tasks
    pub async fn tasks_list<T>(&self, limit: i32, offset: i32, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
        let result = self
            .client
            .get(format!("{}/api/v1/tasks", self.addr))
            .json(&RequestLimitOffset { limit, offset })
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
    pub async fn tasks_create<T>(&self, req: RequestTask, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
        let result = self
            .client
            .post(format!("{}/api/v1/tasks", self.addr))
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
    pub async fn tasks_update<T>(&self, task_id: Uuid, req: RequestTask, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
        let result = self
            .client
            .put(format!("{}/api/v1/tasks/{}", self.addr, task_id))
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
    pub async fn tasks_history<T>(&self, task_id: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(Result<(u16, String), String>),
    {
        let result = self
            .client
            .get(format!("{}/api/v1/tasks/{}/history", self.addr, task_id))
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
}
