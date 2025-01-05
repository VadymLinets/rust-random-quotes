use std::sync::Arc;
use tokio::signal;

use crate::config::GlobalConfig;
use crate::database::seaorm::SeaORM;
use crate::heartbeat;
use crate::quote;
use crate::quote_api;
use crate::server;

pub async fn start(cfg: GlobalConfig) {
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
    } else if cfg.server_config.service_type.eq("axum") {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .compact()
            .init();

        let (listener, app) = server::start_axum(&cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to create server");

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .expect("failed to start server");
    } else if cfg.server_config.service_type.eq("grpc") {
        server::start_grpc(&cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to create server");
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
