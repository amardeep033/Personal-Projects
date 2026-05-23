# APIs

## 1. Introduction
- Base URL: http://localhost:8080
- This service contains 4 basic APIs:
    - **/health** - checking the db connection from app
    - **/accounts** - creating a new account
    - **/transactions** - credit, debit or transactions between accounts
    - **/accounts/{id}** - checking the balance of the account

- **Note** : All endpoints (except /health) require API key authentication.
- **Note** : /transactions requires idempotency key

## 2. Authentication
- Every request must include an API key in the Authorization header. 
- Format: Authorization: Bearer <API_KEY>
- Example: Authorization: Bearer demo_api_key_123
- If the API key is missing or invalid, the service responds with: `HTTP/1.1 401 Unauthorized`

## 3. Error Responses
- Errors are returned in JSON format.
- Example: 
    ```
    {
    "error": "INSUFFICIENT_FUNDS"
    }
    ```
- Common Errors
    | Error                   | Meaning                                     |
    | ----------------------- | ------------------------------------------- |
    | `UNAUTHORIZED`          | Missing or invalid API key                  |
    | `NOT_FOUND`             | Account does not exist                      |
    | `INVALID_AMOUNT`        | Amount ≤ 0                                  |
    | `INSUFFICIENT_FUNDS`    | Balance too low                             |
    | `SAME_ACCOUNT_TRANSFER` | From and To accounts are same               |
    | `IDEMPOTENCY_CONFLICT`  | Same idempotency key with different payload |
    | `RATE_LIMITED`          | Too many requests                           |
    | `INTERNAL_ERROR`        | Unexpected server error                     |


## 4. Health Check
- **4.1 Usage**: Check Service & DB Health
- **4.2 Method**: GET
- **4.3 Endpoint**: /health
- **4.4 Headers**: None
- **4.5 Request**: Does not take any arguments
    ```
    curl http://localhost:8080/health
    ```
- **4.6 Response**: Returns ok if the service can successfully connect to the database
    ```
    {
    "status": "ok"
    }
    ```
- **4.7 Errors**: INTERNAL_ERROR


## 5. Accounts

### 5.1 Create Account
- **5.1.1 Usage**: Creates a new account under the authenticated business.
- **5.1.2 Method**: POST
- **5.1.3 Endpoint**: /accounts
- **5.1.4 Headers**: Authorization: Bearer <API_KEY>
- **5.1.5 Request**: Pass the account name using the name field
    ```
    curl -s -X POST http://localhost:8080/accounts -H "Authorization: Bearer demo_api_key_123" -H "Content-Type: application/json" -d '{"name":"primary"}'
    ```
- **5.1.6 Response**: Returns the generated account_id with an initial balance of 0
    ```
    {"account_id":"#ACCOUNT_ID#","balance":0}
    ```
- **5.1.7 Errors**: UNAUTHORIZED, INVALID_REQUEST and INTERNAL_ERROR


### 5.2 Get Account Balance
- **5.2.1 Usage**: Fetches the current balance of an account
- **5.2.2 Method**: GET
- **5.2.3 Endpoint**: /accounts/{account_id}
- **5.2.4 Headers**: Authorization: Bearer <API_KEY>
- **5.2.5 Request**: Pass the account ID as a path parameter
    ```
    curl http://localhost:8080/accounts/#ACCOUNT_ID# \
    -H "Authorization: Bearer demo_api_key_123"
    ```
- **5.2.6 Response**: Returns the account ID and current balance
    ```
    {
    "account_id": "#ACCOUNT_ID#",
    "balance": 100
    }
    ```
- **5.2.7 Errors**: NOT_FOUND, UNAUTHORIZED, INVALID_REQUEST and INTERNAL_ERROR

## 6. Transactions

