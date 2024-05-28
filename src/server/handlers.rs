use anyhow::{Context, Result};
use rocket::http::{Method, Status};
use rocket::response::{content, status};
use rocket::{catch, catchers, get, patch, routes, Build, Request, Rocket, State};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use std::collections::HashSet;

use crate::heartbeat::service::Heartbeat;
use crate::quote::service::Service;

pub fn register_routes(
    builder: Rocket<Build>,
    h: Heartbeat,
    quotes: Service,
) -> Result<Rocket<Build>> {
    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: vec![Method::Get, Method::Patch, Method::Options]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        expose_headers: HashSet::from(["Link".to_string()]),
        max_age: Some(300),
        ..Default::default()
    }
    .to_cors()
    .context("failed to make cors")?;

    Ok(builder
        .attach(cors)
        .manage(h)
        .manage(quotes)
        .register("/", catchers![catch_default])
        .mount("/heartbeat", routes![heartbeat_handler])
        .mount("/", routes![get_quote_handler])
        .mount("/", routes![like_quote_handler])
        .mount("/", routes![get_same_quote_handler]))
}

#[catch(default)]
fn catch_default(status: Status, req: &Request) -> String {
    format!("{}: ({})", status, req.uri())
}

#[get("/")]
async fn heartbeat_handler(h: &State<Heartbeat>) -> Status {
    match h.ping_database().await {
        Ok(_) => Status::Ok,
        Err(err) => {
            log::error!("failed to ping database: {err}");
            Status::InternalServerError
        }
    }
}

#[get("/?<user_id>")]
async fn get_quote_handler(
    user_id: String,
    quotes: &State<Service>,
) -> status::Custom<Option<content::RawJson<String>>> {
    match quotes.get_quote(user_id).await {
        Ok(quote) => match serde_json::to_string(&quote) {
            Ok(value) => status::Custom(Status::Ok, Some(content::RawJson(value))),
            Err(err) => {
                log::error!("failed to serialize quote: {err}");
                status::Custom(Status::InternalServerError, None)
            }
        },
        Err(err) => {
            log::error!("failed to get quote: {err}");
            status::Custom(Status::InternalServerError, None)
        }
    }
}

#[patch("/like?<user_id>&<quote_id>")]
async fn like_quote_handler(quote_id: String, user_id: String, quotes: &State<Service>) -> Status {
    match quotes.like_quote(user_id, quote_id).await {
        Ok(_) => Status::Ok,
        Err(err) => {
            log::error!("failed to like quote: {err}");
            Status::InternalServerError
        }
    }
}

#[get("/same?<user_id>&<quote_id>")]
async fn get_same_quote_handler(
    quote_id: String,
    user_id: String,
    quotes: &State<Service>,
) -> status::Custom<Option<content::RawJson<String>>> {
    match quotes.get_same_quote(user_id, quote_id).await {
        Ok(quote) => match serde_json::to_string(&quote) {
            Ok(value) => status::Custom(Status::Ok, Some(content::RawJson(value))),
            Err(err) => {
                log::error!("failed to serialize quote: {err}");
                status::Custom(Status::InternalServerError, None)
            }
        },
        Err(err) => {
            log::error!("failed to get same quote: {err}");
            status::Custom(Status::InternalServerError, None)
        }
    }
}
