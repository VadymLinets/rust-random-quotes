use anyhow::Result;
use async_trait::async_trait;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::cfg::QuotesConfig;
use crate::database::seaorm::Errors;
use crate::database::structs::quotes::Model as Quotes;
use crate::database::structs::views::Model as Views;

use super::structs::{self, from_database_quote_to_quote};

#[async_trait]
pub trait Database {
    async fn get_quote(&self, quote_id: String) -> Result<Quotes>;
    async fn get_quotes(&self, user_id: String) -> Result<Vec<Quotes>>;
    async fn get_same_quote(&self, user_id: String, viewed_quote: Quotes) -> Result<Quotes>;
    async fn get_view(&self, user_id: String, quote_id: String) -> Result<Views>;
    async fn mark_as_viewed(&self, user_id: String, quote_id: String) -> Result<()>;
    async fn mark_as_liked(&self, user_id: String, quote_id: String) -> Result<()>;
    async fn like_quote(&self, quote_id: String) -> Result<()>;
}

#[async_trait]
pub trait Api {
    async fn get_random_quote(&self) -> Result<Quotes>;
}

pub struct Service {
    cfg: QuotesConfig,
    db: Arc<Mutex<dyn Database + Send>>,
    api: Arc<Mutex<dyn Api + Send>>,
}

impl Service {
    pub async fn get_quote(&self, user_id: String) -> Result<structs::Quote> {
        let db = self.db.lock().await;
        let quotes = db.get_quotes(user_id.clone()).await?;
        let quote = self.randomize_quote(quotes).await?;
        db.mark_as_viewed(user_id, quote.clone().id).await?;
        Ok(from_database_quote_to_quote(quote))
    }

    pub async fn like_quote(&self, user_id: String, quote_id: String) -> Result<()> {
        let db = self.db.lock().await;
        let view = db.get_view(user_id.clone(), quote_id.clone()).await?;
        if view.liked {
            return Ok(());
        }
        db.like_quote(quote_id.clone()).await?;
        db.mark_as_liked(user_id, quote_id).await?;
        Ok(())
    }

    pub async fn get_same_quote(
        &self,
        user_id: String,
        quote_id: String,
    ) -> Result<structs::Quote> {
        let db = self.db.lock().await;
        let quote = db.get_quote(quote_id.clone()).await?;
        let quote = match db.get_same_quote(user_id.clone(), quote).await {
            Ok(quote) => quote,
            Err(err) => match err.downcast_ref::<Errors>() {
                Some(Errors::ErrNotFound) => self.api.lock().await.get_random_quote().await?,
                None => return Err(err),
            },
        };
        db.mark_as_viewed(user_id, quote_id).await?;
        Ok(from_database_quote_to_quote(quote))
    }

    pub fn new(
        cfg: QuotesConfig,
        db: Arc<Mutex<dyn Database + Send>>,
        api: Arc<Mutex<dyn Api + Send>>,
    ) -> Self {
        Service { cfg, db, api }
    }

    async fn randomize_quote(&self, quotes: Vec<Quotes>) -> Result<Quotes> {
        let random_percent = rand::thread_rng().gen_range(0.0..101.0);
        if (100.0 - self.cfg.random_quote_chance) > random_percent && !quotes.is_empty() {
            let mut likes_count: f64 = 0.0;
            for mut q in quotes.clone() {
                if q.likes == 0 {
                    q.likes += 1;
                }

                likes_count += q.likes as f64;
            }

            let mut accumulator: f64 = 0.0;
            let del = likes_count * 100.0 / (100.0 - self.cfg.random_quote_chance);
            for (i, q) in quotes.iter().enumerate() {
                let mut likes = q.likes;
                if likes == 0 {
                    likes += 1;
                }

                let percent = likes as f64 / del * 100.0;
                if percent + accumulator >= random_percent {
                    return Ok(quotes[i].to_owned());
                }

                accumulator += percent
            }
        }

        self.api.lock().await.get_random_quote().await
    }
}
