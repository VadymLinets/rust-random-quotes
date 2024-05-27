mod config;
mod database;
mod heartbeat;
mod quote;
mod quoteapi;
mod server;

use std::sync::Arc;
use tokio::sync::Mutex;

#[rocket::main]
async fn main() {
    let cfg = config::cfg::GlobalConfig::get().expect("failed to get config");

    let db = database::seaorm::SeaORM::new(cfg.orm_config)
        .await
        .expect("failed to init database");

    let db = Arc::new(Mutex::new(db));
    let heartbeat = heartbeat::service::Heartbeat::new(db.clone());
    let quoteapi = quoteapi::service::Service::new(db.clone());
    let quote = quote::service::Service::new(cfg.quotes_config, db, Arc::new(Mutex::new(quoteapi)));

    server::srv::start(cfg.server_config, heartbeat, quote).await;
}
