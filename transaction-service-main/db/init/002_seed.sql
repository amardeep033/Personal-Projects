-- =========================
-- DUMMY BUSINESS
-- =========================
INSERT INTO businesses (id, name)
VALUES (
    '11111111-1111-1111-1111-111111111111',
    'demo_business'
)
ON CONFLICT DO NOTHING;

-- =========================
-- DUMMY API KEY
-- =========================
-- NOTE:
-- key_hash here is plain text ONLY for local testing.
-- In prod, store a hashed value.
INSERT INTO api_keys (
    id,
    business_id,
    key_hash,
    rate_limit_per_min,
    is_active
)
VALUES (
    '22222222-2222-2222-2222-222222222222',
    '11111111-1111-1111-1111-111111111111',
    'demo_api_key_123',
    60,
    true
)
ON CONFLICT DO NOTHING;

-- =========================
-- DUMMY WEBHOOK ENDPOINT
-- =========================
INSERT INTO webhook_endpoints (
    id,
    business_id,
    url,
    secret,
    is_active
)
VALUES (
    '33333333-3333-3333-3333-333333333333',
    '11111111-1111-1111-1111-111111111111',
    'http://webhook-listener:8080/webhook',
    'super_secret_webhook_key',
    true
)
ON CONFLICT DO NOTHING;