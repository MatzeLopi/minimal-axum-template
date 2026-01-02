#[derive(clap::Parser)]
pub struct Config {
    #[clap(long, env)]
    pub database_url: String,

    #[clap(long, env)]
    pub hmac_key: String,

    #[clap(long, env)]
    pub mail_sender: String,

    #[clap(long, env)]
    pub mail_from: String,

    #[clap(long, env)]
    pub mail_host: String,

    #[clap(long, env)]
    pub mail_port: u16,

    #[clap(long, env)]
    pub mail_username: String,

    #[clap(long, env)]
    pub mail_password: String,
}
