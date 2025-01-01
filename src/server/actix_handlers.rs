use actix_web::web::Data;
use actix_web::{get, patch, web, Error, HttpRequest, HttpResponse, Responder};
use juniper_actix::graphql_handler;

use super::graphql::quotes_resolver::{Context as graphql_context, Schema};
use crate::heartbeat::Heartbeat;
use crate::quote::Service;
use crate::server::structs;

#[get("/heartbeat")]
async fn heartbeat_handler(heartbeat: Data<Heartbeat>) -> impl Responder {
    match heartbeat.ping_database().await {
        Ok(_) => HttpResponse::Ok(),
        Err(err) => {
            log::error!("failed to ping database: {err}");
            HttpResponse::InternalServerError()
        }
    }
}

#[get("/")]
async fn get_quote_handler(
    query: web::Query<structs::UserID>,
    quotes: Data<Service>,
) -> impl Responder {
    match quotes.get_quote(&query.user_id).await {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(err) => {
            log::error!("failed to get quote: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[patch("/like")]
async fn like_quote_handler(
    query: web::Query<structs::UserAndQuoteID>,
    quotes: Data<Service>,
) -> impl Responder {
    match quotes.like_quote(&query.user_id, &query.quote_id).await {
        Ok(_) => HttpResponse::Ok(),
        Err(err) => {
            log::error!("failed to like quote: {err}");
            HttpResponse::InternalServerError()
        }
    }
}

#[get("/same")]
async fn get_same_quote_handler(
    query: web::Query<structs::UserAndQuoteID>,
    quotes: Data<Service>,
) -> impl Responder {
    match quotes.get_same_quote(&query.user_id, &query.quote_id).await {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(err) => {
            log::error!("failed to get same quote: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn graphql(
    req: HttpRequest,
    payload: web::Payload,
    schema: Data<Schema>,
    quotes: Data<Service>,
    heartbeat: Data<Heartbeat>,
) -> Result<HttpResponse, Error> {
    graphql_handler(
        &schema,
        &graphql_context {
            quotes: quotes.get_ref().clone(),
            heartbeat: heartbeat.get_ref().clone(),
        },
        req,
        payload,
    )
    .await
}
