-- =========================
-- EXTENSIONS
-- =========================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =========================
-- BUSINESSES
-- =========================
CREATE TABLE IF NOT EXISTS businesses (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- =========================
-- API KEYS
-- =========================
CREATE TABLE IF NOT EXISTS api_keys (
    id uuid PRIMARY KEY,
    business_id uuid NOT NULL,
    key_hash text NOT NULL,
    rate_limit_per_min integer NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),

    CONSTRAINT api_keys_business_id_key UNIQUE (business_id),
    CONSTRAINT api_keys_business_id_fkey
        FOREIGN KEY (business_id)
        REFERENCES businesses (id)
);

-- Fast auth lookup
CREATE UNIQUE INDEX IF NOT EXISTS idx_api_keys_key_hash
ON api_keys (key_hash)
WHERE is_active = true;

-- =========================
-- API RATE LIMITS
-- =========================
CREATE TABLE IF NOT EXISTS api_rate_limits (
    api_key_id uuid NOT NULL,
    window_start timestamptz NOT NULL,
    count integer NOT NULL,

    CONSTRAINT api_rate_limits_pkey PRIMARY KEY (api_key_id, window_start),
    CONSTRAINT api_rate_limits_api_key_id_fkey
        FOREIGN KEY (api_key_id)
        REFERENCES api_keys (id)
        ON DELETE CASCADE
);

-- =========================
-- ACCOUNTS
-- =========================
CREATE TABLE IF NOT EXISTS accounts (
    id uuid PRIMARY KEY,
    business_id uuid NOT NULL,
    name text NOT NULL,
    balance bigint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),

    CONSTRAINT accounts_business_id_fkey
        FOREIGN KEY (business_id)
        REFERENCES businesses (id),
    CONSTRAINT accounts_balance_check CHECK (balance >= 0)
);

-- Lock + fetch account during tx
CREATE INDEX IF NOT EXISTS idx_accounts_business_id_id
ON accounts (business_id, id);

-- =========================
-- TRANSACTIONS
-- =========================
CREATE TABLE IF NOT EXISTS transactions (
    id uuid PRIMARY KEY,
    api_key_id uuid NOT NULL,
    business_id uuid NOT NULL,
    from_account_id uuid,
    to_account_id uuid,
    type text NOT NULL,
    amount bigint NOT NULL,
    balance_before bigint NOT NULL,
    balance_after bigint NOT NULL,
    idempotency_key text NOT NULL,
    status text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),

    CONSTRAINT transactions_api_key_id_idempotency_key_key
        UNIQUE (api_key_id, idempotency_key),

    CONSTRAINT transactions_api_key_id_fkey
        FOREIGN KEY (api_key_id)
        REFERENCES api_keys (id),

    CONSTRAINT transactions_business_id_fkey
        FOREIGN KEY (business_id)
        REFERENCES businesses (id),

    CONSTRAINT transactions_from_account_id_fkey
        FOREIGN KEY (from_account_id)
        REFERENCES accounts (id),

    CONSTRAINT transactions_to_account_id_fkey
        FOREIGN KEY (to_account_id)
        REFERENCES accounts (id),

    CONSTRAINT transactions_type_check
        CHECK (type IN ('credit', 'debit', 'transfer')),

    CONSTRAINT transactions_amount_check
        CHECK (amount > 0),

    CONSTRAINT transactions_status_check
        CHECK (status IN ('posted', 'failed')),

    CONSTRAINT transactions_semantic_check CHECK (
        (type = 'credit'   AND from_account_id IS NULL AND to_account_id IS NOT NULL) OR
        (type = 'debit'    AND from_account_id IS NOT NULL AND to_account_id IS NULL) OR
        (type = 'transfer' AND from_account_id IS NOT NULL AND to_account_id IS NOT NULL)
    )
);

-- Query indexes
CREATE INDEX IF NOT EXISTS idx_transactions_business_created
ON transactions (business_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_transactions_from_account
ON transactions (from_account_id);

CREATE INDEX IF NOT EXISTS idx_transactions_to_account
ON transactions (to_account_id);

-- =========================
-- WEBHOOK ENDPOINTS
-- =========================
CREATE TABLE IF NOT EXISTS webhook_endpoints (
    id uuid PRIMARY KEY,
    business_id uuid NOT NULL,
    url text NOT NULL,
    secret text NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),

    CONSTRAINT webhook_endpoints_business_id_fkey
        FOREIGN KEY (business_id)
        REFERENCES businesses (id)
);

-- Active endpoints lookup
CREATE INDEX IF NOT EXISTS idx_webhook_endpoints_business_active
ON webhook_endpoints (business_id)
WHERE is_active = true;

-- =========================
-- WEBHOOK EVENTS
-- =========================
CREATE TABLE IF NOT EXISTS webhook_events (
    id uuid PRIMARY KEY,
    business_id uuid NOT NULL,
    endpoint_id uuid NOT NULL,
    event_type text NOT NULL,
    resource_id uuid,
    payload jsonb NOT NULL,
    attempt_count integer NOT NULL DEFAULT 0,
    last_error text,
    status text NOT NULL,
    next_retry_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    processing_started_at timestamptz,

    CONSTRAINT webhook_events_business_id_fkey
        FOREIGN KEY (business_id)
        REFERENCES businesses (id),

    CONSTRAINT webhook_events_endpoint_id_fkey
        FOREIGN KEY (endpoint_id)
        REFERENCES webhook_endpoints (id)
);

-- Worker polling efficiency
CREATE INDEX IF NOT EXISTS idx_webhook_events_status_retry
ON webhook_events (status, next_retry_at)
WHERE status IN ('pending', 'processing');

-- Cleanup / observability
CREATE INDEX IF NOT EXISTS idx_webhook_events_created
ON webhook_events (created_at);
