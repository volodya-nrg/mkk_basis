use mkk_basis::adapter::email::{EmailError, EmailSender};

#[derive(Clone)]
pub struct EmailServiceMock {}
impl EmailSender for EmailServiceMock {
    fn send(&self, _to: String, _subject: String, _body: String) -> Result<(), EmailError> {
        Ok(())
    }
}
