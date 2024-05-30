mod handlers;

use rocket::{build, Config};
use std::net::SocketAddr;

use crate::config::cfg::ServerConfig;
use crate::heartbeat::service::Heartbeat;
use crate::quote;

pub async fn start(cfg: ServerConfig, h: Heartbeat, quotes: quote::service::Service) {
    let addr: SocketAddr = cfg.addr.parse().expect("failed to parse addresses");

    let config = Config {
        port: addr.port(),
        address: addr.ip(),
        ..Config::default()
    };

    let rocket = build().configure(&config);
    handlers::register_routes(rocket, h, quotes)
        .expect("failed to register fairings")
        .launch()
        .await
        .expect("failed to start server");
}