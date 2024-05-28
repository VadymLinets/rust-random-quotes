pub mod config;
pub mod database;
mod heartbeat;
pub mod quote;
mod quoteapi;
mod server;

use std::sync::Arc;

pub async fn start(cfg: config::cfg::GlobalConfig) {
    let db = database::seaorm::SeaORM::new(cfg.orm_config)
        .await
        .expect("failed to init database");

    let db = Arc::new(db);
    let heartbeat = heartbeat::service::Heartbeat::new(db.clone());
    let quoteapi = quoteapi::service::Service::new(db.clone());
    let quote = quote::service::Service::new(cfg.quotes_config, db, Arc::new(quoteapi));

    server::srv::start(cfg.server_config, heartbeat, quote).await;
}
