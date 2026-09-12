use lettre::{
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
    {Address, Message, SmtpTransport, Transport},
};
use std::time::Duration;

// EmailSender. Трейт для подмены (прод, тест). Сразу добавим ограничения
// (Clone + Send + Sync + 'static), чтоб их не добавлять потом везде. "'static" - для Router.
pub trait EmailSender: Clone + Send + Sync + 'static {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), String>;
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
        host: &str,
        login: &str,
        pass: &str,
        from_email: &str,
        from_name: &str,
        timeout: Duration,
    ) -> Self {
        Self {
            host: host.to_string(),
            login: login.to_string(),
            pass: pass.to_string(),
            from_email: from_email.to_string(),
            from_name: from_name.to_string(),
            timeout,
        }
    }
}

impl EmailSender for Email {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let (local_from_email, domain_from_email) = self
            .from_email
            .split_once('@')
            .ok_or("invalid email: missing @ from 'from'".to_string())?;
        let address_from_email = Address::new(local_from_email, domain_from_email)
            .map_err(|e| format!("failed to create address from 'from': {e}"))?;
        let mailbox_from = Mailbox::new(Some(self.from_name.clone()), address_from_email);
        let (local_to, domain_to) = to
            .split_once('@')
            .ok_or("invalid email: missing @ from 'to'".to_string())?;
        let address_to = Address::new(local_to, domain_to)
            .map_err(|e| format!("failed to create address from 'to': {e}"))?;
        let mailbox_to = Mailbox::new(None, address_to);
        let email = Message::builder()
            .from(mailbox_from)
            // .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to(mailbox_to)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body.to_string())
            .map_err(|e| format!("failed to create body: {e}"))?;

        SmtpTransport::starttls_relay(self.host.as_str())
            .map_err(|e| format!("failed to create smtp-transport: {e}"))?
            .timeout(Some(self.timeout))
            .credentials(Credentials::new(self.login.clone(), self.pass.clone()))
            .build()
            .send(&email)
            .map_err(|e| format!("failed to send: {e}"))
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
            "",
            "support@altair.uz",
            "support",
            Duration::from_secs(3),
        )
        .send("volodya-nrg@mail.ru", "my subj", "my <strong>body</strong>");
        assert!(result.is_ok());
    }
}
