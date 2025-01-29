use anyhow::Context; // Needed for context to work
use clap::Parser; // Needed for parse to work
use mail_send::SmtpClientBuilder;
use rust_backend::config::Config;
use rust_backend::http;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check if .env file exists, init logger, loda config
    dotenv::dotenv().ok();
    env_logger::init();

    let config = Config::parse();
    todo!("Wrap the smtp in deadpool, connect and auto reconnect");
    let smtp_builder =
        SmtpClientBuilder::new(config.mail_host.to_string(), config.mail_port.clone()).credentials(
            (
                config.mail_username.to_string(),
                config.mail_password.to_string(),
            ),
        );
    // Create DB pool
    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database_url)
        .await
        .context("could not connect to database_url")?;

    http::serve(config, db, smtp_builder).await.unwrap();

    Ok(())
}
