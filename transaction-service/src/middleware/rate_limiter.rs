use actix_web::HttpMessage;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use sqlx::PgPool;
use std::{
    rc::Rc,
    task::{Context, Poll},
};
use tracing::{error, info};

use crate::{constants::ERR_RATE_LIMIT_EXCEEDED, middleware::auth::ApiKeyId};

#[derive(Clone)]
pub struct RateLimiter {
    pool: PgPool,
}

impl RateLimiter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl<S> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = RateLimitMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddleware {
            service: Rc::new(service),
            pool: self.pool.clone(),
        })
    }
}

pub struct RateLimitMiddleware<S> {
    service: Rc<S>,
    pool: PgPool,
}

impl<S> Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        let meter = global::meter("rate_limiter");
        let rl_counter: Counter<u64> = meter
            .u64_counter("rate_limit_requests_total")
            .with_description("Total number of rate limit checks")
            .init();

        let pool = self.pool.clone();

        let api_key_id = req.extensions().get::<ApiKeyId>().copied();

        let api_key_id = match api_key_id {
            Some(v) => v,
            None => {
                return Box::pin(async move {
                    Ok(req.into_response(HttpResponse::Unauthorized().finish()))
                });
            }
        };

        Box::pin(async move {
            let limit = match fetch_rate_limit(&pool, api_key_id).await {
                Ok(v) => v,
                Err(e) => {
                    error!("rate limit fetch failed: {}", e);
                    rl_counter.add(1, &[KeyValue::new("result", "error")]);
                    return Ok(req.into_response(HttpResponse::Unauthorized().finish()));
                }
            };

            if check_global_rate_limit(&pool, api_key_id, limit)
                .await
                .is_err()
            {
                info!(
                    api_key_id = %api_key_id.0,
                    "rate limit exceeded"
                );
                rl_counter.add(1, &[KeyValue::new("result", "limited")]);
                return Ok(req.into_response(
                    HttpResponse::TooManyRequests()
                        .json(serde_json::json!({ "error": ERR_RATE_LIMIT_EXCEEDED })),
                ));
            }
            rl_counter.add(1, &[KeyValue::new("result", "allowed")]);

            svc.call(req).await
        })
    }
}

pub async fn fetch_rate_limit(pool: &PgPool, api_key_id: ApiKeyId) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT rate_limit_per_min
        FROM api_keys
        WHERE id = $1 AND is_active = true
        "#,
        api_key_id.0
    )
    .fetch_one(pool)
    .await
}

pub async fn check_global_rate_limit(
    pool: &PgPool,
    api_key_id: ApiKeyId,
    limit: i32,
) -> Result<(), ()> {
    let row = sqlx::query!(
        r#"
        INSERT INTO api_rate_limits (api_key_id, window_start, count)
        VALUES ($1, date_trunc('minute', now()), 1)
        ON CONFLICT (api_key_id, window_start)
        DO UPDATE SET count = api_rate_limits.count + 1
        RETURNING count
        "#,
        api_key_id.0
    )
    .fetch_one(pool)
    .await
    .map_err(|_| ())?;

    if row.count > limit {
        Err(())
    } else {
        Ok(())
    }
}