### 6.1 Credit Transaction
- **6.1.1 Usage**: Credits money into an internal account.
- **6.1.2 Method**: POST
- **6.1.3 Endpoint**: /transactions
- **6.1.4 Headers**: Authorization: Bearer <API_KEY> and Idempotency-Key: <UNIQUE_KEY>
- **6.1.5 Request**: Pass the account ID and amount with transc_type 'credit'
    ```
    curl -X POST http://localhost:8080/transactions \
    -H "Authorization: Bearer demo_api_key_123" \
    -H "Idempotency-Key: credit-primary-100" \
    -H "Content-Type: application/json" \
    -d '{
        "transc_type": "credit",
        "account_id": "#ACCOUNT_ID#",
        "amount": 100
    }'
    ```
- **6.1.6 Response**: Returns transaction details and balance change
    ```
    {
    "transaction_id": "cc4f4cea-9b26-4772-8748-246a38cf87fe",
    "status": "posted",
    "balance_before": 0,
    "balance_after": 100
    }
    ```

    ```
    {"idempotent":true,"status":"posted","transaction_id":"6a72aaa8-5a80-4b82-aafb-9ffe2e810165"}
    ```
- **6.1.7 Errors**: INVALID_AMOUNT, UNAUTHORIZED, INTERNAL_ERROR

### 6.2 Debit Transaction
- **6.2.1 Usage**: Debits money from an internal account
- **6.2.2 Method**: POST
- **6.2.3 Endpoint**: /transactions
- **6.2.4 Headers**: Authorization: Bearer <API_KEY> and Idempotency-Key: <UNIQUE_KEY>
- **6.2.5 Request**: Pass the account ID and amount with transc_type as debit.
    ```
    curl -X POST http://localhost:8080/transactions \
    -H "Authorization: Bearer demo_api_key_123" \
    -H "Idempotency-Key: debit-primary-70" \
    -H "Content-Type: application/json" \
    -d '{
        "transc_type": "debit",
        "account_id": "#ACCOUNT_ID#",
        "amount": 70
    }'
    ```
- **6.2.6 Response**: Returns transaction details and balance change
    ```
    {
    "transaction_id": "c13e965b-6786-4578-b2fa-7f072222dda1",
    "status": "posted",
    "balance_before": 100,
    "balance_after": 30
    }
    ```

    ```
    {"idempotent":true,"status":"posted","transaction_id":"6a72aaa8-5a80-4b82-aafb-9ffe2e810165"}
    ```
- **6.2.7 Errors**: INSUFFICIENT_FUNDS, INVALID_AMOUNT, UNAUTHORIZED, INTERNAL_ERROR


### 6.3 Transfer Transaction
- **6.3.1 Usage**: Transfers money between two internal accounts
- **6.3.2 Method**: POST
- **6.3.3 Endpoint**: /transactions
- **6.3.4 Headers**: Authorization: Bearer <API_KEY> and Idempotency-Key: <UNIQUE_KEY>
- **6.3.5 Request**: Pass the source account ID, destination account ID, and amount with transc_type as transfer.
    ```
    curl -X POST http://localhost:8080/transactions \
    -H "Authorization: Bearer demo_api_key_123" \
    -H "Idempotency-Key: transfer-primary-secondary-10" \
    -H "Content-Type: application/json" \
    -d '{
        "transc_type": "transfer",
        "from_account_id": "#FROM_ACCOUNT_ID#",
        "to_account_id": "#TO_ACCOUNT_ID#",
        "amount": 10
    }'
    ```
- **6.3.6 Response**: Returns transaction details and balance change
    ```
    {
    "transaction_id": "5d51aa46-493d-413b-8fab-3e3d4f04edb3",
    "status": "posted",
    "from_account": "#FROM_ACCOUNT_ID#",
    "to_account": "#TO_ACCOUNT_ID#"
    }
    ```

    ```
    {"idempotent":true,"status":"posted","transaction_id":"6a72aaa8-5a80-4b82-aafb-9ffe2e810165"}
    ```
- **6.3.7 Errors**: SAME_ACCOUNT_TRANSFER, INSUFFICIENT_FUNDS, INVALID_AMOUNT, UNAUTHORIZED, INTERNAL_ERROR