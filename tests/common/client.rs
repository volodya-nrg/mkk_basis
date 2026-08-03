use mkk_basis::transport::models::*;
use reqwest::Client as ReqwestClient;
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
    pub async fn register(
        &self,
        req: RequestRegister,
        cb: fn(Result<ResponseRegister, String>),
    ) -> &Self {
        let result = self
            .client
            .post(format!("{}/api/v1/register", self.addr))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("failed to request: {e}"));
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
            .map_err(|e| format!("failed convert to json: {e}"));

        cb(result);
        self
    }
    pub async fn login(&self, email: String, password: String) {
        let res = self
            .client
            .post(format!("{}/api/v1/x", self.addr))
            .body("the exact body that is sent")
            .send()
            .await;
    }
    pub fn teams_list(&self, limit: i32, offset: i32) {}
    pub fn teams_create(&self) {}
    pub fn teams_invite(&self) {}
    pub fn tasks_list(&self, limit: i32, offset: i32) {}
    pub fn tasks_create(&self) {}
    pub fn tasks_update(&self) {}
    pub fn tasks_history(&self, task_id: Uuid) {}
}
