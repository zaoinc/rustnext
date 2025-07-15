use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use toml;
use log::{info, warn, error};
use once_cell::sync::OnceCell;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub features: FeatureConfig,
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub session_timeout: u64,
    pub bcrypt_cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub compression: bool,
    pub metrics: bool,
    pub hot_reload: bool,
    pub logging: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                workers: num_cpus::get(),
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/rustnext".to_string(),
                max_connections: 10,
                timeout: 30,
            },
            auth: AuthConfig {
                jwt_secret: "your-secret-key".to_string(),
                session_timeout: 3600,
                bcrypt_cost: 12,
            },
            features: FeatureConfig {
                compression: true,
                metrics: false,
                hot_reload: false,
                logging: true,
            },
            custom: HashMap::new(),
        }
    }
}

impl Config {
    pub fn load(file_path: Option<&str>) -> Self {
        let mut config = Config::default();

        if let Some(path) = file_path {
            match fs::read_to_string(path) {
                Ok(contents) => {
                    match toml::from_str(&contents) {
                        Ok(file_config) => {
                            info!("Configuration loaded from {}", path);
                            config = file_config;
                        },
                        Err(e) => {
                            error!("Failed to parse config file {}: {}", path, e);
                            warn!("Using default configuration.");
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to read config file {}: {}", path, e);
                    warn!("Using default configuration.");
                }
            }
        } else {
            info!("No config file specified, using default configuration.");
        }
        
        // Override with environment variables
        if let Ok(host) = env::var("RUSTNEXT_HOST") {
            info!("Overriding server host with RUSTNEXT_HOST={}", host);
            config.server.host = host;
        }
        if let Ok(port) = env::var("RUSTNEXT_PORT") {
            if let Ok(port_num) = port.parse() {
                info!("Overriding server port with RUSTNEXT_PORT={}", port_num);
                config.server.port = port_num;
            } else {
                warn!("Invalid RUSTNEXT_PORT value: {}", port);
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            info!("Overriding database URL with DATABASE_URL");
            config.database.url = db_url;
        }
        
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            info!("Overriding JWT secret with JWT_SECRET");
            config.auth.jwt_secret = jwt_secret;
        }
        
        config.features.compression = env::var("ENABLE_COMPRESSION").map_or(config.features.compression, |s| s == "true");
        config.features.metrics = env::var("ENABLE_METRICS").map_or(config.features.metrics, |s| s == "true");
        config.features.hot_reload = env::var("ENABLE_HOT_RELOAD").map_or(config.features.hot_reload, |s| s == "true");
        config.features.logging = env::var("ENABLE_LOGGING").map_or(config.features.logging, |s| s == "true");

        config
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.custom.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.custom.insert(key.to_string(), value.to_string());
    }
}

static GLOBAL_CONFIG: OnceCell<Config> = OnceCell::new();

pub fn get_config() -> &'static Config {
    GLOBAL_CONFIG.get_or_init(|| {
        Config::load(None)
    })
}

pub fn init_config(config: Config) {
    if GLOBAL_CONFIG.set(config).is_err() {
        warn!("Config already initialized, ignoring new initialization.");
    }
}