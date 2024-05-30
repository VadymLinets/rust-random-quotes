mod handlers;
mod structs;

use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use anyhow::{Context, Result};
use env_logger::Env;
use std::net::SocketAddr;

use crate::config::cfg::ServerConfig;
use crate::heartbeat::service::Heartbeat;
use crate::quote::service::Service;

pub async fn start(cfg: ServerConfig, heartbeat: Heartbeat, quotes: Service) -> Result<Server> {
    let addr: SocketAddr = cfg.addr.parse().context("failed to parse address")?;

    let heartbeat = web::Data::new(heartbeat);
    let quotes = web::Data::new(quotes);

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(heartbeat.clone())
            .app_data(quotes.clone())
            .service(handlers::heartbeat_handler)
            .service(handlers::get_quote_handler)
            .service(handlers::like_quote_handler)
            .service(handlers::get_same_quote_handler)
    })
    .bind(addr)?
    .run())
}
