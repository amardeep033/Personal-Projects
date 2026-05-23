# Transaction Service

## 1.  Overview
  This is a simple **transaction-service** which has the following features:
  - **1.1** There are pre-registered businesses in the system.
  - **1.2** Each business have some accounts linked with them.
  - **1.3** Three types of transactions are supported between accounts: 
      - **credit**: external to internal 
      - **debit**: internal to external 
      - **transaction**: in between internal accounts
  - **1.4** Businesses can create account and check their balances.

## 2. TechStack
  This service makes use of the following technologies:
  - **2.1** Rust (Actix-Web)  
  - **2.2** PostgreSQL  
  - **2.3** OpenTelemetry + Jaeger  


## 3. Features
  - **3.1** Makes use of idempotency key to avoid duplicate transaction on re-trigger.  
  - **3.2** Also make sure of cases like invalid balance, same account transfer, insufficient balance.  
  - **3.3** Authentication using API keys  
  - **3.4** Atomic balance update, no incomplete transaction  
  - **3.5** Webhook implementation with signed key and retry feature  
  - **3.6** API rate limiting  
  - **3.7** OpenTelemetry for observability  
  **Note:** Please refer to `docs/DESIGN.md` for full details.


## 4. Folder Structure

- **4.1** `README.md` – current file, serving as help documentation to get started  
- **4.2** `db/` – database schema definitions and dummy data seeds  
- **4.3** `docker-compose.yaml` and `Dockerfile` – one-click setup for local development  
- **4.4** `docs/`  
  - **4.4.1** `API.md` – details about requests, responses, and error formats  
  - **4.4.2** `DESIGN.md` – database design, architectural decisions, and trade-offs  
  - **4.4.3** `WEBHOOKS.md` – details about webhooks integration  
- **4.5** `src/`, `Cargo.toml`, and `Cargo.lock` – application source code and Rust dependency metadata  
- **4.6** `.sqlx/` – SQLx offline metadata used for compile-time query validation  

## 5.  Getting started
  - **5.1** Clone the repository
    `git clone git@github.com:amardeep033/transaction-service.git`
  - **5.2** Navigate to the cloned directory
    `cd transaction-service`
  - **5.3** Start the environment using Docker (single-click setup):
    `docker compose up --build`
    It will
      - **5.3.1** Start the application
      - **5.3.2** Initialize the database with tables and seed data
      - **5.3.3** Run database health checks from within the application
      - **5.3.4** Start Jaeger for OpenTelemetry logs and traces

