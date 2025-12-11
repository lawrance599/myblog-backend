use std::{ops::Deref, sync::Arc};

use config::Config;
use dotenv::dotenv;
use serde::Deserialize;
use tracing::event;
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
    pub host: String,
    pub secret: String,
    pub jwt_expiration_min: i64,
    pub run_migrations: bool,
    pub rust_log: String,
    pub save_dir: String,
    pub migrate_dir: String,
}
impl AppConfig {
    pub fn new() -> Self {
        dotenv().ok();
        let config = Config::builder()
            .add_source(config::File::with_name("default.toml"))
            .add_source(config::File::with_name("local.toml").required(false))
            .add_source(config::Environment::default())
            .build();
        match config {
            Ok(config) => {
                let config = config.try_deserialize::<AppConfig>().unwrap();
                event!(tracing::Level::INFO, "Config loaded successfully");
                config
            }
            Err(err) => {
                panic!("Error loading config: {}", err);
            }
        }
    }

    pub fn get_run_migrations(&self) -> bool {
        self.run_migrations
    }
    pub fn get_database_url(&self) -> &str {
        self.database_url.as_str()
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
    pub fn get_host(&self) -> &str {
        self.host.as_str()
    }

    pub fn get_listener_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
    pub fn get_secret(&self) -> &str {
        self.secret.as_str()
    }
    pub fn get_jwt_expiration(&self) -> i64 {
        self.jwt_expiration_min
    }
    pub fn get_log_level(&self) -> &str {
        self.rust_log.as_str()
    }
    pub fn get_save_dir(&self) -> &str {
        &self.save_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_config() {
        let config = AppConfig::new();
        assert_eq!(config.get_port(), 8080);
        assert_eq!(config.get_host(), "0.0.0.0");
        assert_eq!(config.get_listener_addr(), "0.0.0.0:8080");
        assert_eq!(config.get_secret(), "secret");
        assert_eq!(config.get_jwt_expiration(), 86400);
        assert_eq!(config.get_run_migrations(), true);
        assert!(config.get_database_url().contains("postgres"));
        assert_eq!(config.get_log_level(), "info");
    }
}
