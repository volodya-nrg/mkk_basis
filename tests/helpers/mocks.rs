use mkk_basis::adapter::email::EmailSender;

#[derive(Clone)]
pub struct EmailServiceMock {}
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
}
