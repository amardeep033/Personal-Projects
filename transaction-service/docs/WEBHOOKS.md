# WEBHOOKS
- The service supports outbound webhooks to notify businesses about transaction-related events.
- Webhooks are delivered asynchronously, reliably, and securely.

## 1 Webhook Endpoints
- Webhook endpoints are pre-registered in the database in the **webhook_endpoints** table.
- Each webhook endpoint belongs to a business and can be enabled or disabled.
- Only active webhook endpoints receive events.
- Webhook events are stored in the **webhook_events** table for durability and retry tracking.

## 2. Webhook Event Lifecycle

- A webhook event can be in one of the following states:  
  **PENDING**, **PROCESSING**, **DELIVERED**, **FAILED**.

- When a **transaction is successfully created**, a webhook event is enqueued in the database with status **PENDING**.
- A **background worker** processes pending webhook events and attempts delivery.
  - The worker is started asynchronously during application startup.
  - Webhook processing runs independently of the HTTP request lifecycle and does not block API responses.
- The worker picks pending events and marks them as **PROCESSING** before attempting delivery.
- On successful delivery, the event is marked as **DELIVERED**.
- On failure, the event is reset back to **PENDING** and the attempt count is incremented.
- The event is retried in subsequent cycles, and once the maximum retry count is exhausted, it is marked as **FAILED**.
- Webhook events that remain in **PROCESSING** beyond a configured timeout are automatically recovered.  
  These stuck events are reset back to **PENDING** so they can be retried.

## 3 Event Types
Currently supported event types include: **transaction.created**

## 4 Webhook Request
- Method: POST
- Content-Type: application/json
- Destination URL: Configured webhook endpoint URL
- Example Payload : The payload is stored and delivered exactly as JSON
    ```
    {
    "event_type": "transaction.created",
    "data": {
        "transaction_id": "cc4f4cea-9b26-4772-8748-246a38cf87fe",
        "type": "credit",
        "account_id": "04ba9e9d-7d95-43ea-a0df-a95022c0b788",
        "amount": 50
    }
    }
    ```

## 5 Webhook Security (Signature Verification)
- Each webhook request includes an HMAC signature header: X-Signature: sha256=<hex_digest>
- The signature is generated using HMAC-SHA256 over the raw request body.
- The webhook endpoint’s secret is used as the signing key.
- The prefix sha256= is included in the header value.
- This allows webhook receivers to verify authenticity and prevent tampering.

## 6 Retry Strategy
- Webhooks are delivered with **at-least-once** semantics.
- Failures are retried using **exponential backoff**: delay = 2 ^ attempt_count seconds
- Retries occur when: Network errors occur, HTTP status is >= 500, HTTP status is 429
- Webhook delivery stops when: The maximum retry count is reached or A non-retryable HTTP status (< 500 and ≠ 429) is returned