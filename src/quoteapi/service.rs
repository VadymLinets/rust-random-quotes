use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{database::structs::quotes::Model as Quotes, quote};

use super::structs;

const RANDOM_QUOTE_URL: &str = "https://api.quotable.io/random";

#[async_trait]
pub trait Database {
    async fn save_quote(&self, quote: Quotes) -> Result<()>;
}

pub struct Service {
    db: Arc<Mutex<dyn Database + Send>>,
    client: reqwest::Client,
}

impl Service {
    pub async fn get_random_quote(&self) -> Result<Quotes> {
        let resp = self.client.get(RANDOM_QUOTE_URL).send().await?;
        let bytes = resp.bytes().await?;
        let quote: structs::Quote = serde_json::from_slice(bytes.to_vec().as_slice())?;
        let quote = structs::to_database(quote);
        self.db.lock().await.save_quote(quote.clone()).await?;
        Ok(quote)
    }

    pub fn new(db: Arc<Mutex<dyn Database + Send>>) -> Self {
        Service {
            db,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl quote::service::Api for Service {
    async fn get_random_quote(&self) -> Result<Quotes> {
        self.get_random_quote().await
    }
}
