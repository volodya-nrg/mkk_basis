use async_trait::async_trait;
use mkk_basis::adapter::email::{ConfirmationCodeStorer, EmailError, EmailSender};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct ConfirmationCodeStoreMock {
    m_save_code: Arc<RwLock<HashMap<String, String>>>,
}

impl ConfirmationCodeStoreMock {
    pub fn new() -> Self {
        Self {
            m_save_code: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
#[async_trait]
impl ConfirmationCodeStorer for ConfirmationCodeStoreMock {
    fn store(&self, email: &str, code: &str) {
        let lock_attempt = self.m_save_code.write();
        if let Ok(mut guard) = lock_attempt {
            guard.insert(email.to_string(), code.to_string());
        }
    }

    fn take(&self, email: &str) -> String {
        self.m_save_code.read().map_or_else(
            |_| String::new(),
            |guard| guard.get(email).cloned().unwrap_or_default(),
        )
    }
}

#[derive(Clone)]
pub struct EmailServiceMock {}

#[async_trait]
impl EmailSender for EmailServiceMock {
    fn send(&self, _to: &str, _subject: &str, _body: &str) -> Result<(), EmailError> {
        // let re = Regex::new(r#"href="([^"]+)""#).unwrap();
        // let url_str = &re.captures(body).unwrap()[1];
        // let url = Url::parse(url_str).unwrap();
        // let mut email = None;
        // let mut code = None;
        //
        // for (key, value) in url.query_pairs() {
        //     match key.as_ref() {
        //         "email" => email = Some(value.into_owned()),
        //         "code" => code = Some(value.into_owned()),
        //         _ => {}
        //     }
        // }
        //
        // if let Some(email_value) = email
        //     && let Some(code_value) = code
        // {
        //     self.save_code(email_value, code_value)
        // }

        Ok(())
    }
}
