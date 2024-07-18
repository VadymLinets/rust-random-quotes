mod proto {
    include!("proto/quotes.rs");
}
mod actix_handlers;
mod graphql;
mod grpc_handlers;
mod rocket_handlers;
mod structs;

use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use anyhow::{Context, Result};
use env_logger::Env;
use proto::quotes_server::QuotesServer;
use rocket::{build, Config};
use std::net::SocketAddr;
use juniper::EmptySubscription;

use crate::config::ServerConfig;
use crate::heartbeat::Heartbeat;
use crate::quote::Service;
use crate::server::grpc_handlers::Grpc;
use crate::server::graphql::quotes_resolver::{Mutation, Query, Schema};

pub async fn start_rocket(cfg: &ServerConfig, heartbeat: Heartbeat, quotes: Service) -> Result<()> {
    let addr: SocketAddr = cfg.addr.parse().context("failed to parse address")?;

    let config = Config {
        port: addr.port(),
        address: addr.ip(),
        ..Config::default()
    };

    let rocket = build().configure(&config);
    rocket_handlers::register_routes(rocket, heartbeat, quotes)
        .context("failed to register fairings")?
        .launch()
        .await
        .context("failed to start server")?;

    Ok(())
}

pub async fn start_actix(
    cfg: &ServerConfig,
    heartbeat: Heartbeat,
    quotes: Service,
) -> Result<Server> {
    let addr: SocketAddr = cfg.addr.parse().context("failed to parse address")?;

    let heartbeat = web::Data::new(heartbeat);
    let quotes = web::Data::new(quotes);

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    Ok(HttpServer::new(move || {
        let schema = web::Data::new(Schema::new(Query, Mutation, EmptySubscription::new()));
        App::new()
            .wrap(Logger::default())
            .app_data(heartbeat.clone())
            .app_data(quotes.clone())
            .app_data(schema)
            .service(actix_handlers::heartbeat_handler)
            .service(actix_handlers::get_quote_handler)
            .service(actix_handlers::like_quote_handler)
            .service(actix_handlers::get_same_quote_handler)
            .service(
                web::resource("/graphql")
                    .route(web::post().to(actix_handlers::graphql))
                    .route(web::get().to(actix_handlers::graphql)),
            )
    })
    .bind(addr)?
    .run())
}

pub async fn start_grpc(cfg: &ServerConfig, heartbeat: Heartbeat, quotes: Service) -> Result<()> {
    let addr: SocketAddr = cfg.addr.parse().context("failed to parse address")?;
    let srv = Grpc::new(heartbeat, quotes);

    println!("GreeterServer listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(QuotesServer::new(srv))
        .serve(addr)
        .await
        .context("Failed to start grpc server")?;

    Ok(())
}
