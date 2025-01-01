use anyhow::Result;
use async_trait::async_trait;

use crate::database::structs::quotes::Model as Quotes;
use crate::database::structs::views::Model as Views;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Database {
    async fn get_quote(&self, quote_id: &str) -> Result<Quotes>;
    async fn get_quotes(&self, user_id: &str) -> Result<Vec<Quotes>>;
    async fn get_same_quote(&self, user_id: &str, viewed_quote: &Quotes) -> Result<Quotes>;
    async fn get_view(&self, user_id: &str, quote_id: &str) -> Result<Views>;
    async fn mark_as_viewed(&self, user_id: &str, quote_id: &str) -> Result<()>;
    async fn mark_as_liked(&self, user_id: &str, quote_id: &str) -> Result<()>;
    async fn like_quote(&self, quote_id: &str) -> Result<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Api {
    async fn get_random_quote(&self) -> Result<Quotes>;
}
