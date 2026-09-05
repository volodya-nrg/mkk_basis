use http::StatusCode;
use reqwest::{
    Certificate, Client as ReqwestClient, Error as ReqwestError, Identity, Response, header,
    multipart::Form,
};
use std::time::Duration;
use uuid::Uuid;

use mkk_basis::adapter::db::postgres::Postgres as PostgresService;
use mkk_basis::transport::models::{
    RequestLimitOffset, RequestLogin, RequestRegister, RequestTask, RequestTaskComment,
    RequestTaskData, RequestTeam, RequestTeamInvite, RequestUserCreate, RequestUserUpdate,
};

use super::rand;

// mut - везде потому что перемешиваются методы, то (не)mut и передается ссылка. Из-за этого нужно
// указать один вариант.

pub type StatusCodeBodyError = Result<(StatusCode, String), ReqwestError>;

pub struct Client<'a> {
    addr: String,
    client: ReqwestClient,
    pub pg_service: &'a PostgresService,
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
                .user_agent("my-rust-test-client/1.0")
                .add_root_certificate(ca)
                .identity(identity)
                .timeout(Duration::from_secs(10)) // общее время соединения
                .cookie_store(true)
                .build()
                .unwrap(),
            pg_service,
        }
    }
    async fn parse_response(&self, resp: Response) -> StatusCodeBodyError {
        let status_code = resp.status();
        let result = resp.text().await?;
        Ok((status_code, result))
    }

    // etc
    pub async fn index<F>(&mut self, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self.client.get(&self.addr).send().await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn health<F>(&mut self, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
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
    pub async fn page404<F>(&mut self, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
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
    pub async fn get_file<F>(&mut self, url_filepath: String, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
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
    pub async fn register<F>(&mut self, req: RequestRegister, is_full: bool, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
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

            self.register_confirm(Some(req.email), Some(email_code), |result2| {
                let (status_code, _body_str) = result2.unwrap();
                assert!(status_code.is_success());
            })
            .await;
        }

        cb(result);
        self
    }
    pub async fn register_confirm<F>(
        &mut self,
        email: Option<String>,
        code: Option<String>,
        mut cb: F,
    ) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
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
    pub async fn login<F>(&mut self, req: RequestLogin, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/login", self.addr))
                .json(&req)
                .send()
                .await?;
            let set_cookies: Vec<_> = response.headers().get_all("set-cookie").iter().collect();
            for header_value in set_cookies {
                let value = header_value.to_str().unwrap().to_string();
                log::debug!("new cookie: {}", value);
            }
            self.parse_response(response).await
        })()
        .await;

        cb(result);
        self
    }
    pub async fn logout<F>(&mut self, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/logout", self.addr))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;

        cb(result);
        self
    }
    pub async fn refresh_tokens<F>(&mut self, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/refresh_tokens", self.addr))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;

        cb(result);
        self
    }

    // teams
    pub async fn teams_list<F>(&mut self, limit: i32, offset: i32, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/teams", self.addr))
                .json(&RequestLimitOffset { limit, offset })
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn teams_one<F>(&mut self, uuid: Uuid, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/teams/{}", self.addr, uuid))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn teams_create<F>(&mut self, req: RequestTeam, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/teams", self.addr))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn teams_update<F>(&mut self, item_id: Uuid, req: RequestTeam, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .put(format!("{}/api/v1/teams/{}", self.addr, item_id))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn teams_delete<F>(&mut self, item_id: Uuid, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .delete(format!("{}/api/v1/teams/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn teams_invite<F>(
        &mut self,
        item_id: Uuid,
        req: RequestTeamInvite,
        mut cb: F,
    ) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/teams/{}/invite", self.addr, item_id))
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
    pub async fn tasks_list<F>(&mut self, req: RequestTaskData, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks", self.addr))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn tasks_one<F>(&mut self, item_id: Uuid, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn tasks_create<F>(&mut self, req: RequestTask, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/tasks", self.addr))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn tasks_update<F>(&mut self, item_id: Uuid, req: RequestTask, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .put(format!("{}/api/v1/tasks/{}", self.addr, item_id))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn tasks_delete<F>(&mut self, item_id: Uuid, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .delete(format!("{}/api/v1/tasks/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn tasks_history<F>(&mut self, item_id: Uuid, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks/{}/history", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }

    // users
    pub async fn users_list<F>(&mut self, limit: i32, offset: i32, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/users", self.addr))
                .json(&RequestLimitOffset { limit, offset })
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn users_one<F>(&mut self, item_id: Uuid, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/users/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn users_create<F>(&mut self, req: RequestUserCreate, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
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
                .multipart(form)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn users_update<F>(
        &mut self,
        item_id: Uuid,
        req: RequestUserUpdate,
        mut cb: F,
    ) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
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
                .multipart(form)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn users_delete<F>(&mut self, item_id: Uuid, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .delete(format!("{}/api/v1/users/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }

    // task comments
    pub async fn task_comments_list<F>(
        &mut self,
        task_id: Uuid,
        limit: i32,
        offset: i32,
        mut cb: F,
    ) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks/{}/comments", self.addr, task_id))
                .json(&RequestLimitOffset { limit, offset })
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn task_comments_create<F>(
        &mut self,
        task_id: Uuid,
        req: RequestTaskComment,
        mut cb: F,
    ) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .post(format!("{}/api/v1/tasks/{}/comments", self.addr, task_id))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
    pub async fn task_comments_delete<F>(&mut self, item_id: Uuid, mut cb: F) -> &mut Self
    where
        F: FnMut(StatusCodeBodyError),
    {
        let result = (|| async {
            let response = self
                .client
                .delete(format!("{}/api/v1/tasks/comment/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        })()
        .await;
        cb(result);
        self
    }
}
