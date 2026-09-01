use lettre::{
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
    {Address, Message, SmtpTransport, Transport},
};
use std::fmt;
use std::time::Duration;

// EmailSender. Трейт для подмены (прод, тест). Сразу добавим ограничения
// (Clone + Send + Sync + 'static), чтоб их не добавлять потом везде. "'static" - для Router.
pub trait EmailSender: Clone + Send + Sync + 'static {
    fn send(&self, to: String, subject: String, body: String) -> Result<(), EmailError>;
}

#[derive(Debug)]
pub enum EmailError {
    Common(String),
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailError::Common(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Clone)] // из-за usecase-auth
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
        host: String,
        login: String,
        pass: String,
        from_email: String,
        from_name: String,
        timeout: Duration,
    ) -> Self {
        Self {
            host,
            login,
            pass,
            from_email,
            from_name,
            timeout,
        }
    }
}

impl EmailSender for Email {
    fn send(&self, to: String, subject: String, body: String) -> Result<(), EmailError> {
        let (local_from_email, domain_from_email) = self.from_email.split_once('@').ok_or(
            EmailError::Common("invalid email: missing @ from 'from'".to_string()),
        )?;
        let address_from_email =
            Address::new(local_from_email, domain_from_email).map_err(|e| {
                EmailError::Common(format!("failed to create address from 'from': {e}"))
            })?;
        let mailbox_from = Mailbox::new(Some(self.from_name.clone()), address_from_email);
        let (local_to, domain_to) = to.split_once('@').ok_or(EmailError::Common(
            "invalid email: missing @ from 'to'".to_string(),
        ))?;
        let address_to = Address::new(local_to, domain_to)
            .map_err(|e| EmailError::Common(format!("failed to create address from 'to': {e}")))?;
        let mailbox_to = Mailbox::new(None, address_to);
        let email = Message::builder()
            .from(mailbox_from)
            // .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to(mailbox_to)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| EmailError::Common(format!("failed to create body: {e}")))?;
        let _resp = SmtpTransport::starttls_relay(self.host.as_str())
            .map_err(|e| EmailError::Common(format!("failed to create smtp-transport: {e}")))?
            .timeout(Some(self.timeout))
            .credentials(Credentials::new(self.login.clone(), self.pass.clone()))
            .build()
            .send(&email)
            .map_err(|e| EmailError::Common(format!("failed to send: {e}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_send() {
        let result = Email::new(
            "smtp.yandex.ru".to_string(),
            "support@altair.uz".to_string(),
            "".to_string(),
            "support@altair.uz".to_string(),
            "support".to_string(),
            Duration::from_secs(3),
        )
        .send(
            "volodya-nrg@mail.ru".to_string(),
            "my subj".to_string(),
            "my <strong>body</strong>".to_string(),
        );
        assert!(result.is_ok());
    }
}
