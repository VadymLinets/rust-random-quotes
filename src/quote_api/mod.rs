mod structs;

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc;

use crate::{database::structs::quotes::Model as Quotes, quote};

#[cfg(test)]
use mockall::automock;

const RANDOM_QUOTE_URL: &str = "https://api.quotable.io/random";

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Database {
    async fn save_quote(&self, quote: Quotes) -> Result<()>;
}

pub struct Service {
    db: Arc<dyn Database + Send + Sync>,
    client: reqwest::Client,
    quote_url: String,
}

impl Service {
    pub async fn get_random_quote(&self) -> Result<Quotes> {
        let resp = self
            .client
            .get(&self.quote_url)
            .send()
            .await
            .context("failed to receive random quote from site")?;

        let data = resp
            .text()
            .await
            .context("failed to receive response text")?;

        let quote: structs::Quote =
            serde_json::from_str(&data).context("failed to deserialize random quote")?;

        let quote = structs::to_database(quote);
        self.db
            .save_quote(quote.to_owned())
            .await
            .context("failed to save new random quote")?;

        Ok(quote)
    }

    pub fn new(db: Arc<dyn Database + Send + Sync>) -> Self {
        Service {
            db,
            client: reqwest::Client::new(),
            quote_url: RANDOM_QUOTE_URL.to_string(),
        }
    }
}

#[async_trait]
impl quote::Api for Service {
    async fn get_random_quote(&self) -> Result<Quotes> {
        self.get_random_quote().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fake::{
        faker::{lorem, name},
        uuid, Fake, Faker,
    };
    use mockall::predicate::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_get_random_quote_success() {
        let quote: structs::Quote = structs::Quote {
            id: uuid::UUIDv4.fake(),
            content: lorem::en::Sentence(5..10).fake(),
            author: name::en::Name().fake(),
            tags: Faker.fake(),
        };

        let mut db = MockDatabase::new();
        db.expect_save_quote()
            .with(eq(structs::to_database(quote.clone())))
            .returning(|_| Ok(()));

        let raw_quote = match serde_json::to_string(&quote.clone()) {
            Ok(quote) => quote,
            Err(err) => panic!("{err}"),
        };

        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .with_body(raw_quote)
            .create_async()
            .await;

        let service = Service {
            db: Arc::new(db),
            client: reqwest::Client::new(),
            quote_url: server.url(),
        };

        let res = service.get_random_quote().await;
        assert!(!res.is_err(), "err = {}", res.err().unwrap());
        assert_eq!(res.unwrap(), structs::to_database(quote));
        mock.assert_async().await;
    }
}
