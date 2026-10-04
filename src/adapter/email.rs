use lettre::{
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
    {Address, Message, SmtpTransport, Transport},
};
use std::time::Duration;

// все таки надежнее создавать ошибки через thiserror::Error, потому что:
// 1. отсутствует отличие ошибок
// 2. теряется источник (предыдущая ошибка в цепочке)
// 3. не реализует std::error::Error
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("invalid email address: {0}")]
    InvalidEmailAddress(String),
    #[error("{0}")]
    Common(String),
}

// Crate "async_trait" упрощает объявление Pin<Box>, внутренних vtable у трейтов и разных lifetime.
// Укажем сразу поддержку "Send + Sync + 'static", чтоб не писать подобное в других местах.
#[async_trait::async_trait]
pub trait EmailSender: Send + Sync + 'static {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), EmailError>;
}

pub struct Email {
    host: String,
    login: String,
    pass: String,
    from_email: String,
    from_name: String,
    timeout: Duration,
}

impl Email {
    pub fn new(
        ref_host: &str,
        ref_login: &str,
        ref_pass: &str,
        ref_from_email: &str,
        ref_from_name: &str,
        timeout: Duration,
    ) -> Self {
        Self {
            host: ref_host.to_string(),
            login: ref_login.to_string(),
            pass: ref_pass.to_string(),
            from_email: ref_from_email.to_string(),
            from_name: ref_from_name.to_string(),
            timeout,
        }
    }
}

#[async_trait::async_trait]
impl EmailSender for Email {
    fn send(&self, ref_to: &str, ref_subject: &str, ref_body: &str) -> Result<(), EmailError> {
        let (local_from_email, domain_from_email) =
            self.from_email.split_once('@').ok_or_else(|| {
                EmailError::InvalidEmailAddress("invalid email: missing @ from 'from'".to_string())
            })?;
        let address_from_email =
            Address::new(local_from_email, domain_from_email).map_err(|e| {
                EmailError::Common(format!("failed to create address from 'from': {e}"))
            })?;
        let mailbox_from = Mailbox::new(Some(self.from_name.clone()), address_from_email);
        let (local_to, domain_to) = ref_to.split_once('@').ok_or_else(|| {
            EmailError::InvalidEmailAddress("invalid email: missing @ from 'to'".to_string())
        })?; // linter просит использовать эту ф-ию, а не ok_or
        let address_to = Address::new(local_to, domain_to)
            .map_err(|e| EmailError::Common(format!("failed to create address from 'to': {e}")))?;
        let mailbox_to = Mailbox::new(None, address_to);
        let email = Message::builder()
            .from(mailbox_from)
            // .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to(mailbox_to)
            .subject(ref_subject)
            .header(ContentType::TEXT_HTML)
            .body(ref_body.to_string())
            .map_err(|e| EmailError::Common(format!("failed to create body: {e}")))?;

        SmtpTransport::starttls_relay(self.host.as_str())
            .map_err(|e| EmailError::Common(format!("failed to create smtp-transport: {e}")))?
            .timeout(Some(self.timeout))
            .credentials(Credentials::new(self.login.clone(), self.pass.clone()))
            .build()
            .send(&email)
            .map_err(|e| EmailError::Common(format!("failed to send: {e}")))
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_send() {
        let result = Email::new(
            "smtp.yandex.ru",
            "support@altair.uz",
            "x",
            "support@altair.uz",
            "support",
            Duration::from_secs(3),
        )
        .send("volodya-nrg@mail.ru", "my subj", "my <strong>body</strong>");
        assert!(result.is_ok());
    }
}
