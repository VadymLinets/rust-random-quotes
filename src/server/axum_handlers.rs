use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};

use crate::{
    heartbeat::Heartbeat,
    quote::{structs::Quote, Service},
};

use super::structs;

pub async fn heartbeat_handler(heartbeat: State<Heartbeat>) -> StatusCode {
    match heartbeat.ping_database().await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            log::error!("failed to ping database: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_quote_handler(
    query: Query<structs::UserID>,
    quotes: State<Service>,
) -> (StatusCode, Json<Quote>) {
    match quotes.get_quote(&query.user_id).await {
        Ok(quote) => (StatusCode::OK, Json(quote)),
        Err(err) => {
            log::error!("failed to get quote: {err}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json::default())
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
            log::error!("failed to like quote: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_same_quote_handler(
    query: Query<structs::UserAndQuoteID>,
    quotes: State<Service>,
) -> (StatusCode, Json<Quote>) {
    match quotes.get_same_quote(&query.user_id, &query.quote_id).await {
        Ok(quote) => (StatusCode::OK, Json(quote)),
        Err(err) => {
            log::error!("failed to get same quote: {err}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json::default())
        }
    }
}
