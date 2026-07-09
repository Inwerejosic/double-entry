use std::time::Instant;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use actix_web_validator::JsonConfig;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod dto;
mod dao;
mod handlers;

#[derive(Clone)]
struct AppState {
    started_at: Instant,
}

impl AppState {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    uptime_seconds: u64,
}

async fn health(state: web::Data<AppState>) -> impl Responder {
    let uptime_seconds = state.started_at.elapsed().as_secs();
    info!("health check requested; uptime_seconds={uptime_seconds}");

    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        uptime_seconds,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

    info!("Initializing asynchronous SQLx connection pools...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Fatal Error: Could not connect to the database storage cluster backend.");

    let state = AppState::new();
    info!("Starting Actix HTTP Server framework cluster on port 8080...");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(state.clone()))
            .app_data(JsonConfig::default().error_handler(|err, _req| {
                actix_web::error::ErrorBadRequest(serde_json::json!({
                    "error": "ValidationError",
                    "details": err.to_string(),
                }))
            }))
            .wrap(TracingLogger::default())
            .route("/", web::get().to(health))
            .route("/health", web::get().to(health))
            .route("/uptime", web::get().to(health))
            .service(handlers::transaction::execute_transaction)
            .service(handlers::accounts::get_balance)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
