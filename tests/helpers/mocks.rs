use mkk_basis::adapter::email::{EmailError, EmailSender};

#[derive(Clone)]
pub struct EmailServiceMock {}
impl EmailSender for EmailServiceMock {
    fn send(&self, to: String, subject: String, body: String) -> Result<(), EmailError> {
        log::debug!(
            "emulate send email. to: {}; subject: {}; body: {}",
            to,
            subject,
            body
        );
        Ok(())
    }
}
