use actix_web::HttpMessage;
use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use sqlx::PgPool;
use std::rc::Rc;
use std::task::{Context, Poll};
use tracing::error;
use uuid::Uuid;

use crate::constants::{AUTH_SCHEME_BEARER, HEADER_AUTHORIZATION};

#[derive(Clone, Copy)]
pub struct BusinessId(pub Uuid);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct ApiKeyId(pub Uuid);

pub struct ApiKeyAuth;

impl<S> Transform<S, ServiceRequest> for ApiKeyAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = ApiKeyAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ApiKeyAuthMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct ApiKeyAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for ApiKeyAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        let meter = global::meter("middleware.auth");
        let auth_counter: Counter<u64> = meter
            .u64_counter("auth_requests_total")
            .with_description("Total number of API key authentication attempts")
            .init();

        Box::pin(async move {
            let header = req
                .headers()
                .get(HEADER_AUTHORIZATION)
                .and_then(|h| h.to_str().ok());

            let api_key = match header {
                Some(h) if h.starts_with(AUTH_SCHEME_BEARER) => &h[AUTH_SCHEME_BEARER.len()..],
                _ => {
                    auth_counter.add(1, &[KeyValue::new("result", "unauthorized")]);
                    return Ok(req.into_response(HttpResponse::Unauthorized().finish()));
                }
            };

            let pool = match req.app_data::<actix_web::web::Data<PgPool>>() {
                Some(p) => p,
                None => {
                    error!("PgPool missing in app data");
                    auth_counter.add(1, &[KeyValue::new("result", "error")]);
                    return Ok(req.into_response(HttpResponse::InternalServerError().finish()));
                }
            };

            let row = match sqlx::query!(
                r#"
                SELECT id, business_id
                FROM api_keys
                WHERE key_hash = $1 AND is_active = true
                "#,
                api_key
            )
            .fetch_optional(pool.get_ref())
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    error!("API key lookup failed: {}", e);
                    auth_counter.add(1, &[KeyValue::new("result", "error")]);
                    return Ok(req.into_response(HttpResponse::InternalServerError().finish()));
                }
            };

            let row = match row {
                Some(r) => r,
                None => {
                    auth_counter.add(1, &[KeyValue::new("result", "unauthorized")]);
                    return Ok(req.into_response(HttpResponse::Unauthorized().finish()));
                }
            };

            req.extensions_mut().insert(ApiKeyId(row.id));
            req.extensions_mut().insert(BusinessId(row.business_id));

            auth_counter.add(1, &[KeyValue::new("result", "success")]);

            svc.call(req).await
        })
    }
}
