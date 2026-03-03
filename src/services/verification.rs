use hex;
use std::sync::Arc;
use anyhow::Result;
use chrono::{Datelike, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use redis::cmd;
use sha2::{Digest, Sha256};

use crate::config::AppConfig;
use crate::core::email::constants::{EMAIL_HTML_BODY_TEMPLATE, EMAIL_SUBJECT_TEMPLATE, TTL_MINUTES};
use crate::core::email::send::EmailSender;

#[derive(Clone)]
pub struct RegistrationVerification<'a> {
    cfg: &'a AppConfig,
    email_recipient: String,
    verification_code: String,
    redis: Arc<ConnectionManager>,
}

impl <'a>RegistrationVerification<'a> {
    pub fn new(cfg: &'a AppConfig, email_recipient: String, verification_code: String, redis: Arc<ConnectionManager>) -> Self {
        Self { cfg, email_recipient, verification_code, redis }
    }

    pub async fn send_code_verification(&self) -> Result<()> {
        let year = Utc::now().year().to_string();
        let ver_subject = EMAIL_SUBJECT_TEMPLATE.replace("{{app_name}}", &self.cfg.app_name);
        let html_body_message = EMAIL_HTML_BODY_TEMPLATE
            .replace("{{app_name}}", &self.cfg.app_name)
            .replace("{{code}}", self.verification_code.as_str())
            .replace("{{ttl_minutes}}", &TTL_MINUTES.to_string())
            .replace("{{year}}", &year);

        let email_sender = EmailSender::new(
            &self.cfg, &self.email_recipient, ver_subject, html_body_message
        );

        email_sender.send().await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    pub async fn set_code_verification_in_redis(&self) -> Result<()> {
        let mut conn = (*self.redis).clone();
        let code_lifetime = (TTL_MINUTES * 60) as u64;
        let key = self.make_key();

        conn.set_ex::<_, _, u64>(key, self.verification_code.to_string(), code_lifetime).await?;

        Ok(())
    }

    pub async fn is_correct_verification_code(&self) -> Result<bool> {
        let mut conn = (*self.redis).clone();
        let key = self.make_key();
        let stored: Option<String> = cmd("GETDEL").arg(&key).query_async(&mut conn).await?;

        Ok(
            match stored {
             Some(v) => v == self.verification_code,
             None => false,
            }
        )
    }

    fn make_key(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.email_recipient.as_bytes());
        let hash = hasher.finalize();
        let hex = hex::encode(hash);
        format!("{}{}", "verify:email:", hex)
    }
}