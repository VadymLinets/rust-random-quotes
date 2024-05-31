mod handlers;

use anyhow::{Context, Result};
use rocket::{build, Config};
use std::net::SocketAddr;

use crate::config::ServerConfig;
use crate::heartbeat::Heartbeat;
use crate::quote::Service;

pub async fn start(cfg: &ServerConfig, heartbeat: Heartbeat, quotes: Service) -> Result<()> {
    let addr: SocketAddr = cfg.addr.parse().context("failed to parse address")?;

    let config = Config {
        port: addr.port(),
        address: addr.ip(),
        ..Config::default()
    };

    let rocket = build().configure(&config);
    handlers::register_routes(rocket, heartbeat, quotes)
        .context("failed to register fairings")?
        .launch()
        .await
        .context("failed to start server")?;

    Ok(())
}
