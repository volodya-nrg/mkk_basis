use http::StatusCode;
use reqwest::{
    Certificate, Client as ReqwestClient, Error as ReqwestError, Identity, Response, header,
    multipart::Form,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

use mkk_basis::adapter::db::postgres::Postgres as PostgresService;
use mkk_basis::transport::models::{
    RequestLimitOffset, RequestLogin, RequestRefreshToken, RequestRegister, RequestTask,
    RequestTaskComment, RequestTaskData, RequestTeam, RequestTeamInvite, RequestUserCreate,
    RequestUserUpdate, ResponseLogin, ResponseRefreshToken,
};

use super::rand;

pub type StatusCodeBodyError = Result<(StatusCode, String), ReqwestError>;

pub struct Client<'a> {
    addr: String,
    client: ReqwestClient,
    pub access_token: Arc<Mutex<String>>,
    pub refresh_token: Arc<Mutex<String>>,
    pg_service: &'a PostgresService,
}

impl<'a> Client<'a> {
    pub fn new(
        addr: String,
        ca: String,
        crt: String,
        key: String,
        pg_service: &'a PostgresService,
    ) -> Self {
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
            pg_service,
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
    async fn parse_response(&self, resp: Response) -> StatusCodeBodyError {
        let status_code = resp.status();
        let result = resp.text().await?;
        Ok((status_code, result))
        // Ok(resp.await?)
    }

    // etc
    pub async fn index<T>(&self, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self.client.get(&self.addr).send().await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn health<T>(&self, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/health", self.addr))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn page404<T>(&self, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/{}", self.addr, rand::str()))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn get_file<T>(&self, url_filepath: String, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let url_filepath = url_filepath
            .strip_prefix('/')
            .unwrap_or(url_filepath.as_str());
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/{}", self.addr, url_filepath))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }

    // auth
    pub async fn register<T>(&self, req: RequestRegister, is_full: bool, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/register", self.addr))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;

        if is_full {
            let email_code = self
                .pg_service
                .tbl_users
                .by_email(req.email.clone())
                .await
                .unwrap()
                .email_code
                .unwrap();

            self.register_confirm(
                Some(req.email),
                Some(email_code),
                |_: StatusCodeBodyError| {},
            )
            .await;
        }

        cb(result);
        self
    }
    pub async fn register_confirm<T>(
        &self,
        email: Option<String>,
        code: Option<String>,
        mut cb: T,
    ) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let mut address = format!("{}/register/confirm", self.addr);
        let mut query_items: Vec<String> = Vec::new();

        if let Some(v) = email {
            query_items.push(format!("email={}", v));
        }
        if let Some(v) = code {
            query_items.push(format!("code={}", v));
        }
        if query_items.len() > 0 {
            address = address + "?" + &query_items.join("&").to_string();
        }

        let result = (|| async {
            let response = self.client.get(address).send().await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn login<T>(&self, req: RequestLogin, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/login", self.addr))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;

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
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/logout", self.addr))
                .headers(self.headers().await)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;

        if let Ok((status_code, _body_str)) = &result
            && status_code.is_success()
        {
            let mut at = self.access_token.lock().await;
            let mut rt = self.refresh_token.lock().await;
            *at = "".to_string();
            *rt = "".to_string();
        }

        cb(result);
        self
    }

    pub async fn refresh_tokens<T>(&self, req: RequestRefreshToken, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/refresh_tokens", self.addr))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;

        if let Ok((status_code, body_str)) = &result
            && status_code.is_success()
        {
            let resp_login: ResponseRefreshToken =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            let mut at = self.access_token.lock().await;
            let mut rt = self.refresh_token.lock().await;
            *at = resp_login.access_token;
            *rt = resp_login.refresh_token;
        }

        cb(result);
        self
    }

    // teams
    pub async fn teams_list<T>(&self, limit: i32, offset: i32, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/teams", self.addr))
                .headers(self.headers().await)
                .json(&RequestLimitOffset { limit, offset })
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn teams_one<T>(&self, uuid: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/teams/{}", self.addr, uuid))
                .headers(self.headers().await)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn teams_create<T>(&self, req: RequestTeam, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/teams", self.addr))
                .headers(self.headers().await)
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn teams_update<T>(&self, item_id: Uuid, req: RequestTeam, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .put(format!("{}/api/v1/teams/{}", self.addr, item_id))
                .headers(self.headers().await)
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn teams_delete<T>(&self, item_id: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .delete(format!("{}/api/v1/teams/{}", self.addr, item_id))
                .headers(self.headers().await)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn teams_invite<T>(&self, item_id: Uuid, req: RequestTeamInvite, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/teams/{}/invite", self.addr, item_id))
                .headers(self.headers().await)
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }

    // tasks
    pub async fn tasks_list<T>(&self, req: RequestTaskData, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks", self.addr))
                .headers(self.headers().await)
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn tasks_one<T>(&self, item_id: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks/{}", self.addr, item_id))
                .headers(self.headers().await)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn tasks_create<T>(&self, req: RequestTask, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/tasks", self.addr))
                .headers(self.headers().await)
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn tasks_update<T>(&self, item_id: Uuid, req: RequestTask, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .put(format!("{}/api/v1/tasks/{}", self.addr, item_id))
                .headers(self.headers().await)
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn tasks_delete<T>(&self, item_id: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .delete(format!("{}/api/v1/tasks/{}", self.addr, item_id))
                .headers(self.headers().await)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn tasks_history<T>(&self, item_id: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks/{}/history", self.addr, item_id))
                .headers(self.headers().await)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }

    // users
    pub async fn users_list<T>(&self, limit: i32, offset: i32, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/users", self.addr))
                .headers(self.headers().await)
                .json(&RequestLimitOffset { limit, offset })
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn users_one<T>(&self, item_id: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/users/{}", self.addr, item_id))
                .headers(self.headers().await)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn users_create<T>(&self, req: RequestUserCreate, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let mut form = Form::new()
            .text("email", req.email)
            .text("password", req.password);

        if let Some(v) = req.name {
            form = form.text("name", v);
        }
        if let Some(v) = req.role {
            form = form.text("role", v);
        }
        if let Some(v) = req.avatar {
            form = form.file("avatar", v).await.unwrap();
        }

        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/users", self.addr))
                .headers(self.headers().await)
                .multipart(form)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn users_update<T>(&self, item_id: Uuid, req: RequestUserUpdate, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let mut form = Form::new();

        if let Some(v) = req.email {
            form = form.text("email", v);
        }
        if let Some(v) = req.password {
            form = form.text("password", v);
        }
        if let Some(v) = req.name {
            form = form.text("name", v);
        }
        if let Some(v) = req.role {
            form = form.text("role", v);
        }
        if let Some(v) = req.avatar {
            form = form.file("avatar", v).await.unwrap();
        }
        if req.is_remove_avatar {
            form = form.text("is_remove_avatar", "true");
        }

        let result = (|| async {
            let response = self
                .client
                .patch(format!("{}/api/v1/users/{}", self.addr, item_id))
                .headers(self.headers().await)
                .multipart(form)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn users_delete<T>(&self, item_id: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .delete(format!("{}/api/v1/users/{}", self.addr, item_id))
                .headers(self.headers().await)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }

    // task comments
    pub async fn task_comments_list<T>(
        &self,
        task_id: Uuid,
        limit: i32,
        offset: i32,
        mut cb: T,
    ) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks/{}/comments", self.addr, task_id))
                .headers(self.headers().await)
                .json(&RequestLimitOffset { limit, offset })
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn task_comments_create<T>(
        &self,
        task_id: Uuid,
        req: RequestTaskComment,
        mut cb: T,
    ) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/tasks/{}/comments", self.addr, task_id))
                .headers(self.headers().await)
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn task_comments_delete<T>(&self, item_id: Uuid, mut cb: T) -> &Self
    where
        T: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .delete(format!("{}/api/v1/tasks/comment/{}", self.addr, item_id))
                .headers(self.headers().await)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
}
