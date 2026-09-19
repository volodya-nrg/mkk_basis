use mkk_basis::adapter::email::EmailSender;
use std::collections::HashMap;

#[derive(Clone)]
pub struct EmailServiceMock {
    m_save_code: HashMap<String, String>,
}
impl EmailServiceMock {
    pub fn new() -> Self {
        Self {
            m_save_code: HashMap::new(),
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

    fn save_code(&mut self, email: &str, code: &str) {
        let _ = self.m_save_code.insert(email.to_string(), code.to_string());
    }
    fn get_code(&self, email: &str) -> String {
        self.m_save_code.get(email).cloned().unwrap_or_default()
    }
}
