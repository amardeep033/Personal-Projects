use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use opentelemetry::{global, metrics::Counter, KeyValue};
use sqlx::PgPool;
use std::io;
use tokio::signal;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;

use crate::{
    config::AppConfig,
    constants::SERVICE,
    log::{init_logger, shutdown_tracer},
    middleware::rate_limiter::RateLimiter,
    webhook::retry_worker::webhook_retry_worker,
};

mod api;
mod config;
mod constants;
mod enums;
mod log;
mod middleware;
mod models;
mod webhook;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = AppConfig::from_env().map_err(|e| {
        error!("config load failed: {}", e);
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    init_logger(&config.log);

    let meter = global::meter(config.log.clone().service_name);
    let startup_counter: Counter<u64> = meter
        .u64_counter("service_startups_total")
        .with_description("Number of times the service has started")
        .init();

    startup_counter.add(
        1,
        &[KeyValue::new(SERVICE, config.log.clone().service_name)],
    );

    let pool = PgPool::connect(&config.database.database_url)
        .await
        .map_err(|e| {
            error!("DB connection failed: {}", e);
            io::Error::new(io::ErrorKind::Other, e)
        })?;

    let webhook_cfg = config.webhook.clone();

    tokio::spawn(webhook_retry_worker(pool.clone(), webhook_cfg));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(pool.clone()))
            .configure(api::handlers)
            .service(
                web::scope("")
                    .wrap(RateLimiter::new(pool.clone()))
                    .wrap(middleware::auth::ApiKeyAuth)
                    .configure(api::protected_handlers),
            )
    })
    .bind((config.server.host.as_str(), config.server.port))?
    .run();

    let handle = server.handle();

    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");

        info!("shutdown signal received");

        let meter = global::meter(config.log.clone().service_name);
        let shutdown_counter: Counter<u64> = meter
            .u64_counter("service_shutdowns_total")
            .with_description("Number of graceful shutdowns")
            .init();

        shutdown_counter.add(1, &[KeyValue::new(SERVICE, config.log.service_name)]);

        handle.stop(true).await;
        shutdown_tracer();
        info!("graceful shutdown complete");
    });

    server.await
}
