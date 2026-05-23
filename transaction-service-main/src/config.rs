use std::env;

use crate::constants::*;

#[derive(Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub log: LogConfig,
    pub webhook: WebhookConfig,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
}

#[derive(Clone)]
pub struct LogConfig {
    pub level: String,
    pub service_name: String,
    pub otel_endpoint: String,
}

#[derive(Clone)]
pub struct WebhookConfig {
    pub batch_size: i64,
    pub max_retries: i32,
    pub processing_timeout_secs: i64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            server: ServerConfig {
                host: env::var(ENV_SERVER_HOST).unwrap_or_else(|_| DEFAULT_SERVER_HOST.to_string()),
                port: env::var(ENV_SERVER_PORT)
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(DEFAULT_SERVER_PORT),
            },
            database: DatabaseConfig {
                database_url: env::var(ENV_DATABASE_URL)?,
            },
            log: LogConfig {
                level: env::var(ENV_LOG_LEVEL).unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string()),
                service_name: env::var(ENV_SERVICE_NAME)
                    .unwrap_or_else(|_| DEFAULT_SERVICE_NAME.to_string()),
                otel_endpoint: env::var(ENV_OTEL_EXPORTER_ENDPOINT)
                    .unwrap_or_else(|_| DEFAULT_OTEL_EXPORTER_ENDPOINT.to_string()),
            },
            webhook: WebhookConfig {
                batch_size: env::var(ENV_WEBHOOK_BATCH_SIZE)
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(DEFAULT_WEBHOOK_BATCH_SIZE),
                max_retries: env::var(ENV_WEBHOOK_MAX_RETRIES)
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(DEFAULT_WEBHOOK_MAX_RETRIES),
                processing_timeout_secs: env::var(ENV_WEBHOOK_PROCESSING_TIMEOUT_SECS)
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(DEFAULT_WEBHOOK_PROCESSING_TIMEOUT_SECS),
            },
        })
    }
}
