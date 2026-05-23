// ==============================
// Environment variables
// ==============================
pub const ENV_DATABASE_URL: &str = "DATABASE_URL";
pub const ENV_SERVER_HOST: &str = "SERVER_HOST";
pub const ENV_SERVER_PORT: &str = "SERVER_PORT";
pub const ENV_LOG_LEVEL: &str = "LOG_LEVEL";

pub const ENV_WEBHOOK_BATCH_SIZE: &str = "WEBHOOK_BATCH_SIZE";
pub const ENV_WEBHOOK_MAX_RETRIES: &str = "WEBHOOK_MAX_RETRIES";
pub const ENV_WEBHOOK_PROCESSING_TIMEOUT_SECS: &str = "WEBHOOK_PROCESSING_TIMEOUT_SECS";

// ==============================
// Default configuration values
// ==============================
pub const DEFAULT_SERVER_HOST: &str = "127.0.0.1";
pub const DEFAULT_SERVER_PORT: u16 = 8080;
pub const DEFAULT_LOG_LEVEL: &str = "info";

pub const DEFAULT_WEBHOOK_BATCH_SIZE: i64 = 20;
pub const DEFAULT_WEBHOOK_MAX_RETRIES: i32 = 5;
pub const DEFAULT_WEBHOOK_PROCESSING_TIMEOUT_SECS: i64 = 120;

// ==============================
// Cryptography
// ==============================
pub const HMAC_SHA256_PREFIX: &str = "sha256=";

// ==============================
// Transaction types
// ==============================
pub const TXN_CREDIT: &str = "credit";
pub const TXN_DEBIT: &str = "debit";
pub const TXN_TRANSFER: &str = "transfer";

// ==============================
// Transaction status & events
// ==============================
pub const TXN_STATUS_POSTED: &str = "posted";
pub const EVENT_TRANSACTION_POSTED: &str = "transaction.posted";

// ==============================
// Webhook statuses & errors
// ==============================
pub const WEBHOOK_STATUS_PENDING: &str = "pending";
pub const WEBHOOK_STATUS_PROCESSING: &str = "processing";
pub const WEBHOOK_STATUS_DELIVERED: &str = "delivered";
pub const WEBHOOK_STATUS_FAILED: &str = "failed";

pub const ERR_MAX_RETRIES: &str = "max retries exceeded";

// ==============================
// API routing
// ==============================
pub const ROUTE_ACCOUNTS: &str = "/accounts";
pub const ROUTE_ACCOUNT_BY_ID: &str = "/accounts/{id}";
pub const ROUTE_TRANSACTIONS: &str = "/transactions";
pub const ROUTE_HEALTH: &str = "/health";

// ==============================
// Rate limiting
// ==============================
pub const ERR_RATE_LIMIT_EXCEEDED: &str = "RATE_LIMIT_EXCEEDED";

// ==============================
// HTTP headers & auth
// ==============================
pub const HEADER_AUTHORIZATION: &str = "Authorization";
pub const AUTH_SCHEME_BEARER: &str = "Bearer ";
pub const HEADER_IDEMPOTENCY_KEY: &str = "Idempotency-Key";

// ==============================
// Logging / OTEL
// ==============================
pub const ENV_SERVICE_NAME: &str = "SERVICE_NAME";
pub const ENV_OTEL_EXPORTER_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
pub const DEFAULT_SERVICE_NAME: &str = "transaction-service";
pub const DEFAULT_OTEL_EXPORTER_ENDPOINT: &str = "http://localhost:4317";
pub const SERVICE: &str = "service";
