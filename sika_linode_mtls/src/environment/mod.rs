use serde::Deserialize;
use config::ConfigError;
use dotenv::dotenv;

#[derive(Deserialize, Clone)]
pub struct EnvConfig {
    pub server_cert: String,
    pub server_key: String,
    pub shopify_mtls: String,
    pub server_domain: String,
    pub mysql_root_password: String,
    pub mysql_database: String,
    pub mysql_user: String,
    pub mysql_password: String,
    pub mysql_port: String,
}

impl EnvConfig {
    pub fn env_variables() -> Result<Self, ConfigError>{
        dotenv().ok();
        let cf = config::Config::builder()
        .add_source(config::Environment::default())
        .build()?;
        cf.try_deserialize()
    }
}