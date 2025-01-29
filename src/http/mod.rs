use crate::config::Config;
use anyhow::Context;
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
pub mod error;
pub mod utils;
use mail_send::SmtpClientBuilder;
mod dependencies;
mod routers;

// Include auth router

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub smtp_builder: Arc<SmtpClientBuilder<String>>,
}

pub async fn serve(
    config: Config,
    db: PgPool,
    smtp_builder: SmtpClientBuilder<String>,
) -> anyhow::Result<()> {
    // Create shared state
    let shared_state = Arc::new(AppState {
        config: Arc::new(config),
        db,
        smtp_builder: Arc::new(smtp_builder),
    });

    // Build the app router
    let app = Router::new().with_state(shared_state.clone());
    let app = api_router(app, shared_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    // Start the server using the listener
    axum::serve(listener, app)
        .await
        .context("Error running the server")
}

// Define the API router
fn api_router(router: Router, shared_state: Arc<AppState>) -> Router {
    router
        .merge(routers::auth::router(shared_state.clone())) // Add auth router
        .merge(routers::user::router(shared_state.clone())) // Add user router
}
