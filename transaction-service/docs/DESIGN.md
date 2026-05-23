# Design Specification – Transaction Service

## 1. Overview

This service implements a **simple, reliable transaction processing system** for businesses.  
It supports account management, atomic financial transactions, API authentication, rate limiting, and asynchronous webhooks.

The system is designed to prioritize:
- **Correctness over throughput**
- **Strong consistency for balances**
- **Operational reliability**
- **Clear failure handling**

---

## 2. High-Level Architecture

The service follows a **single-service, database-centric architecture**:

- HTTP API built using **Rust + Actix-Web**
- **PostgreSQL** as the source of truth
- Background workers for webhook delivery and retries
- OpenTelemetry for tracing and logging
- Docker Compose for local orchestration

There is **no in-memory state** critical to correctness; all important state is persisted in the database.

-- 

## 3. Data Model Design
The design contains seven tables:

### 3.1 businesses
- Schema: id | name | created_at
- Role: Represents a top-level tenant in the system.
- Assumption: Businesses are pre-registered; all accounts and API keys are scoped to a specific business ID to ensure strict multi-tenant isolation.

### 3.2 api_keys
- Schema: id | business_id (fk) | key_hash (indexed) | rate_limit_per_min | is_active | created_at
- Role: Handles authentication and global rate-limit configuration.
- Security: API keys are stored as hashes (key_hash) to prevent exposure in case of a database leak.
- Tenant Isolation: Each key is hard-linked to a business_id, ensuring a simple and robust multi-tenant boundary.
- Rate Limit Scoping: Limits are applied per API key rather than per endpoint to ensure predictable resource consumption for each business.

### 3.3 api_rate_limits
- Schema: api_key_id (fk) | window_start | count
- Strategy: Implements a fixed-window counter (per minute).
- Consistency: By persisting limits in PostgreSQL, the system maintains deterministic rate limiting across multiple API nodes without requiring an external cache like Redis.
- Trade-off: This approach prioritizes architectural simplicity and consistency over the lower latency of an in-memory store.

### 3.4 accounts
- Schema: id (indexed) | business_id (fk) | name | balance (bigint) | created_at
- Role: Stores the current state of a business's funds.
- Integrity: The balance is maintained as a materialized value to allow for fast reads.
- Constraints: A database-level Check Constraint ensures balance >= 0, providing a final layer of protection against accidental overdrawing (insufficient funds) at the engine level.
- Precision: Uses bigint (representing minor units, e.g., cents) to avoid the floating-point errors inherent in f64.

### 3.5 transactions
- Schema: id | api_key_id (fk) | business_id (fk) | from_account_id (fk) | to_account_id (fk) | type | amount | balance_before | balance_after | idempotency_key | status | created_at
- Atomicity: All updates (balance adjustments and ledger insertion) are executed within a single ACID-compliant database transaction. This prevents orphaned records and inconsistent balances.
- Concurrency: Implements strict row-level locking (SELECT FOR UPDATE) to prevent race conditions during concurrent updates to the same account.
- Auditability: Records are immutable. By storing balance_before and balance_after, the system provides a clear audit trail for every movement of funds.
- Idempotency: Enforced via a UNIQUE (api_key_id, idempotency_key) constraint.
- Matching Key: Returns the existing transaction record without re-processing.
- Payload Mismatch: Rejects the request if the same key is used for a different transaction payload.
- Resilience: Protects against network retries, timeouts, and duplicate client submissions.


### 3.6 webhook_endpoints
- Schema: id | business_id (fk) | url | secret | is_active | created_at
- Role: Configures the destination and security parameters for outbound notifications.
- Security: Each request is signed using HMAC-SHA256 with the endpoint's secret. This enables the receiver to verify that the request originated from this service and that the payload has not been altered in transit.
- Control: The is_active flag allows businesses to toggle notifications without deleting their configuration.

### 3.7 webhook_events
- Schema: id | business_id (fk) | endpoint_id (fk) | event_type | resource_id | payload | attempt_count | last_error | status | next_retry_at | created_at | processing_started_at
- Pattern: Implements the Transactional Outbox Pattern. Events are inserted into this table within the same database transaction as the financial ledger update, ensuring atomicity between state changes and notifications.
- Lifecycle Management:
    - Status States: PENDING, PROCESSING, DELIVERED, FAILED.
    - Reliability: Events are persisted to survive process crashes. The processing_started_at timestamp allows for the recovery of "stuck" events that exceeded a timeout threshold.
- Delivery Guarantees:
    - At-least-once: The system guarantees delivery by retrying until an acknowledgment is received or the attempt_count limit is reached.
    - Scope: Only source-of-truth events (e.g., successful transactions) are emitted to maintain high signal-to-noise for consumers. 
- For more details, refer to `docs/WEBHOOKS.md`

---

## 4. Observability

### Logging
- Structured logs are emitted at:
  - Request entry
  - Error paths
  - Webhook enqueueing and delivery failures

### Tracing
- OpenTelemetry instrumentation captures:
  - HTTP request spans
  - Database operations
  - Webhook delivery attempts

### Traces Export
- Traces are exported via OTLP to Jaeger
- Enables end-to-end request visibility

---

## 5. Operational Considerations

### Failure Modes Handled
- Partial DB failures
- Duplicate client requests
- Webhook endpoint downtime
- Worker crashes

### Deployment
- Stateless application containers
- Single command local setup via Docker Compose
- Database acts as coordination layer

### Scalability
- Horizontal scaling supported
- Database remains the primary bottleneck
- Design favors correctness over raw throughput

---

## 6. Trade-offs & Limitations

### Chosen Trade-offs
- PostgreSQL used for rate limiting instead of Redis for simplicity
- Materialized balances instead of ledger recomputation for performance
- Single service instead of microservices for clarity

### Known Limitations
- No pagination for transaction listing
- No webhook management APIs
- No metrics dashboards (only tracing)

These were consciously excluded to keep the scope focused.

---

## 7. Summary

This design emphasizes:
- Strong consistency
- Clear failure handling
- Operational reliability
- Simplicity over premature optimization

It provides a solid foundation for a payment-style transaction service while remaining easy to reason about and extend.
