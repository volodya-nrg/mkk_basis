use mkk_basis::adapter::email::EmailSender;
use regex::Regex;
use reqwest::Url;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct EmailServiceMock {
    m_save_code: Arc<RwLock<HashMap<String, String>>>,
}
impl EmailServiceMock {
    pub fn new() -> Self {
        Self {
            m_save_code: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl EmailServiceMock {
    pub fn get_code(&self, email: String) -> String {
        self.m_save_code.read().map_or_else(
            |_| String::new(),
            |guard| guard.get(&email).cloned().unwrap_or_default(),
        )
    }
    fn save_code(&self, email: String, code: String) {
        let lock_attempt = self.m_save_code.write();
        if let Ok(mut guard) = lock_attempt {
            guard.insert(email, code);
        }
    }
}

#[async_trait::async_trait]
impl EmailSender for EmailServiceMock {
    fn send(&self, _to: &str, _subject: &str, body: &str) -> Result<(), String> {
        let re = Regex::new(r#"href="([^"]+)""#).unwrap();
        let url_str = &re.captures(body).unwrap()[1];
        let url = Url::parse(url_str).unwrap();
        let mut email = None;
        let mut code = None;

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "email" => email = Some(value.into_owned()),
                "code" => code = Some(value.into_owned()),
                _ => {}
            }
        }

        if let Some(email_value) = email
            && let Some(code_value) = code
        {
            self.save_code(email_value, code_value)
        }

        Ok(())
    }
}
