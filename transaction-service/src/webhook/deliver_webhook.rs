use anyhow::Result;
use sqlx::{postgres::types::PgInterval, PgPool};
use tracing::{error, info};
use uuid::Uuid;

use crate::{constants::ERR_MAX_RETRIES, enums::WebhookStatus, webhook::signer::sign_hmac_sha256};

fn backoff_delay(attempt: i32) -> PgInterval {
    let seconds = 2_i64.pow(attempt as u32);

    PgInterval {
        months: 0,
        days: 0,
        microseconds: seconds * 1_000_000,
    }
}

pub async fn deliver_webhook(pool: PgPool, event_id: Uuid, max_retries: i32) -> Result<()> {
    let event = sqlx::query!(
        r#"
        SELECT
            e.id,
            e.payload,
            e.attempt_count,
            w.url,
            w.secret
        FROM webhook_events e
        JOIN webhook_endpoints w ON w.id = e.endpoint_id
        WHERE e.id = $1
          AND e.status = $2
        "#,
        event_id,
        WebhookStatus::Processing.as_str()
    )
    .fetch_optional(&pool)
    .await?;

    let Some(event) = event else {
        info!(event_id = %event_id, "webhook event not found or not in processing state");
        return Ok(());
    };

    let body = serde_json::to_vec(&event.payload).map_err(|e| {
        error!("Webhook payload serialization failed: {}", e);
        e
    })?;

    let signature = match sign_hmac_sha256(&event.secret, &body) {
        Ok(sig) => sig,
        Err(e) => {
            error!("Webhook signature generation failed: {}", e);
            return Ok(());
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| {
            error!("Reqwest client build failed: {}", e);
            e
        })?;

    let res = client
        .post(&event.url)
        .header("X-Signature", signature)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await;

    match res {
        Ok(r) if r.status().is_success() => {
            sqlx::query!(
                r#"
                UPDATE webhook_events
                SET status = $2
                WHERE id = $1
                "#,
                event.id,
                WebhookStatus::Delivered.as_str()
            )
            .execute(&pool)
            .await?;
        }

        Ok(resp) => {
            let status = resp.status().as_u16();
            let attempts = event.attempt_count + 1;
            let retryable = status >= 500 || status == 429;

            if !retryable {
                sqlx::query!(
                    r#"
                    UPDATE webhook_events
                    SET status = $2,
                        attempt_count = $3,
                        last_error = $4
                    WHERE id = $1
                    "#,
                    event.id,
                    WebhookStatus::Failed.as_str(),
                    attempts,
                    format!("HTTP {}", status)
                )
                .execute(&pool)
                .await?;
                return Ok(());
            }

            if attempts >= max_retries {
                sqlx::query!(
                    r#"
                    UPDATE webhook_events
                    SET status = $2,
                        attempt_count = $3,
                        last_error = $4
                    WHERE id = $1
                    "#,
                    event.id,
                    WebhookStatus::Failed.as_str(),
                    attempts,
                    ERR_MAX_RETRIES
                )
                .execute(&pool)
                .await?;
            } else {
                let delay = backoff_delay(attempts);
                info!(
                    event_id = %event.id,
                    attempts,
                    "scheduling webhook retry after error"
                );

                sqlx::query!(
                    r#"
                    UPDATE webhook_events
                    SET status = $2,
                        attempt_count = $3,
                        next_retry_at = now() + $4::interval,
                        last_error = $5
                    WHERE id = $1
                    "#,
                    event.id,
                    WebhookStatus::Pending.as_str(),
                    attempts,
                    delay,
                    format!("HTTP {}", status)
                )
                .execute(&pool)
                .await?;
            }
        }

        Err(err) => {
            let attempts = event.attempt_count + 1;
            error!(
                event_id = %event.id,
                error = %err,
                attempt = attempts,
                "webhook request failed"
            );

            if attempts >= max_retries {
                sqlx::query!(
                    r#"
                    UPDATE webhook_events
                    SET status = $2,
                        attempt_count = $3,
                        last_error = $4
                    WHERE id = $1
                    "#,
                    event.id,
                    WebhookStatus::Failed.as_str(),
                    attempts,
                    ERR_MAX_RETRIES
                )
                .execute(&pool)
                .await?;
            } else {
                let delay = backoff_delay(attempts);

                sqlx::query!(
                    r#"
                    UPDATE webhook_events
                    SET status = $2,
                        attempt_count = $3,
                        next_retry_at = now() + $4::interval,
                        last_error = $5
                    WHERE id = $1
                    "#,
                    event.id,
                    WebhookStatus::Pending.as_str(),
                    attempts,
                    delay,
                    err.to_string()
                )
                .execute(&pool)
                .await?;
            }
        }
    }

    Ok(())
}
