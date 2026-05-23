use actix_web::HttpMessage;
use actix_web::{web, HttpRequest, HttpResponse};
use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

use crate::middleware::auth::BusinessId;
use crate::models::*;

pub async fn create_account(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateAccountRequest>,
) -> HttpResponse {
    let meter = global::meter("accounts");
    let counter: Counter<u64> = meter
        .u64_counter("accounts_created_total")
        .with_description("Total number of accounts created")
        .init();

    let business_id = match req.extensions().get::<BusinessId>() {
        Some(v) => v.0,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let account_id = Uuid::new_v4();

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO accounts (id, business_id, name, balance)
        VALUES ($1, $2, $3, 0)
        "#,
    )
    .bind(account_id)
    .bind(business_id)
    .bind(&body.name)
    .execute(pool.get_ref())
    .await
    {
        error!(error = %e, "account creation failed");
        counter.add(1, &[KeyValue::new("result", "error")]);
        return HttpResponse::InternalServerError().finish();
    }

    info!(
        account_id = %account_id,
        business_id = %business_id,
        "account created"
    );
    counter.add(1, &[KeyValue::new("result", "success")]);

    HttpResponse::Created().json(AccountResponse {
        account_id,
        balance: 0,
    })
}
