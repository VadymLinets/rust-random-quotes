use tonic::{Request, Response, Status};

use super::proto::quotes_server::Quotes;
use super::proto::{Empty, Quote, UserAndQuoteIdRequest, UserIdRequest};
use crate::heartbeat::Heartbeat;
use crate::quote::Service;

pub struct Grpc {
    heartbeat: Heartbeat,
    quotes: Service,
}

impl Grpc {
    pub fn new(heartbeat: Heartbeat, quotes: Service) -> Self {
        Grpc { heartbeat, quotes }
    }
}

#[tonic::async_trait]
impl Quotes for Grpc {
    async fn heartbeat(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        match self.heartbeat.ping_database().await {
            Ok(_) => Ok(Response::new(Empty {})),
            Err(err) => {
                log::error!("failed to ping database: {err}");
                Err(Status::new(
                    tonic::Code::Internal,
                    "failed to ping database",
                ))
            }
        }
    }

    async fn get_quote_handler(
        &self,
        request: Request<UserIdRequest>,
    ) -> Result<Response<Quote>, Status> {
        let req = request.into_inner();
        match self.quotes.get_quote(req.user_id.as_str()).await {
            Ok(quote) => Ok(Response::new(Quote {
                id: quote.id,
                quote: quote.quote,
                author: quote.author,
                tags: quote.tags,
                likes: quote.likes as i64,
            })),
            Err(err) => {
                log::error!("failed to get quote: {err}");
                Err(Status::new(tonic::Code::Internal, "failed to get quote"))
            }
        }
    }

    async fn get_same_quote_handler(
        &self,
        request: Request<UserAndQuoteIdRequest>,
    ) -> Result<Response<Quote>, Status> {
        let req = request.into_inner();
        match self
            .quotes
            .get_same_quote(req.user_id.as_str(), req.quote_id.as_str())
            .await
        {
            Ok(quote) => Ok(Response::new(Quote {
                id: quote.id,
                quote: quote.quote,
                author: quote.author,
                tags: quote.tags,
                likes: quote.likes as i64,
            })),
            Err(err) => {
                log::error!("failed to get same quote: {err}");
                Err(Status::new(
                    tonic::Code::Internal,
                    "failed to get same quote",
                ))
            }
        }
    }

    async fn like_quote_handler(
        &self,
        request: Request<UserAndQuoteIdRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        match self
            .quotes
            .like_quote(req.user_id.as_str(), req.quote_id.as_str())
            .await
        {
            Ok(_) => Ok(Response::new(Empty {})),
            Err(err) => {
                log::error!("failed to like quote: {err}");
                Err(Status::new(tonic::Code::Internal, "failed to like quote"))
            }
        }
    }
}
