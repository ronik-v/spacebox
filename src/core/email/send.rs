use anyhow::Result;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use lettre::error::Error;
use lettre::transport::smtp::authentication::Credentials;
use std::error::Error as StdError;

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct EmailSender<'a> {
    cfg: &'a AppConfig,
    email_recipient: &'a String,
    email_subject: String,
    email_body: String
}

impl <'a>EmailSender<'a> {
    pub fn new(cfg: &'a AppConfig, email_recipient: &'a String, email_subject: String, email_body: String) -> Self {
        Self { cfg, email_recipient, email_subject, email_body }
    }

    pub async fn send(&self) -> Result<()> {
        let creds = Credentials::new(self.cfg.email_sender.to_string(), self.cfg.email_sender_password.to_string());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(self.cfg.email_smtp_server.as_str())
            .unwrap()
            .credentials(creds)
            .build();

        let message = self.build_letter();
        mailer
            .send(message.ok().unwrap())
            .await
            .map(|_message_id| ())
            .map_err(|e| Box::new(e) as Box<dyn StdError + Send + Sync>)
            .expect("Email doesnt send");

        Ok(())
    }

    fn build_letter(&self) -> Result<Message, Error> {
        let message = Message::builder()
            .from(self.cfg.email_sender.parse().unwrap())
            .to(self.cfg.email_smtp_server.parse().unwrap())
            .subject(self.email_subject.to_string())
            .body(self.email_body.to_string());

        message
    }
}