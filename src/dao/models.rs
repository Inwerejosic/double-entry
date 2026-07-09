use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type-safe tuple struct wrapping standard integers to enforce financial minor unit rules.
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MinorUnits(pub i64);

#[allow(dead_code)]
pub struct DbTransactionRow {
    pub id: Uuid,
    pub idempotency_key: String,
    pub description: String,
}

#[allow(dead_code)]
pub struct DbEntryRow {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub amount: MinorUnits,
}