## 6.  Basic API testing
  This service contains 4 basic APIs:
  - **/health** - checking the db connection from app
  - **/accounts** - creating a new account
  - **/transactions** - credit, debit or trasaction between accounts
  - **/accounts/{id}** - checking the balance of the account
  - **Note**: Please refer to docs/API.md for full detail

    ### 6.1 Testing the /health:
      - **6.1.1** Let us check the connection of db from app using Request: `curl http://localhost:8080/health`
      - **6.1.2** Expected Response: `{"status":"ok"}`

    ### 6.2 Testing the /accounts:
      - **6.2.1** In the seed, one business account is already created id '11111111-1111-1111-1111-111111111111' along with API key 'demo_api_key_123'. You can confirm this by connecting to db `docker exec -it payments-db psql -U myuser -d test` and running the queries `select * from businesses;` and `select * from api_keys;`.
      - **6.2.2** Let's create new account named 'primary' using Request: `curl -s -X POST http://localhost:8080/accounts -H "Authorization: Bearer demo_api_key_123" -H "Content-Type: application/json" -d '{"name":"primary"}'`
      - **6.2.3** Expected Response = `{"account_id":"#PRIMARY_ACCOUNT_KEY#","balance":0}`
      - **6.2.4** We will create one more account named secondary: `curl -s -X POST http://localhost:8080/accounts -H "Authorization: Bearer demo_api_key_123" -H "Content-Type: application/json" -d '{"name":"secondary"}'`
      Expected Response = `{"account_id":"#SECONDARY_ACCOUNT_KEY#","balance":0}`

    **NOTE**: In below requests: replace #PRIMARY_ACCOUNT_KEY# with returned id in step 6.2.2
          and replace #SECONDARY_ACCOUNT_KEY# with returned id in step 6.2.4

    ### 6.3 Testing the /transactions and /accounts/{id}:

    ### - **6.3.1** Testing the credit:
    - **6.3.1.1** Lets credit 100 amount to primary_account using Request:
      `curl -X POST http://127.0.0.1:8080/transactions -H "Authorization: Bearer demo_api_key_123" -H "Idempotency-Key: credit-primary-100" -H "Content-Type: application/json" -d '{ "transc_type":"credit", "account_id":"#PRIMARY_ACCOUNT_KEY#", "amount":100 }'`
    - **6.3.1.2** Expected response : `{"balance_after":100,"balance_before":0,"status":"posted","transaction_id":"cc4f4cea-9b26-4772-8748-246a38cf87fe"}`
    - **6.3.1.3** Confirm the amount using request:
      `curl http://127.0.0.1:8080/accounts/#PRIMARY_ACCOUNT_KEY# -H "Authorization: Bearer demo_api_key_123"`
      Expected Response: `{"account_id":"#PRIMARY_ACCOUNT_KEY#","balance":100}`

    ### 6.3.2 Testing the debit:
      - **6.3.2.1** Lets debit 70 amount to primary_account using Request:
      `curl -X POST http://127.0.0.1:8080/transactions -H "Authorization: Bearer demo_api_key_123" -H "Idempotency-Key: debit-primary-70" -H "Content-Type: application/json" -d '{ "transc_type":"debit", "account_id":"#PRIMARY_ACCOUNT_KEY#", "amount":70 }'`
      - **6.3.2.2** Expected response : `{"balance_after":30,"balance_before":100,"status":"posted","transaction_id":"c13e965b-6786-4578-b2fa-7f072222dda1"}`
      - **6.3.2.3** Confirm the amount using request:
      `curl http://127.0.0.1:8080/accounts/#PRIMARY_ACCOUNT_KEY# -H "Authorization: Bearer demo_api_key_123"`
      Expected Response: `{"account_id":"#PRIMARY_ACCOUNT_KEY#","balance":30}`

    ### 6.3.3 Testing the transfer:
      - **6.3.3.1** Currently the initial amount in primary account is 30 and secondary account is 0.
      `curl http://127.0.0.1:8080/accounts/#PRIMARY_ACCOUNT_KEY# -H "Authorization: Bearer demo_api_key_123"` will yield `{"account_id":"#PRIMARY_ACCOUNT_KEY#","balance":30}`
      and 
      `curl http://127.0.0.1:8080/accounts/#SECONDARY_ACCOUNT_KEY# -H "Authorization: Bearer demo_api_key_123"` will yield `{"account_id":"#SECONDARY_ACCOUNT_KEY#","balance":0}`
      - **6.3.3.2** Let's transfer amount 10 from primary to secondary account using request: `curl -X POST http://127.0.0.1:8080/transactions -H "Authorization: Bearer demo_api_key_123" -H "Idempotency-Key: transfer-prim-sec-10" -H "Content-Type: application/json" -d '{ "transc_type":"transfer", "from_account_id":"#PRIMARY_ACCOUNT_KEY#", "to_account_id":"#SECONDARY_ACCOUNT_KEY#", "amount":10 }'`
    - **6.3.3.3** Expected response: `{"from_account":"#PRIMARY_ACCOUNT_KEY#","status":"posted","to_account":"#SECONDARY_ACCOUNT_KEY#","transaction_id":"5d51aa46-493d-413b-8fab-3e3d4f04edb3"}`
    - **6.3.3.4** Now let's re-check the amount: 
      `curl http://127.0.0.1:8080/accounts/#PRIMARY_ACCOUNT_KEY# -H "Authorization: Bearer demo_api_key_123"` will yield `{"account_id":"#PRIMARY_ACCOUNT_KEY#","balance":20}`
      and 
      `curl http://127.0.0.1:8080/accounts/#SECONDARY_ACCOUNT_KEY# -H "Authorization: Bearer demo_api_key_123"` will yield `{"account_id":"#SECONDARY_ACCOUNT_KEY#","balance":10}`



