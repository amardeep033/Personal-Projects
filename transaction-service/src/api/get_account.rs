use actix_web::HttpMessage;
use actix_web::{web, HttpRequest, HttpResponse};
use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use sqlx::{PgPool, Row};
use tracing::{error, info};
use uuid::Uuid;

use crate::middleware::auth::BusinessId;
use crate::models::*;

pub async fn get_account(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let meter = global::meter("accounts");
    let counter: Counter<u64> = meter
        .u64_counter("account_get_requests_total")
        .with_description("Total number of get account requests")
        .init();

    let business_id = match req.extensions().get::<BusinessId>() {
        Some(v) => v.0,
        None => {
            counter.add(1, &[KeyValue::new("result", "unauthorized")]);
            return HttpResponse::Unauthorized().finish();
        }
    };

    let row = match sqlx::query(
        r#"
        SELECT id, balance
        FROM accounts
        WHERE id = $1 AND business_id = $2
        "#,
    )
    .bind(*path)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(v) => v,
        Err(e) => {
            error!(error = %e, "account fetch failed");
            counter.add(1, &[KeyValue::new("result", "error")]);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match row {
        Some(r) => {
            let account_id: Uuid = r.get("id");
            let balance: i64 = r.get("balance");

            info!(
                account_id = %account_id,
                business_id = %business_id,
                "account fetched"
            );

            counter.add(1, &[KeyValue::new("result", "ok")]);

            HttpResponse::Ok().json(AccountResponse {
                account_id,
                balance,
            })
        }
        None => {
            counter.add(1, &[KeyValue::new("result", "not_found")]);
            HttpResponse::NotFound().finish()
        }
    }
}
