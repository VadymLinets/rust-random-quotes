pub mod structs;
pub mod traits;

use anyhow::{Context, Result};
use rand::Rng;
use std::sync::Arc;

use crate::config::QuotesConfig;
use crate::database::errors::Error as DatabaseErrors;
use crate::database::structs::quotes::Model as Quotes;

use structs::from_database_quote_to_quote;
pub use traits::{Api, Database};

const ONE_HUNDRED_PERCENT: f64 = 100.0;

#[derive(Clone)]
pub struct Service {
    cfg: QuotesConfig,
    db: Arc<dyn Database + Send + Sync>,
    api: Arc<dyn Api + Send + Sync>,
}

impl Service {
    pub async fn get_quote(&self, user_id: &str) -> Result<structs::Quote> {
        let quotes = self
            .db
            .get_quotes(user_id)
            .await
            .context("failed to get quotes")?;

        let quote = self
            .randomize_quote(&quotes)
            .await
            .context("failed to get random quote")?;

        self.db
            .mark_as_viewed(user_id, &quote.id)
            .await
            .context("failed to mark as viewed")?;

        Ok(from_database_quote_to_quote(quote))
    }

    pub async fn like_quote(&self, user_id: &str, quote_id: &str) -> Result<()> {
        let view = self
            .db
            .get_view(user_id, quote_id)
            .await
            .context("failed to get view")?;

        if view.liked {
            return Ok(());
        }

        self.db
            .like_quote(quote_id)
            .await
            .context("failed to like quote")?;

        self.db
            .mark_as_liked(user_id, quote_id)
            .await
            .context("failed to mark as liked")?;

        Ok(())
    }

    pub async fn get_same_quote(&self, user_id: &str, quote_id: &str) -> Result<structs::Quote> {
        let viewed_quote = self
            .db
            .get_quote(quote_id)
            .await
            .context("failed to get viewed quote")?;

        let quote = match self.db.get_same_quote(user_id, &viewed_quote).await {
            Ok(quote) => quote,
            Err(err) => match err.downcast_ref::<DatabaseErrors>() {
                Some(DatabaseErrors::ErrNotFound) => self
                    .api
                    .get_random_quote()
                    .await
                    .context("failed to get random quote")?,
                _ => return Err(err.context("failed to get same quote")),
            },
        };

        self.db
            .mark_as_viewed(user_id, quote.id.as_str())
            .await
            .context("failed to mark as viewed")?;

        Ok(from_database_quote_to_quote(quote))
    }

    pub fn new(
        cfg: &QuotesConfig,
        db: Arc<dyn Database + Send + Sync>,
        api: Arc<dyn Api + Send + Sync>,
    ) -> Self {
        Service {
            cfg: cfg.to_owned(),
            db,
            api,
        }
    }

