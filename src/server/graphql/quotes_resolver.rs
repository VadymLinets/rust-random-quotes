use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode};

use super::quotes::{EmptyResult, Quote, QuoteResult};
use crate::heartbeat::Heartbeat as heartbeat_service;
use crate::quote::Service as quote_service;

pub struct Context {
    pub quotes: quote_service,
    pub heartbeat: heartbeat_service,
}

impl juniper::Context for Context {}

pub struct Query;

#[graphql_object]
#[graphql(context = Context)]
impl Query {
    async fn heartbeat(ctx: &Context) -> FieldResult<EmptyResult> {
        match ctx.heartbeat.ping_database().await {
            Ok(_) => Ok(EmptyResult {
                success: true,
                errors: vec![],
            }),
            Err(err) => Ok(EmptyResult {
                success: false,
                errors: vec![err.to_string()],
            }),
        }
    }

    #[graphql(name = "get_quote_handler")]
    async fn get_quote_handler(
        ctx: &Context,
        #[graphql(name = "user_id")] user_id: String,
    ) -> FieldResult<QuoteResult> {
        let quote = match ctx.quotes.get_quote(user_id.as_str()).await {
            Ok(quote) => quote,
            Err(err) => {
                return Ok(QuoteResult {
                    success: false,
                    errors: vec![err.to_string()],
                    quote: None,
                })
            }
        };

        Ok(QuoteResult {
            success: true,
            errors: vec![],
            quote: Some(Quote {
                id: quote.id,
                quote: quote.quote,
                author: quote.author,
                tags: quote.tags,
                likes: quote.likes,
            }),
        })
    }

    #[graphql(name = "get_same_quote_handler")]
    async fn get_same_quote_handler(
        ctx: &Context,
        #[graphql(name = "user_id")] user_id: String,
        #[graphql(name = "quote_id")] quote_id: String,
    ) -> FieldResult<QuoteResult> {
        let quote = match ctx
            .quotes
            .get_same_quote(user_id.as_str(), quote_id.as_str())
            .await
        {
            Ok(quote) => quote,
            Err(err) => {
                return Ok(QuoteResult {
                    success: false,
                    errors: vec![err.to_string()],
                    quote: None,
                })
            }
        };

        Ok(QuoteResult {
            success: true,
            errors: vec![],
            quote: Some(Quote {
                id: quote.id,
                quote: quote.quote,
                author: quote.author,
                tags: quote.tags,
                likes: quote.likes,
            }),
        })
    }
}

pub struct Mutation;

#[graphql_object]
#[graphql(context = Context)]
impl Mutation {
    #[graphql(name = "like_quote_handler")]
    async fn like_quote_handler(
        ctx: &Context,
        #[graphql(name = "user_id")] user_id: String,
        #[graphql(name = "quote_id")] quote_id: String,
    ) -> FieldResult<EmptyResult> {
        match ctx
            .quotes
            .like_quote(user_id.as_str(), quote_id.as_str())
            .await
        {
            Ok(_) => Ok(EmptyResult {
                success: true,
                errors: vec![],
            }),
            Err(err) => Ok(EmptyResult {
                success: false,
                errors: vec![err.to_string()],
            }),
        }
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;
