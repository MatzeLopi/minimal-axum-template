use crate::{config::Config, SmtpManager};
use anyhow::Context;
use axum::Router;
use deadpool::managed::Pool;
use sqlx::PgPool;
use std::sync::Arc;

mod dependencies;
pub mod error;
mod routers;
pub mod utils;

// Include auth router

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub smtp_pool: Arc<Pool<SmtpManager>>,
}

pub async fn serve(config: Config, db: PgPool, smtp_pool: Pool<SmtpManager>) -> anyhow::Result<()> {
    // Create shared state
    let shared_state = Arc::new(AppState {
        config: Arc::new(config),
        db,
        smtp_pool: Arc::new(smtp_pool),
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
