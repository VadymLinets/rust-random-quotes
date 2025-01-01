use crate::database::structs::quotes::Model as Quotes;
use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Database {
    async fn save_quote(&self, quote: Quotes) -> anyhow::Result<()>;
}