    async fn randomize_quote(&self, quotes: &[Quotes]) -> Result<Quotes> {
        let random_percent = rand::rng().random_range(0.0..101.0);
        if (ONE_HUNDRED_PERCENT - self.cfg.random_quote_chance) > random_percent
            && !quotes.is_empty()
        {
            let likes_count = quotes.iter().fold(0.0, |acc, q| {
                acc + if q.likes == 0 { 1.0 } else { q.likes as f64 }
            });

            let mut accumulator = 0.0;
            let del = likes_count * ONE_HUNDRED_PERCENT
                / (ONE_HUNDRED_PERCENT - self.cfg.random_quote_chance);

            for q in quotes.iter() {
                let likes = if q.likes == 0 { 1.0 } else { q.likes as f64 };
                let percent = likes / del * ONE_HUNDRED_PERCENT;
                if percent + accumulator >= random_percent {
                    return Ok(q.clone());
                }

                accumulator += percent
            }
        }

        self.api.get_random_quote().await
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use fake::{
        faker::{lorem, name},
        uuid, Fake, Faker,
    };
    use mockall::predicate::*;
    use std::sync::LazyLock;

    use super::*;
    use crate::database::structs::quotes::Model as quote_model;
    use crate::database::structs::views::Model as view_model;
    use crate::quote::traits::{MockApi, MockDatabase};

    static USER_ID: LazyLock<String> = LazyLock::new(|| uuid::UUIDv4.fake());
    static QUOTE_ID: LazyLock<String> = LazyLock::new(|| uuid::UUIDv4.fake());
    static QUOTE: LazyLock<quote_model> = LazyLock::new(|| quote_model {
        id: QUOTE_ID.clone(),
        quote: lorem::en::Sentence(5..10).fake(),
        author: name::en::Name().fake(),
        likes: Faker.fake(),
        tags: Faker.fake(),
    });
    static VIEW: LazyLock<view_model> = LazyLock::new(|| view_model {
        user_id: USER_ID.clone(),
        quote_id: QUOTE_ID.clone(),
        liked: true,
    });

    #[tokio::test]
    async fn test_get_quote_success() {
        let mut db = MockDatabase::new();

        db.expect_get_quotes()
            .with(eq(USER_ID.clone()))
            .returning(|_| Ok(vec![QUOTE.clone()]));

        db.expect_mark_as_viewed()
            .with(eq(USER_ID.clone()), eq(QUOTE_ID.clone()))
            .returning(|_, _| Ok(()));

        let service = new_service(QuotesConfig::default(), (db, MockApi::new()));
        let res = service.get_quote(&USER_ID).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), from_database_quote_to_quote(QUOTE.clone()));
    }

    #[tokio::test]
    async fn test_get_quote_success_random() {
        let mut db = MockDatabase::new();

        db.expect_get_quotes()
            .with(eq(USER_ID.clone()))
            .returning(|_| Ok(vec![QUOTE.clone()]));

        db.expect_mark_as_viewed()
            .with(eq(USER_ID.clone()), eq(QUOTE_ID.clone()))
            .returning(|_, _| Ok(()));

        let mut api = MockApi::new();

        api.expect_get_random_quote()
            .returning(|| Ok(QUOTE.clone()));

        let service = new_service(
            QuotesConfig {
                random_quote_chance: 100.0,
            },
            (db, api),
        );

        let res = service.get_quote(&USER_ID).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), from_database_quote_to_quote(QUOTE.clone()));
    }

    #[tokio::test]
    async fn test_like_quote_success() {
        let mut db = MockDatabase::new();

        db.expect_get_view()
            .with(eq(USER_ID.clone()), eq(QUOTE_ID.clone()))
            .returning(|_, _| Ok(VIEW.clone()));

        db.expect_like_quote()
            .with(eq(QUOTE_ID.clone()))
            .returning(|_| Ok(()));

        db.expect_mark_as_liked()
            .with(eq(USER_ID.clone()), eq(QUOTE_ID.clone()))
            .returning(|_, _| Ok(()));

        let service = new_service(QuotesConfig::default(), (db, MockApi::new()));
        let res = service.like_quote(&USER_ID, &QUOTE_ID).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_like_quote_already_liked() {
        let mut db = MockDatabase::new();

        db.expect_get_view()
            .with(eq(USER_ID.clone()), eq(QUOTE_ID.clone()))
            .returning(|_, _| Ok(VIEW.clone()));

        let service = new_service(QuotesConfig::default(), (db, MockApi::new()));
        let res = service.like_quote(&USER_ID, &QUOTE_ID).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_same_quote_success() {
        let mut db = MockDatabase::new();

        db.expect_get_quote()
            .with(eq(QUOTE_ID.clone()))
            .returning(|_| Ok(QUOTE.clone()));

        db.expect_get_same_quote()
            .with(eq(USER_ID.clone()), eq(QUOTE.clone()))
            .returning(|_, _| Ok(QUOTE.clone()));

        db.expect_mark_as_viewed()
            .with(eq(USER_ID.clone()), eq(QUOTE_ID.clone()))
            .returning(|_, _| Ok(()));

        let service = new_service(QuotesConfig::default(), (db, MockApi::new()));
        let res = service.get_same_quote(&USER_ID, &QUOTE_ID).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), from_database_quote_to_quote(QUOTE.clone()));
    }

    #[tokio::test]
    async fn test_get_same_quote_random() {
        let mut db = MockDatabase::new();

        db.expect_get_quote()
            .with(eq(QUOTE_ID.clone()))
            .returning(|_| Ok(QUOTE.clone()));

        db.expect_get_same_quote()
            .with(eq(USER_ID.clone()), eq(QUOTE.clone()))
            .returning(|_, _| Err(anyhow!(DatabaseErrors::ErrNotFound)));

        db.expect_mark_as_viewed()
            .with(eq(USER_ID.clone()), eq(QUOTE_ID.clone()))
            .returning(|_, _| Ok(()));

        let mut api = MockApi::new();

        api.expect_get_random_quote()
            .returning(|| Ok(QUOTE.clone()));

        let service = new_service(QuotesConfig::default(), (db, api));
        let res = service.get_same_quote(&USER_ID, &QUOTE_ID).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), from_database_quote_to_quote(QUOTE.clone()));
    }

    fn new_service(cfg: QuotesConfig, mocks: (MockDatabase, MockApi)) -> Service {
        Service::new(&cfg, Arc::new(mocks.0), Arc::new(mocks.1))
    }
}
