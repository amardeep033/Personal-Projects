use actix_web::HttpMessage;
use actix_web::{web, HttpRequest, HttpResponse};
use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use sqlx::{PgPool, Row};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    constants::{EVENT_TRANSACTION_POSTED, HEADER_IDEMPOTENCY_KEY, TXN_STATUS_POSTED},
    enums::TransactionType,
    middleware::auth::{ApiKeyId, BusinessId},
    models::*,
    webhook::dispatcher::enqueue_transaction_webhooks,
};

pub async fn create_transaction(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateTransactionRequest>,
) -> HttpResponse {
    let meter = global::meter("transaction-service.transactions");
    let txn_counter: Counter<u64> = meter
        .u64_counter("transactions_total")
        .with_description("Total number of transactions")
        .init();

    let business_id = match req.extensions().get::<BusinessId>() {
        Some(v) => v.0,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let api_key_id = match req.extensions().get::<ApiKeyId>() {
        Some(v) => v.0,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let idem_key = match req
        .headers()
        .get(HEADER_IDEMPOTENCY_KEY)
        .and_then(|v| v.to_str().ok())
    {
        Some(v) => v.to_string(),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "IDEMPOTENCY_KEY_REQUIRED"
            }))
        }
    };

    if body.amount <= 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_AMOUNT"
        }));
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("tx begin failed: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let existing = match sqlx::query(
        r#"
        SELECT id, status
        FROM transactions
        WHERE api_key_id = $1 AND idempotency_key = $2
        "#,
    )
    .bind(api_key_id)
    .bind(&idem_key)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("idempotency check failed: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let Some(row) = existing {
        let txn_id: Uuid = row.get("id");
        let status: String = row.get("status");
        let _ = tx.rollback().await;

        info!(
            transaction_id = %txn_id,
            transaction_type = body.transc_type.as_str(),
            business_id = %business_id,
            "idempotent transfer"
        );

        txn_counter.add(
            1,
            &[
                KeyValue::new("type", body.transc_type.as_str()),
                KeyValue::new("result", "idempotent"),
            ],
        );

        return HttpResponse::Created().json(serde_json::json!({
            "transaction_id": txn_id,
            "status": status,
            "idempotent": true
        }));
    }

    let transaction_id = Uuid::new_v4();

    if matches!(
        body.transc_type,
        TransactionType::Credit | TransactionType::Debit
    ) {
        let account_id = match body.account_id {
            Some(v) => v,
            None => {
                let _ = tx.rollback().await;
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "ACCOUNT_ID_REQUIRED"
                }));
            }
        };

        let row = match sqlx::query(
            r#"
            SELECT balance
            FROM accounts
            WHERE id = $1 AND business_id = $2
            FOR UPDATE
            "#,
        )
        .bind(account_id)
        .bind(business_id)
        .fetch_optional(&mut *tx)
        .await
        {
            Ok(Some(r)) => r,
            Ok(None) => {
                let _ = tx.rollback().await;
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "ACCOUNT_NOT_FOUND"
                }));
            }
            Err(e) => {
                error!("account fetch failed: {}", e);
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError().finish();
            }
        };

        let balance_before: i64 = row.get("balance");

        let balance_after = match body.transc_type {
            TransactionType::Credit => balance_before + body.amount,
            TransactionType::Debit => {
                if balance_before < body.amount {
                    let _ = tx.rollback().await;
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "INSUFFICIENT_FUNDS"
                    }));
                }
                balance_before - body.amount
            }
            _ => unreachable!(),
        };

        if let Err(e) = sqlx::query("UPDATE accounts SET balance = $1 WHERE id = $2")
            .bind(balance_after)
            .bind(account_id)
            .execute(&mut *tx)
            .await
        {
            error!("balance update failed: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO transactions (
                id, api_key_id, business_id,
                from_account_id, to_account_id,
                type, amount,
                balance_before, balance_after,
                idempotency_key, status
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            "#,
        )
        .bind(transaction_id)
        .bind(api_key_id)
        .bind(business_id)
        .bind(if body.transc_type == TransactionType::Debit {
            Some(account_id)
        } else {
            None
        })
        .bind(if body.transc_type == TransactionType::Credit {
            Some(account_id)
        } else {
            None
        })
        .bind(body.transc_type.as_str())
        .bind(body.amount)
        .bind(balance_before)
        .bind(balance_after)
        .bind(&idem_key)
        .bind(TXN_STATUS_POSTED)
        .execute(&mut *tx)
        .await
        {
            error!("transaction insert failed: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }

        if let Err(e) = tx.commit().await {
            error!("commit failed: {}", e);
            return HttpResponse::InternalServerError().finish();
        }

        enqueue_transaction_webhooks(
            pool.get_ref(),
            business_id,
            EVENT_TRANSACTION_POSTED,
            serde_json::json!({
                "transaction_id": transaction_id,
                "type": body.transc_type.as_str(),
                "account_id": account_id,
                "amount": body.amount
            }),
        )
        .await;

        info!(
            transaction_id = %transaction_id,
            transaction_type = body.transc_type.as_str(),
            business_id = %business_id,
            "transaction completed"
        );

        txn_counter.add(
            1,
            &[
                KeyValue::new("type", body.transc_type.as_str()),
                KeyValue::new("result", "success"),
            ],
        );

        return HttpResponse::Created().json(serde_json::json!({
            "transaction_id": transaction_id,
            "status": TXN_STATUS_POSTED,
            "balance_before": balance_before,
            "balance_after": balance_after
        }));
    }

    if body.transc_type == TransactionType::Transfer {
        let from_id = match body.from_account_id {
            Some(v) => v,
            None => {
                let _ = tx.rollback().await;
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "FROM_ACCOUNT_REQUIRED"
                }));
            }
        };

        let to_id = match body.to_account_id {
            Some(v) => v,
            None => {
                let _ = tx.rollback().await;
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "TO_ACCOUNT_REQUIRED"
                }));
            }
        };

        if from_id == to_id {
            let _ = tx.rollback().await;
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "SAME_ACCOUNT_TRANSFER"
            }));
        }

        let mut ids = vec![from_id, to_id];
        ids.sort();

        let rows = match sqlx::query(
            r#"
        SELECT id, balance
        FROM accounts
        WHERE id = ANY($1) AND business_id = $2
        FOR UPDATE
        "#,
        )
        .bind(&ids)
        .bind(business_id)
        .fetch_all(&mut *tx)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                error!("account lock failed: {}", e);
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError().finish();
            }
        };

        if rows.len() != 2 {
            let _ = tx.rollback().await;
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "ACCOUNT_NOT_FOUND"
            }));
        }

        let mut from_balance = None;

        for r in &rows {
            let id: Uuid = r.get("id");
            let bal: i64 = r.get("balance");

            if id == from_id {
                from_balance = Some(bal);
            }
        }

        let from_balance = match from_balance {
            Some(v) => v,
            None => {
                let _ = tx.rollback().await;
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "ACCOUNT_NOT_FOUND"
                }));
            }
        };

        if from_balance < body.amount {
            let _ = tx.rollback().await;
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "INSUFFICIENT_FUNDS"
            }));
        }

        if let Err(e) = sqlx::query("UPDATE accounts SET balance = balance - $1 WHERE id = $2")
            .bind(body.amount)
            .bind(from_id)
            .execute(&mut *tx)
            .await
        {
            error!("debit failed: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }

        if let Err(e) = sqlx::query("UPDATE accounts SET balance = balance + $1 WHERE id = $2")
            .bind(body.amount)
            .bind(to_id)
            .execute(&mut *tx)
            .await
        {
            error!("credit failed: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }

        if let Err(e) = sqlx::query(
            r#"
        INSERT INTO transactions (
            id, api_key_id, business_id,
            from_account_id, to_account_id,
            type, amount,
            balance_before, balance_after,
            idempotency_key, status
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        "#,
        )
        .bind(transaction_id)
        .bind(api_key_id)
        .bind(business_id)
        .bind(from_id)
        .bind(to_id)
        .bind(body.transc_type.as_str())
        .bind(body.amount)
        .bind(from_balance)
        .bind(from_balance - body.amount)
        .bind(&idem_key)
        .bind(TXN_STATUS_POSTED)
        .execute(&mut *tx)
        .await
        {
            error!("transaction insert failed: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }

        if let Err(e) = tx.commit().await {
            error!("commit failed: {}", e);
            return HttpResponse::InternalServerError().finish();
        }

        enqueue_transaction_webhooks(
            pool.get_ref(),
            business_id,
            EVENT_TRANSACTION_POSTED,
            serde_json::json!({
                "transaction_id": transaction_id,
                "type": body.transc_type.as_str(),
                "from_account_id": from_id,
                "to_account_id": to_id,
                "amount": body.amount,
                "status": TXN_STATUS_POSTED
            }),
        )
        .await;

        info!(
            transaction_id = %transaction_id,
            transaction_type = body.transc_type.as_str(),
            business_id = %business_id,
            "transfer completed"
        );

        txn_counter.add(
            1,
            &[
                KeyValue::new("type", body.transc_type.as_str()),
                KeyValue::new("result", "success"),
            ],
        );

        return HttpResponse::Created().json(serde_json::json!({
            "transaction_id": transaction_id,
            "status": TXN_STATUS_POSTED,
            "from_account": from_id,
            "to_account": to_id
        }));
    }

    let _ = tx.rollback().await;

    txn_counter.add(
        1,
        &[
            KeyValue::new("type", "unknown"),
            KeyValue::new("result", "error"),
        ],
    );

    HttpResponse::BadRequest().json(serde_json::json!({
        "error": "INVALID_TRANSACTION_TYPE"
    }))
}
