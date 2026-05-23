use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::TransactionType;

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct AccountResponse {
    pub account_id: Uuid,
    pub balance: i64,
}

#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    pub transc_type: TransactionType,
    pub account_id: Option<Uuid>,
    pub from_account_id: Option<Uuid>,
    pub to_account_id: Option<Uuid>,
    pub amount: i64,
}