## 7. Edge cases testing

  ### 7.1 Unauthorised access
  - **7.1.1** Let's try creating an account without API key using request `curl -v -X POST http://127.0.0.1:8080/accounts1:8080/accounts`
  - **7.1.2** Expected response: `HTTP/1.1 401 Unauthorized`

  ### 7.2 Non-existence account
  - **7.2.1** Let's try checking the balance of non-existent account using request `curl -v http://127.0.0.1:8080/accounts/25633e0d-e8c0-4056-8f2c-412e0ede73b7 -H "Authorization: Bearer demo_api_key_123"`
  - **7.2.2** Expected response: `HTTP/1.1 404 Not Found`

  ### 7.3 Transferring invalid amount
  - **7.3.1** Let's try transferring negative amount using request `curl -X POST http://127.0.0.1:8080/transactions -H "Authorization: Bearer demo_api_key_123" -H "Idempotency-Key: credit-primary-50" -H "Content-Type: application/json" -d '{ "transc_type":"credit", "account_id":"#PRIMARY_ACCOUNT_KEY#", "amount":-50 }'`
  - **7.3.2** Expected response: `{"error":"INVALID_AMOUNT"}`

  ### 7.4 Insufficient amount
  - **7.4.1** Let's re-check the amount in primary account: 
        `curl http://127.0.0.1:8080/accounts/#PRIMARY_ACCOUNT_KEY# -H "Authorization: Bearer demo_api_key_123"` will yield `{"account_id":"#PRIMARY_ACCOUNT_KEY#","balance":20}`
  - **7.4.2** It has balance 20. Let's try debiting amount 50 from it using request `curl -X POST http://127.0.0.1:8080/transactions -H "Authorization: Bearer demo_api_key_123" -H "Idempotency-Key: debit-primary-50" -H "Content-Type: application/json" -d '{ "transc_type":"debit", "account_id":"#PRIMARY_ACCOUNT_KEY#", "amount":50 }'`
  - **7.4.3** Expected response: `{"error":"INSUFFICIENT_FUNDS"}`

  ### 7.5 Idempotency
  - **7.5.1** We know from 7.4.1 that primary account balance is 20. Now let's retry the same transaction(same Idempotency-Key) twice using the request `curl -X POST http://127.0.0.1:8080/tcurl -X POST http://127.0.0.1:8080/transactions -H "Authorization: Bearer demo_api_key_123" -H "Idempotency-Key: debit-primary-10" -H "Content-Type: application/json" -d '{ "transc_type":"debit", "account_id":"#PRIMARY_ACCOUNT_KEY#", "amount":10 }'` two times.
  - **7.5.2** First Response: `{"balance_after":10,"balance_before":20,"status":"posted","transaction_id":"6a72aaa8-5a80-4b82-aafb-9ffe2e810165"}`
  - **7.5.3** Second Response: `{"idempotent":true,"status":"posted","transaction_id":"6a72aaa8-5a80-4b82-aafb-9ffe2e810165"}`
  - **7.5.4** Let's check the balance using request `curl http://127.0.0.1:8080/accounts/b4beb5ce-90d6-4ab9-adcurl http://127.0.0.1:8080/accounts/#PRIMARY_ACCOUNT_KEY# -H "Authorization: Bearer demo_api_key_123"` to make sure it was debited only once.
  - **7.5.5** Expected response: `{"account_id":"#PRIMARY_ACCOUNT_KEY#","balance":10}`


  ### 7.6 Transfer from-to same account
  - **7.6.1** Let's try to transfer between same account using request `curl -X POST http://127.0.0.1:8080/transactions -H "Authorization: Bearer demo_api_key_123" -H "Idempotency-Key: transfer-same-same-5" -H "Content-Type: application/json" -d '{ "transc_type":"transfer", "from_account_id":"#PRIMARY_ACCOUNT_KEY#", "to_account_id":"#PRIMARY_ACCOUNT_KEY#", "amount":5 }'`.
  - **7.6.2** Expected response: `{"error":"SAME_ACCOUNT_TRANSFER"}`
  

