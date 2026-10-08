use async_trait::async_trait;
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

    #[error("lettre error: {0}")]
    FailedLettre(#[from] lettre::error::Error),
    #[error("lettre address error: {0}")]
    FailedLettreAddress(#[from] lettre::address::AddressError),
    #[error("lettre transport-smtp error: {0}")]
    FailedLettreTransportSMTP(#[from] lettre::transport::smtp::Error),
}

// Crate "async_trait" упрощает объявление Pin<Box>, внутренних vtable у трейтов и разных lifetime.
// Укажем сразу поддержку "Send + Sync", чтоб не писать подобное в других местах.
#[async_trait]
pub trait EmailSender: Send + Sync {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), EmailError>;
}

#[async_trait]
pub trait ConfirmationCodeStorer: Send + Sync {
    fn store(&self, email: &str, code: &str);
    fn take(&self, email: &str) -> String;
}

pub struct ConfirmationCodeStore {}

#[async_trait]
impl ConfirmationCodeStorer for ConfirmationCodeStore {
    fn store(&self, _email: &str, _code: &str) {}
    fn take(&self, _email: &str) -> String {
        String::new()
    }
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

#[async_trait]
impl EmailSender for Email {
    fn send(&self, ref_to: &str, ref_subject: &str, ref_body: &str) -> Result<(), EmailError> {
        let (local_from_email, domain_from_email) =
            self.from_email.split_once('@').ok_or_else(|| {
                EmailError::InvalidEmailAddress("invalid email: missing @ from 'from'".to_string())
            })?;
        let address_from_email = Address::new(local_from_email, domain_from_email)?;
        let mailbox_from = Mailbox::new(Some(self.from_name.clone()), address_from_email);
        let (local_to, domain_to) = ref_to.split_once('@').ok_or_else(|| {
            EmailError::InvalidEmailAddress("invalid email: missing @ from 'to'".to_string())
        })?;
        let address_to = Address::new(local_to, domain_to)?;
        let mailbox_to = Mailbox::new(None, address_to);
        let email = Message::builder()
            .from(mailbox_from)
            // .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to(mailbox_to)
            .subject(ref_subject)
            .header(ContentType::TEXT_HTML)
            .body(ref_body.to_string())?;
        let smtp_transport_builder = SmtpTransport::starttls_relay(self.host.as_str())?;

        smtp_transport_builder
            .timeout(Some(self.timeout))
            .credentials(Credentials::new(self.login.clone(), self.pass.clone()))
            .build()
            .send(&email)?;

        Ok(())
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
