use actix_web::{web, HttpResponse};
use opentelemetry::{global, metrics::Counter, KeyValue};
use sqlx::PgPool;
use tracing::{error, info};

pub async fn health(pool: web::Data<PgPool>) -> HttpResponse {
    info!("health check called");

    let meter = global::meter("health");
    let counter: Counter<u64> = meter
        .u64_counter("health_checks_total")
        .with_description("Total number of health check requests")
        .init();

    match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => {
            counter.add(1, &[KeyValue::new("result", "ok")]);

            HttpResponse::Ok().json(serde_json::json!({
                "status": "ok"
            }))
        }
        Err(e) => {
            error!(error = %e, "health check failed: database unavailable");

            counter.add(1, &[KeyValue::new("result", "db_unavailable")]);

            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "db_unavailable"
            }))
        }
    }
}
