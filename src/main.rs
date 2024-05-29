mod config;
mod database;
mod heartbeat;
mod quote;
mod quoteapi;
mod server;

use std::sync::Arc;

use config::cfg::GlobalConfig;
use database::seaorm::SeaORM;
use heartbeat::service::Heartbeat;
use quote::service as quote_srv;
use quoteapi::service as quoteapi_srv;
use server::srv::start as start_server;

#[rocket::main]
async fn main() {
    let cfg = GlobalConfig::get().expect("failed to get config");

    let db = SeaORM::new(cfg.orm_config)
        .await
        .expect("failed to start database");

    let db = Arc::new(db);
    let heartbeat = Heartbeat::new(db.clone());
    let quoteapi = quoteapi_srv::Service::new(db.clone());
    let quote = quote_srv::Service::new(cfg.quotes_config, db, Arc::new(quoteapi));

    start_server(cfg.server_config, heartbeat, quote).await;
}
