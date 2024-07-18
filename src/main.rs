mod config;
mod database;
mod heartbeat;
mod quote;
mod quote_api;
mod server;

use std::sync::Arc;

use config::GlobalConfig;
use database::seaorm::SeaORM;

#[tokio::main]
async fn main() {
    let cfg = GlobalConfig::get().expect("failed to get config");

    let db = SeaORM::new(&cfg.orm_config)
        .await
        .expect("failed to start database");

    let db = Arc::new(db);
    let heartbeat = heartbeat::Heartbeat::new(db.clone());
    let quote_api = quote_api::Service::new(db.clone());
    let quote = quote::Service::new(&cfg.quotes_config, db, Arc::new(quote_api));

    if cfg.server_config.service_type.eq("actix") {
        server::start_actix(&cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to create server")
            .await
            .expect("failed to start server");
    } else if cfg.server_config.service_type.eq("rocket") {
        server::start_rocket(&cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to create server");
    } else if cfg.server_config.service_type.eq("grpc") {
        server::start_grpc(&cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to create server");
    }
}
