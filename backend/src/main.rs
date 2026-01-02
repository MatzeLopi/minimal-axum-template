use anyhow::Context; // Needed for context to work
use clap::Parser; // Needed for parse to work
use deadpool::managed::Pool;
use rust_backend::http;
use rust_backend::{config::Config, SmtpManager};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check if .env file exists, init logger, loda config
    dotenv::dotenv().ok();
    env_logger::init();

    // Load config
    let config = Config::parse();

    // Create SMTP pool
    let smtp_manager = SmtpManager {
        host: config.mail_host.clone(),
        port: config.mail_port,
        username: config.mail_username.clone(),
        password: config.mail_password.clone(),
    };

    let smtp_pool: Pool<SmtpManager> = Pool::builder(smtp_manager).max_size(10).build().unwrap();

    // Create DB pool
    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database_url)
        .await
        .context("could not connect to database_url")?;

    // Migrate the DB
    sqlx::migrate!()
        .run(&db)
        .await
        .context("could not run migrations")?;

    // Start Server
    http::serve(config, db).await.unwrap();

    Ok(())
}
