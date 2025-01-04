mod app;
mod config;
mod database;
mod heartbeat;
mod quote;
mod quote_api;
mod server;

use config::GlobalConfig;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cfg = GlobalConfig::get().expect("failed to get config");
    app::start(cfg).await
}
