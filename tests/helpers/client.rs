use super::rand;
use http::StatusCode;
use mkk_basis::transport::models::*;
use reqwest::{Certificate, Client as ReqwestClient, Identity, Response, header};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct Client {
    addr: String,
    client: ReqwestClient,
    access_token: Arc<Mutex<String>>,
    refresh_token: Arc<Mutex<String>>,
}

impl Client {
    pub fn new(addr: String, ca: String, crt: String, key: String) -> Self {
        // ca-сертификат - чтоб проверить сервер
        // crt - чтоб сервер мог проверить клиента
        // key - доказательство владения crt

        let ca = Certificate::from_pem(ca.as_bytes()).unwrap();
        let identity =
            Identity::from_pem(format!("{}{}", crt.to_string(), key.to_string()).as_bytes())
                .unwrap();

        Self {
            addr: addr.to_string(),
            client: ReqwestClient::builder()
                .add_root_certificate(ca)
                .identity(identity)
                .tls_danger_accept_invalid_certs(false)
                .timeout(Duration::from_secs(10))
                .connect_timeout(Duration::from_secs(5))
                .user_agent("my-rust-client/1.0")
                .build()
                .unwrap(),
            access_token: Arc::new(Default::default()),
            refresh_token: Arc::new(Default::default()),
        }
    }
    async fn headers(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        let access_token_clone = self.access_token.lock().await.to_string();

        if !access_token_clone.is_empty() {
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {}", access_token_clone)).unwrap(),
            );
        }

        headers
    }
    async fn parse_response(&self, resp: Response) -> Result<(StatusCode, String), String> {
        let status_code = resp.status();
        let result = resp
            .text()
            .await
            .map_err(|e| format!("failed to read body: {:?}", e))?;
        Ok((status_code, result))
    }

    // etc
    pub async fn index<T>(&self, mut cb: T) -> &Self
    where
        T: FnMut(Result<(StatusCode, String), String>),
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
    pub async fn health<T>(&self, mut cb: T) -> &Self
    where
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .get(format!("{}/health", self.addr))
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
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .get(format!("{}/{}", self.addr, rand::str()))
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
        T: FnMut(Result<(StatusCode, String), String>),
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
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .post(format!("{}/api/v1/register", self.addr))
            // .headers(self.headers().await)
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
        T: FnMut(Result<(StatusCode, String), String>),
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

        if let Ok((status_code, body_str)) = &result
            && status_code.is_success()
        {
            let resp_login: ResponseLogin =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            let mut at = self.access_token.lock().await;
            let mut rt = self.refresh_token.lock().await;
            *at = resp_login.access_token;
            *rt = resp_login.refresh_token;
        }

        cb(result);
        self
    }
    pub async fn logout<T>(&self, mut cb: T) -> &Self
    where
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .post(format!("{}/api/v1/logout", self.addr))
            .headers(self.headers().await)
            .send()
            .await
            .map_err(|e| format!("failed to request: {:?}", e));

        if result.is_ok() {
            let mut at = self.access_token.lock().await;
            let mut rt = self.refresh_token.lock().await;
            *at = "".to_string();
            *rt = "".to_string();
        }

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
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .get(format!("{}/api/v1/teams", self.addr))
            .headers(self.headers().await)
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
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .post(format!("{}/api/v1/teams", self.addr))
            .headers(self.headers().await)
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
        T: FnMut(Result<(StatusCode, String), String>), // + 'static,
    {
        let result = self
            .client
            .post(format!("{}/api/v1/teams/{}/invite", self.addr, team_id))
            .headers(self.headers().await)
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
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .get(format!("{}/api/v1/tasks", self.addr))
            .headers(self.headers().await)
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
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .post(format!("{}/api/v1/tasks", self.addr))
            .headers(self.headers().await)
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
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .put(format!("{}/api/v1/tasks/{}", self.addr, task_id))
            .headers(self.headers().await)
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
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .get(format!("{}/api/v1/tasks/{}/history", self.addr, task_id))
            .headers(self.headers().await)
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

    // users
    pub async fn users_list<T>(&self, limit: i32, offset: i32, mut cb: T) -> &Self
    where
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .get(format!("{}/api/v1/users", self.addr))
            .headers(self.headers().await)
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
    pub async fn users_one<T>(&self, uuid: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .get(format!("{}/api/v1/users/{}", self.addr, uuid))
            .headers(self.headers().await)
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
    pub async fn users_create<T>(&self, req: RequestUser, mut cb: T) -> &Self
    where
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .post(format!("{}/api/v1/users", self.addr))
            .headers(self.headers().await)
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
    pub async fn users_update<T>(&self, user_id: Uuid, req: RequestUser, mut cb: T) -> &Self
    where
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .put(format!("{}/api/v1/users/{}", self.addr, user_id))
            .headers(self.headers().await)
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
    pub async fn users_delete<T>(&self, user_id: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(Result<(StatusCode, String), String>),
    {
        let result = self
            .client
            .delete(format!("{}/api/v1/users/{}", self.addr, user_id))
            .headers(self.headers().await)
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
