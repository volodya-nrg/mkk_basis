use mkk_basis::adapter::email::EmailSender;
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

impl EmailSender for EmailServiceMock {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        log::debug!(
            "emulate send email. to: {}; subject: {}; body: {}",
            to,
            subject,
            body
        );
        Ok(())
    }

    fn save_code(&self, email: &str, code: &str) {
        let lock_attempt = self.m_save_code.write();
        if let Ok(mut guard) = lock_attempt {
            guard.insert(email.to_string(), code.to_string());
        }
    }
    fn get_code(&self, email: &str) -> String {
        self.m_save_code.read().map_or_else(
            |_| String::new(),
            |guard| guard.get(email).cloned().unwrap_or_default(),
        )
    }
}
