use crate::constants::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Credit,
    Debit,
    Transfer,
}

impl<'de> Deserialize<'de> for TransactionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            TXN_CREDIT => Ok(TransactionType::Credit),
            TXN_DEBIT => Ok(TransactionType::Debit),
            TXN_TRANSFER => Ok(TransactionType::Transfer),
            _ => Err(serde::de::Error::custom("invalid transaction type")),
        }
    }
}

impl Serialize for TransactionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            TransactionType::Credit => TXN_CREDIT,
            TransactionType::Debit => TXN_DEBIT,
            TransactionType::Transfer => TXN_TRANSFER,
        };
        serializer.serialize_str(s)
    }
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Credit => TXN_CREDIT,
            TransactionType::Debit => TXN_DEBIT,
            TransactionType::Transfer => TXN_TRANSFER,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookStatus {
    Pending,
    Processing,
    Delivered,
    Failed,
}

impl WebhookStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WebhookStatus::Pending => WEBHOOK_STATUS_PENDING,
            WebhookStatus::Processing => WEBHOOK_STATUS_PROCESSING,
            WebhookStatus::Delivered => WEBHOOK_STATUS_DELIVERED,
            WebhookStatus::Failed => WEBHOOK_STATUS_FAILED,
        }
    }
}
