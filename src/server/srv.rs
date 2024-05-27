use crate::config::cfg::ServerConfig;
use crate::heartbeat::service::Heartbeat;
use crate::quote;
use crate::server::handlers;
use rocket::{build, Config};
use std::net::SocketAddr;

pub async fn start(cfg: ServerConfig, h: Heartbeat, quotes: quote::service::Service) {
    let addr: SocketAddr = cfg.addr.parse().expect("Failed to parse addresses");

    let config = Config {
        port: addr.port(),
        address: addr.ip(),
        ..Config::default()
    };

    let rocket = build().configure(&config);
    handlers::register_routes(rocket, h, quotes)
        .expect("Failed to register fairings")
        .launch()
        .await
        .expect("Failed to start server");
}
