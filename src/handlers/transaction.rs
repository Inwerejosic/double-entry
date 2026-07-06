use actix_web::{post, HttpResponse, Responder, web};
use actix_web_validator::Json;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::dto::transaction::TransactionInput;

#[post("/v1/transactions")]
#[tracing::instrument(skip(pool, payload), fields(idempotency_key = %payload.idempotency_key))]
pub async fn execute_transaction(
    pool: web::Data<Pool<Postgres>>,
    payload: Json<TransactionInput>,
) -> impl Responder {
    let tx_data = payload.into_inner();
    let target_tx_id = Uuid::new_v4();

    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            tracing::error!("Failed to initialize database transaction layer context: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Database internal connection fault" }));
        }
    };

    let mock_session_user = "api_operator@firm.io";
    if let Err(e) = sqlx::query("SET LOCAL \"app.current_user\" = $1")
        .bind(mock_session_user)
        .execute(&mut *tx)
        .await
    {
        tracing::error!("Failed to assign application user metadata session parameter: {:?}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Audit configuration failure" }));
    }

    if let Err(e) = sqlx::query(
            "INSERT INTO transactions (id, idempotency_key, description) VALUES ($1, $2, $3)",
        )
        .bind(target_tx_id)
        .bind(tx_data.idempotency_key)
        .bind(tx_data.description)
        .execute(&mut *tx)
        .await
    {
        if let Some(db_err) = e.as_database_error() {
            if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "error": "Idempotency match signature caught. Transaction dropped to prevent duplicates."
                }));
            }
        }

        tracing::error!("Failed processing ledger transaction mapping header: {:?}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed writing transaction" }));
    }

    for entry in tx_data.entries {
        let entry_id = Uuid::new_v4();
        if let Err(e) = sqlx::query(
                "INSERT INTO entries (id, transaction_id, account_id, amount) VALUES ($1, $2, $3, $4)",
            )
            .bind(entry_id)
            .bind(target_tx_id)
            .bind(entry.account_id)
            .bind(entry.amount)
            .execute(&mut *tx)
            .await
        {
            tracing::error!("System failed atomic application loop constraints execution on entry details: {:?}", e);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Ledger constraints fault: Account reference target could be invalid or non-existent."
            }));
        }
    }

    if let Err(e) = tx.commit().await {
        tracing::error!("Transaction failed to write state down to disk blocks: {:?}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Disk serialization write commit failed" }));
    }

    HttpResponse::Created().json(serde_json::json!({
        "status": "success",
        "transaction_id": target_tx_id.to_string(),
    }))
}
