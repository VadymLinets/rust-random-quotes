use actix_web::{get, patch, web, HttpResponse, Responder};

use crate::actix_server::structs;
use crate::heartbeat::service::Heartbeat;
use crate::quote::service::Service;

#[get("/heartbeat")]
async fn heartbeat_handler(heartbeat: web::Data<Heartbeat>) -> impl Responder {
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
    quotes: web::Data<Service>,
) -> impl Responder {
    match quotes.get_quote(&query.user_id).await {
        Ok(quote) => match serde_json::to_string(&quote) {
            Ok(value) => HttpResponse::Ok().body(value),
            Err(err) => {
                log::error!("failed to serialize quote: {err}");
                HttpResponse::InternalServerError().finish()
            }
        },
        Err(err) => {
            log::error!("failed to get quote: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[patch("/like")]
async fn like_quote_handler(
    query: web::Query<structs::UserAndQuoteID>,
    quotes: web::Data<Service>,
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
    quotes: web::Data<Service>,
) -> impl Responder {
    match quotes.get_same_quote(&query.user_id, &query.quote_id).await {
        Ok(quote) => match serde_json::to_string(&quote) {
            Ok(value) => HttpResponse::Ok().body(value),
            Err(err) => {
                log::error!("failed to serialize quote: {err}");
                HttpResponse::InternalServerError().finish()
            }
        },
        Err(err) => {
            log::error!("failed to get same quote: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
