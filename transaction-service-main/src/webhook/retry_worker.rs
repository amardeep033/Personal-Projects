use sqlx::PgPool;

use crate::{
    config::WebhookConfig, enums::WebhookStatus, webhook::deliver_webhook::deliver_webhook,
};
use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use tracing::{error, info};

pub async fn webhook_retry_worker(pool: PgPool, cfg: WebhookConfig) {
    let meter = global::meter("webhooks.worker");

    let batch_counter: Counter<u64> = meter
        .u64_counter("webhook_batches_processed_total")
        .with_description("Total number of webhook batches processed")
        .init();

    let delivery_counter: Counter<u64> = meter
        .u64_counter("webhook_delivery_attempts_total")
        .with_description("Total number of webhook delivery attempts")
        .init();

    loop {
        if let Err(e) = sqlx::query!(
            r#"
            UPDATE webhook_events
            SET status = $1,
                processing_started_at = NULL
            WHERE status = $2
              AND processing_started_at < now() - ($3 || ' seconds')::interval
            "#,
            WebhookStatus::Pending.as_str(),
            WebhookStatus::Processing.as_str(),
            cfg.processing_timeout_secs.to_string()
        )
        .execute(&pool)
        .await
        {
            error!("failed to reset stuck webhook events: {}", e);
        }

        let events = match sqlx::query!(
            r#"
            UPDATE webhook_events
            SET status = $1,
                processing_started_at = now()
            WHERE id IN (
                SELECT id
                FROM webhook_events
                WHERE status = $2
                  AND (
                    next_retry_at IS NULL
                    OR next_retry_at <= now()
                  )
                LIMIT $3
            )
            RETURNING id
            "#,
            WebhookStatus::Processing.as_str(),
            WebhookStatus::Pending.as_str(),
            cfg.batch_size
        )
        .fetch_all(&pool)
        .await
        {
            Ok(v) => {
                info!(batch_size = v.len(), "webhook batch fetched");
                batch_counter.add(1, &[KeyValue::new("result", "success")]);
                v
            }
            Err(e) => {
                error!("failed to fetch webhook events: {}", e);
                batch_counter.add(1, &[KeyValue::new("result", "error")]);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        for e in events {
            let pool = pool.clone();
            let max_retries = cfg.max_retries;
            let delivery_counter = delivery_counter.clone();

            tokio::spawn(async move {
                match deliver_webhook(pool, e.id, max_retries).await {
                    Ok(_) => {
                        delivery_counter.add(1, &[KeyValue::new("result", "success")]);
                    }
                    Err(err) => {
                        error!("webhook delivery failed: {}", err);
                        delivery_counter.add(1, &[KeyValue::new("result", "failure")]);
                    }
                }
            });
        }

        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}
