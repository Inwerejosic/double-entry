use serde::Deserialize;
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct EntryLineInput {
    pub account_id: Uuid,
    
    // Validates that entries never pass an absolute zero metric down to the database ledger
    #[validate(custom(function = "validate_non_zero"))]
    pub amount: i64,
}

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_double_entry_balance"))]
pub struct TransactionInput {
    #[validate(length(min = 4, message = "Idempotency key string footprint must be longer"))]
    pub idempotency_key: String,

    #[validate(length(min = 5, message = "Transaction descriptions must have clear business context"))]
    pub description: String,

    pub entries: Vec<EntryLineInput>,
}

fn validate_non_zero(amount: i64) -> Result<(), ValidationError> {
    if amount == 0 {
        return Err(ValidationError::new("Zero-Value Clutter Rejection: Entry values cannot accept absolute zero metrics."));
    }
    Ok(())
}

fn validate_double_entry_balance(input: &TransactionInput) -> Result<(), ValidationError> {
    let sum: i64 = input.entries.iter().map(|e| e.amount).sum();
    if sum != 0 {
        let mut err = ValidationError::new("Unbalanced Transaction context");
        err.message = Some(std::borrow::Cow::Owned(format!(
            "Expected balanced net sum 0. Evaluated state resulted in: {}", sum
        )));
        return Err(err);
    }
    Ok(())
}