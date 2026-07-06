use std::time::Instant;

use actix_web::{
    App, HttpResponse, HttpServer, Responder, web,
};
use serde::Serialize;
use tracing::info;

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
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    let state = AppState::new();
    info!("starting server on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/", web::get().to(health))
            .route("/health", web::get().to(health))
            .route("/uptime", web::get().to(health))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
