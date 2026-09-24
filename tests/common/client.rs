#![allow(dead_code)]

use http::StatusCode;
use reqwest::{Certificate, Identity, Response, multipart::Form};
use std::sync::Arc;
use std::time::Duration;

use mkk_basis::adapter::email::EmailSender;
use mkk_basis::transport::models::{
    RequestLimitOffset, RequestLogin, RequestRegister, RequestTask, RequestTaskComment,
    RequestTaskData, RequestTeam, RequestTeamInvite, RequestUserCreate, RequestUserUpdate,
};

use super::rand;

pub type StatusCodeBodyError = Result<(StatusCode, String), reqwest::Error>;

// Client - клиент в некоторых методах имеет "mut self", поэтому удобней везде так объявить и возвращать
// мутабельный объект. Если клиента отдавать по значениям, то между может быть move, что не удобно.
// Частично сделать &mut self не получится, т.к. каждый метод по сути отдает разный тип ((не)mut).

pub struct Client<ES> {
    addr: String,
    client: reqwest::Client,
    email_service: Arc<ES>,
}

impl<ES> Client<ES>
where
    ES: EmailSender,
{
    pub fn new(addr: String, ca: String, crt: String, key: String, email_service: Arc<ES>) -> Self {
        // ca-сертификат - чтоб проверить сервер
        // crt - чтоб сервер мог проверить клиента
        // key - доказательство владения crt

        let ca = Certificate::from_pem(ca.as_bytes()).unwrap();
        let identity = Identity::from_pem(format!("{}{}", crt, key).as_bytes()).unwrap();

        Self {
            addr,
            client: reqwest::Client::builder()
                .user_agent("my-rust-test-client/1.0")
                .add_root_certificate(ca)
                .identity(identity)
                .timeout(Duration::from_secs(10)) // общее время соединения
                .cookie_store(true)
                .build()
                .unwrap(),
            email_service,
        }
    }
    async fn parse_response(&self, resp: Response) -> StatusCodeBodyError {
        let status_code = resp.status();
        let result = resp.text().await?;
        Ok((status_code, result))
    }

    // etc
    pub async fn index(&mut self, mut cb: impl FnMut(StatusCodeBodyError)) -> &mut Self {
        let result = async {
            let response = self.client.get(&self.addr).send().await?;
            self.parse_response(response).await
        }
        .await;
        cb(result);
        self
    }
    pub async fn health(&mut self, mut cb: impl FnMut(StatusCodeBodyError)) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/health", self.addr))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;
        cb(result);
        self
    }
    pub async fn page404(&mut self, mut cb: impl FnMut(StatusCodeBodyError)) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/{}", self.addr, rand::str()))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;
        cb(result);
        self
    }
    pub async fn get_file(
        &mut self,
        url_filepath: String,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let url_filepath = url_filepath
            .strip_prefix('/')
            .unwrap_or(url_filepath.as_str());
        let result = async {
            let response = self
                .client
                .get(format!("{}/{}", self.addr, url_filepath))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;
        cb(result);
        self
    }
    pub async fn swagger_ui(&mut self, mut cb: impl FnMut(StatusCodeBodyError)) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/swagger-ui", self.addr))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;
        cb(result);
        self
    }
    pub async fn openapi(&mut self, mut cb: impl FnMut(StatusCodeBodyError)) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/api-docs/openapi.json", self.addr))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;
        cb(result);
        self
    }

    // auth
    pub async fn register(
        &mut self,
        req: RequestRegister,
        is_full: bool,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .post(format!("{}/api/v1/register", self.addr))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        if is_full {
            let email_code = self.email_service.get_code(&req.email);
            let result2 = self
                .register_confirm_common(Some(req.email), Some(email_code))
                .await;
            let (status_code, _body_str) = result2.unwrap();
            assert!(status_code.is_success());
        }

        cb(result);
        self
    }
    pub async fn register_confirm(
        &mut self,
        email: Option<String>,
        code: Option<String>,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        cb(self.register_confirm_common(email, code).await);
        self
    }
    async fn register_confirm_common(
        &self,
        email: Option<String>,
        code: Option<String>,
    ) -> StatusCodeBodyError {
        let mut address = format!("{}/register/confirm", self.addr);
        let mut query_items: Vec<String> = Vec::new();

        if let Some(v) = email {
            query_items.push(format!("email={}", v));
        }
        if let Some(v) = code {
            query_items.push(format!("code={}", v));
        }
        if !query_items.is_empty() {
            address = address + "?" + &query_items.join("&");
        }

        let response = self.client.get(address).send().await?;
        self.parse_response(response).await
    }
    pub async fn login(
        &mut self,
        req: RequestLogin,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
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
        }
        .await;

        cb(result);
        self
    }
    pub async fn logout(&mut self, mut cb: impl FnMut(StatusCodeBodyError)) -> &mut Self {
        let result = async {
            let response = self
                .client
                .post(format!("{}/api/v1/logout", self.addr))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn refresh_tokens(&mut self, mut cb: impl FnMut(StatusCodeBodyError)) -> &mut Self {
        let result = async {
            let response = self
                .client
                .post(format!("{}/api/v1/refresh_tokens", self.addr))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }

    // teams
    pub async fn teams_list(
        &mut self,
        limit: i32,
        offset: i32,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/api/v1/teams", self.addr))
                .json(&RequestLimitOffset { limit, offset })
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;
        cb(result);
        self
    }
    pub async fn teams_one(
        &mut self,
        uuid: String,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/api/v1/teams/{}", self.addr, uuid))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;
        cb(result);
        self
    }
    pub async fn teams_create(
        &mut self,
        req: RequestTeam,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .post(format!("{}/api/v1/teams", self.addr))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn teams_update(
        &mut self,
        item_id: String,
        req: RequestTeam,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .put(format!("{}/api/v1/teams/{}", self.addr, item_id))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn teams_delete(
        &mut self,
        item_id: String,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .delete(format!("{}/api/v1/teams/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn teams_invite(
        &mut self,
        item_id: String,
        req: RequestTeamInvite,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .post(format!("{}/api/v1/teams/{}/invite", self.addr, item_id))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }

    // tasks
    pub async fn tasks_list(
        &mut self,
        req: RequestTaskData,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks", self.addr))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn tasks_one(
        &mut self,
        item_id: String,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn tasks_create(
        &mut self,
        req: RequestTask,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .post(format!("{}/api/v1/tasks", self.addr))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn tasks_update(
        &mut self,
        item_id: String,
        req: RequestTask,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .put(format!("{}/api/v1/tasks/{}", self.addr, item_id))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn tasks_delete(
        &mut self,
        item_id: String,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .delete(format!("{}/api/v1/tasks/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn tasks_history(
        &mut self,
        item_id: String,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks/{}/history", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }

    // users
    pub async fn users_list(
        &mut self,
        limit: i32,
        offset: i32,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/api/v1/users", self.addr))
                .json(&RequestLimitOffset { limit, offset })
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn users_one(
        &mut self,
        item_id: String,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/api/v1/users/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn users_create(
        &mut self,
        req: RequestUserCreate,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
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

        let result = async {
            let response = self
                .client
                .post(format!("{}/api/v1/users", self.addr))
                .multipart(form)
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn users_update(
        &mut self,
        item_id: String,
        req: RequestUserUpdate,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
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

        let result = async {
            let response = self
                .client
                .patch(format!("{}/api/v1/users/{}", self.addr, item_id))
                .multipart(form)
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn users_delete(
        &mut self,
        item_id: String,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .delete(format!("{}/api/v1/users/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }

    // task comments
    pub async fn task_comments_list(
        &mut self,
        task_id: String,
        limit: i32,
        offset: i32,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .get(format!("{}/api/v1/tasks/{}/comments", self.addr, task_id))
                .json(&RequestLimitOffset { limit, offset })
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn task_comments_create(
        &mut self,
        task_id: String,
        req: RequestTaskComment,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .post(format!("{}/api/v1/tasks/{}/comments", self.addr, task_id))
                .json(&req)
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
    pub async fn task_comments_delete(
        &mut self,
        item_id: String,
        mut cb: impl FnMut(StatusCodeBodyError),
    ) -> &mut Self {
        let result = async {
            let response = self
                .client
                .delete(format!("{}/api/v1/tasks/comment/{}", self.addr, item_id))
                .send()
                .await?;
            self.parse_response(response).await
        }
        .await;

        cb(result);
        self
    }
}
