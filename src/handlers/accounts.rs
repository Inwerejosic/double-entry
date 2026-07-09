use actix_web::{get, HttpResponse, Responder, web};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(serde::Serialize)]
struct BalanceResponse {
    account_id: String,
    balance: i64,
}

#[get("/v1/accounts/{account_id}/balance")]
#[tracing::instrument(skip(pool))]
pub async fn get_balance(
    pool: web::Data<Pool<Postgres>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let account_id = path.into_inner();

    // Ensure the SQL returns an INT8 to match Rust's i64 mapping.
    let sum: i64 = match sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(amount), 0)::BIGINT FROM entries WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(%e, "Failed to query account balance");
            return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed fetching balance"}));
        }
    };

    HttpResponse::Ok().json(BalanceResponse {
        account_id: account_id.to_string(),
        balance: sum,
    })
}
