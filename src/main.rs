use anyhow::Context; // Needed for context to work
use clap::Parser; // Needed for parse to work
use sqlx::postgres::PgPoolOptions;

use rust_backend::config::Config;
use rust_backend::http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check if .env file exists, init logger, loda config
    dotenv::dotenv().ok();
    env_logger::init();
    let config = Config::parse();

    // Create DB pool
    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database_url)
        .await
        .context("could not connect to database_url")?;

    http::serve(config, db).await.unwrap();

    Ok(())
}
