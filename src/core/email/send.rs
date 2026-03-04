use anyhow::{Context, Result};
use lettre::{
    message::{Mailbox, header::ContentType},
    AsyncSmtpTransport,
    AsyncTransport,
    Message,
    Tokio1Executor,
};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct EmailSender<'a> {
    cfg: &'a AppConfig,
    recipient_email: &'a str,
    subject: String,
    body: String,
}

impl<'a> EmailSender<'a> {
    pub fn new(
        cfg: &'a AppConfig,
        recipient_email: &'a str,
        subject: String,
        body: String,
    ) -> Self {
        Self {
            cfg,
            recipient_email,
            subject,
            body,
        }
    }

    pub async fn send(&self) -> Result<()> {
        let creds = Credentials::new(
            self.cfg.email_sender.clone(),
            self.cfg.email_sender_password.clone(),
        );

        let tls_params = TlsParameters::builder(self.cfg.email_smtp_server.clone())
            .build()
            .context("Ошибка создания параметров TLS (проверьте SNI / домен)".to_string())?;

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.cfg.email_smtp_server)
            .context("Не удалось создать SMTP relay (проверьте email_smtp_server в конфиге)".to_string())?
            .credentials(creds)
            .port(self.cfg.email_smtp_port.unwrap_or(465))
            .tls(Tls::Wrapper(tls_params))
            .build();

        let from: Mailbox = self.cfg.email_sender
            .parse()
            .with_context(|| format!("Невалидный формат адреса отправителя: {}", self.cfg.email_sender))?;

        let to: Mailbox = self.recipient_email
            .parse()
            .with_context(|| format!("Невалидный формат адреса получателя: {}", self.recipient_email))?;

        let message = Message::builder()
            .from(from)
            .to(to)
            .subject(self.subject.clone())
            .header(ContentType::TEXT_HTML)
            .body(self.body.clone())
            .context("Не удалось собрать письмо".to_string())?;
        mailer
            .send(message)
            .await
            .context("Не удалось отправить письмо".to_string())?;

        Ok(())
    }
}