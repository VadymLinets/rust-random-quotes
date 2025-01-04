use anyhow::{Context, Result};
use juniper::EmptySubscription;
use juniper_rocket::{GraphQLRequest, GraphQLResponse};
use rocket::http::{Method, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{catch, catchers, get, patch, post, routes, Build, Request, Rocket, State};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use std::collections::HashSet;

use super::graphql::quotes_resolver::{Context as graphql_context, Mutation, Query, Schema};
use crate::heartbeat::Heartbeat;
use crate::quote::structs::Quote;
use crate::quote::Service;

pub fn register_routes(
    builder: Rocket<Build>,
    heartbeat: Heartbeat,
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
        .manage(heartbeat)
        .manage(quotes)
        .manage(Schema::new(Query, Mutation, EmptySubscription::new()))
        .register("/", catchers![catch_default])
        .mount("/heartbeat", routes![heartbeat_handler])
        .mount("/", routes![get_quote_handler])
        .mount("/", routes![like_quote_handler])
        .mount("/", routes![get_same_quote_handler])
        .mount("/", routes![get_graphql])
        .mount("/", routes![post_graphql]))
}

#[catch(default)]
fn catch_default(status: Status, req: &Request) -> String {
    format!("{}: ({})", status, req.uri())
}

#[get("/")]
async fn heartbeat_handler(heartbeat: &State<Heartbeat>) -> Status {
    match heartbeat.ping_database().await {
        Ok(_) => Status::Ok,
        Err(err) => {
            log::error!("failed to ping database: {:#}", err);
            Status::InternalServerError
        }
    }
}

#[get("/?<user_id>")]
async fn get_quote_handler(
    user_id: String,
    quotes: &State<Service>,
) -> status::Custom<Option<Json<Quote>>> {
    match quotes.get_quote(&user_id).await {
        Ok(quote) => status::Custom(Status::Ok, Some(Json(quote))),
        Err(err) => {
            log::error!("failed to get quote: {:#}", err);
            status::Custom(Status::InternalServerError, None)
        }
    }
}

#[patch("/like?<user_id>&<quote_id>")]
async fn like_quote_handler(quote_id: String, user_id: String, quotes: &State<Service>) -> Status {
    match quotes.like_quote(&user_id, &quote_id).await {
        Ok(_) => Status::Ok,
        Err(err) => {
            log::error!("failed to like quote: {:#}", err);
            Status::InternalServerError
        }
    }
}

#[get("/same?<user_id>&<quote_id>")]
async fn get_same_quote_handler(
    quote_id: String,
    user_id: String,
    quotes: &State<Service>,
) -> status::Custom<Option<Json<Quote>>> {
    match quotes.get_same_quote(&user_id, &quote_id).await {
        Ok(quote) => status::Custom(Status::Ok, Some(Json(quote))),
        Err(err) => {
            log::error!("failed to get same quote: {:#}", err);
            status::Custom(Status::InternalServerError, None)
        }
    }
}

#[get("/graphql?<request..>")]
async fn get_graphql<'a>(
    quotes: &State<Service>,
    heartbeat: &State<Heartbeat>,
    request: GraphQLRequest,
    schema: &State<Schema>,
) -> GraphQLResponse {
    request
        .execute(
            schema,
            &graphql_context {
                quotes: quotes.inner().clone(),
                heartbeat: heartbeat.inner().clone(),
            },
        )
        .await
}

#[post("/graphql", data = "<request>")]
async fn post_graphql<'a>(
    quotes: &State<Service>,
    heartbeat: &State<Heartbeat>,
    request: GraphQLRequest,
    schema: &State<Schema>,
) -> GraphQLResponse {
    request
        .execute(
            schema,
            &graphql_context {
                quotes: quotes.inner().clone(),
                heartbeat: heartbeat.inner().clone(),
            },
        )
        .await
}
