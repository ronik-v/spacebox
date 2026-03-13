use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub app_port: u16,
    pub app_host: String,
    pub app_name: String,
    pub database_uri: String,
    pub database_min_connections: u32,
    pub database_max_connections: u32,
    pub redis_uri: String,
    pub password_salt: String,
    pub email_sender: String,
    pub email_sender_password: String,
    pub email_smtp_server: String,
    pub email_smtp_port: Option<u16>,
    pub root_dir: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        envy::prefixed("").from_env::<AppConfig>().expect("Fall to load .env")
    }
}