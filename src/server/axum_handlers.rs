use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::{heartbeat::Heartbeat, quote::Service};

use super::structs;

pub async fn heartbeat_handler(heartbeat: State<Heartbeat>) -> StatusCode {
    match heartbeat.ping_database().await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            log::error!("failed to ping database: {err:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_quote_handler(
    query: Query<structs::UserID>,
    quotes: State<Service>,
) -> (StatusCode, Response) {
    match quotes.get_quote(&query.user_id).await {
        Ok(quote) => (StatusCode::OK, Json(quote).into_response()),
        Err(err) => {
            log::error!("failed to get quote: {err:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, "".into_response())
        }
    }
}

pub async fn like_quote_handler(
    query: Query<structs::UserAndQuoteID>,
    quotes: State<Service>,
) -> StatusCode {
    match quotes.like_quote(&query.user_id, &query.quote_id).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            log::error!("failed to like quote: {err:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_same_quote_handler(
    query: Query<structs::UserAndQuoteID>,
    quotes: State<Service>,
) -> (StatusCode, Response) {
    match quotes.get_same_quote(&query.user_id, &query.quote_id).await {
        Ok(quote) => (StatusCode::OK, Json(quote).into_response()),
        Err(err) => {
            log::error!("failed to get same quote: {err:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, "".into_response())
        }
    }
}
