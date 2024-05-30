mod actix_server;
pub mod config;
pub mod database;
mod heartbeat;
pub mod quote;
mod quote_api;
mod server;

use std::sync::Arc;

use config::GlobalConfig;
use database::seaorm::SeaORM;

pub async fn start(cfg: GlobalConfig) {
    let db = SeaORM::new(cfg.orm_config)
        .await
        .expect("failed to init database");

    let db = Arc::new(db);
    let heartbeat = heartbeat::Heartbeat::new(db.clone());
    let quote_api = quote_api::Service::new(db.clone());
    let quote = quote::Service::new(cfg.quotes_config, db, Box::new(quote_api));

    if cfg.server_config.service_type.eq("actix") {
        actix_server::start(cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to create server")
            .await
            .expect("failed to start server");
    } else {
        server::start(cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to start server");
    }
}
