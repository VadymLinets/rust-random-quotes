pub mod config;
pub mod database;
mod heartbeat;
pub mod quote;
mod quote_api;
mod server;

use std::sync::Arc;

use config::cfg::GlobalConfig;
use database::seaorm::SeaORM;
use heartbeat::service::Heartbeat;
use quote::service as quote_srv;
use quote_api::service as quote_api_srv;
use server::start as start_server;

pub async fn start(cfg: GlobalConfig) {
    let db = SeaORM::new(cfg.orm_config)
        .await
        .expect("failed to init database");

    let db = Arc::new(db);
    let heartbeat = Heartbeat::new(db.clone());
    let quote_api = quote_api_srv::Service::new(db.clone());
    let quote = quote_srv::Service::new(cfg.quotes_config, db, Box::new(quote_api));

    start_server(cfg.server_config, heartbeat, quote).await;
}
