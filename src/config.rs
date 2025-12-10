use std::sync::{Arc, LazyLock};
use config::Config;
use tracing::event;
use serde::Deserialize;
use dotenv::dotenv;
pub static CONFIG: LazyLock<Arc<AppConfig>> = LazyLock::new(|| {
    dotenv().ok();
    let config = Config::builder()
        .add_source(config::File::with_name("default.toml"))
        .add_source(
            config::File::with_name("local.toml")
                .required(false)
        )
        .add_source(
            config::Environment::with_prefix("APP")
        )
        .build();
    match config {
        Ok(config) => {
            let config = config.try_deserialize::<AppConfig>().unwrap();
            event!(
                tracing::Level::INFO,
                "Config loaded successfully"
            );
            Arc::new(config)
        }
        Err(err) => {
            panic!("Error loading config: {}", err);
        }
    }

});

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    database_url: String,
    port: u16,
    host: String,
    secret: String,
    jwt_expiration_min: i64,
    run_migrations: bool,
    log_level: String,
}
impl AppConfig {
    pub fn new() -> Arc<Self> {
        (*CONFIG).clone()
    }
    
    pub fn get_run_migrations(&self) -> bool {
        self.run_migrations
    }
    pub fn get_database_url(&self) -> String {
        self.database_url.clone()
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
    pub fn get_host(&self) -> String {
        self.host.clone()
    }

    pub fn get_listener_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
    pub fn get_secret(&self) -> String {
        self.secret.clone()
    }
    pub fn get_jwt_expiration(&self) -> i64 {
        self.jwt_expiration_min
    }
    pub fn get_log_level(&self) -> String {
        self.log_level.clone()
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