use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::enums::WebhookStatus;

pub async fn enqueue_transaction_webhooks(
    pool: &PgPool,
    business_id: Uuid,
    event_type: &str,
    payload: serde_json::Value,
) {
    let meter = global::meter("webhooks");
    let enqueue_counter: Counter<u64> = meter
        .u64_counter("webhook_events_enqueued_total")
        .with_description("Total number of webhook events enqueued")
        .init();

    let endpoints = match sqlx::query!(
        r#"
        SELECT id
        FROM webhook_endpoints
        WHERE business_id = $1 AND is_active = true
        "#,
        business_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("failed to fetch webhook endpoints: {}", e);
            enqueue_counter.add(1, &[KeyValue::new("result", "error")]);
            return;
        }
    };

    for ep in endpoints {
        let event_id = Uuid::new_v4();

        match sqlx::query!(
            r#"
            INSERT INTO webhook_events (
                id, business_id, endpoint_id,
                event_type, payload, status
            )
            VALUES ($1,$2,$3,$4,$5,$6)
            "#,
            event_id,
            business_id,
            ep.id,
            event_type,
            payload,
            WebhookStatus::Pending.as_str()
        )
        .execute(pool)
        .await
        {
            Ok(_) => {
                enqueue_counter.add(1, &[KeyValue::new("result", "success")]);
            }
            Err(e) => {
                error!("failed to enqueue webhook event {}: {}", event_id, e);
                enqueue_counter.add(1, &[KeyValue::new("result", "error")]);
            }
        }
    }
}