## 8. Rate limiting
  - **8.1** In the seeds, we have already set the rate limit to 60 request per min in the db.
  - **8.2** You can verify this in by connecting to db using `docker exec -it payments-db psql -U myuser -d test` and running the following query: `select * from api_keys;`.
  - **8.3** Run the following script to send 100 request: 
    `for i in {1..100}; do
      printf "%03d → " "$i"
      curl -s -o /dev/null \
        -w "%{http_code}\n" \
        -H "Authorization: Bearer demo_api_key_123" \
        http://127.0.0.1:8080/accounts/#PRIMARY_ACCOUNT_KEY#
    done`
  - **8.4** The first 60 should return the result `001 → 200 ... 060 → 200` which means ok response, and the next 40 request will result in `61 → 429 ... 100 → 429` which means too many request.

## 9. Webhook Listener
  - **9.1** In the seeds, we have already inserted one url for webhook listener `http://webhook-listener:8080/webhook`.
  - **9.2** If you check the db using following steps, you will see the all the entries in the webhook events are marked as failed with retries as no listener were up yet.
  - **`do**cker exec -it payments-db psql -U myuser -d test` and inside the terminal try the query `select * from webhook_events;`
  - **9.2** Now we will start a dummy listener. Open terminal and run the following command: `docker logs -f webhook-listener`
  - **9.3** Now let's do one transaction using request `curl -X POST http://127.0.0.1:8080/transactions -H "Authorization: Bearer demo_api_key_123" -H "Idempotency-Key: credit-sec-50" -H "Content-Type: application/json" -d '{ "transc_type":"credit", "account_id":"#SECONDARY_ACCOUNT_KEY#", "amount":50 }'`.
  - **9.4** In the above 9.2 terminal, it will be received as `"path": "/webhook",
    "headers": {
        "x-signature": "sha256=9dcaf5f586658bba59751ff25a6f801dcc2dbbd231f4bc2752f145dcd73078a8",
        "content-type": "application/json",
        "accept": "*/*",
        "host": "webhook-listener:8080",
        "content-length": "137"
    },
    "method": "POST"...`.
- **Note**: Please refer to docs/WEBHOOKS.md for full detail

## 10. OpenTelemetry
  - **10.1** The service is instrumented with OpenTelemetry to export distributed traces (and correlated logs) to Jaeger.
  - **10.2** Traces are exported using the OTLP gRPC exporter. Jaeger runs as a sidecar service in Docker.
  - **10.3** The application sends telemetry data to Jaeger via: `OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317`.
  - **10.4** Once the services are running, you can access the Jaeger UI at: `http://localhost:16686`.
  - **10.5** From the Jaeger UI, select the service name configured via: `SERVICE_NAME=transaction-service`

## 11. Configuration
  - **11.1** If running the service outside Docker (without docker compose), configuration must be provided using environment variables.
  - **11.2** You may create a .env file at the project root and export the following values:
  ```
  DATABASE_URL=postgres://myuser:mypassword@localhost:5432/test
  SERVER_HOST=0.0.0.0
  SERVER_PORT=8080
  LOG_LEVEL=info
  SERVICE_NAME=transaction-service
  OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
  WEBHOOK_BATCH_SIZE=20
  WEBHOOK_MAX_RETRIES=5
  WEBHOOK_PROCESSING_TIMEOUT_SECS=120
  ```
  - **11.3** DATABASE_URL is mandatory. All other values have sensible defaults and are optional. Default values can be checked at src/constants.rs.
  - **11.4** Note: When running via Docker Compose, all required configuration is already provided and no .env file is needed.