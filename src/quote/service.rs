use anyhow::{Context, Result};
use async_trait::async_trait;
use rand::Rng;
use std::sync::Arc;

#[cfg(test)]
use mockall::automock;

use crate::config::cfg::QuotesConfig;
use crate::database::seaorm::Errors;
use crate::database::structs::quotes::Model as Quotes;
use crate::database::structs::views::Model as Views;

use super::structs::{self, from_database_quote_to_quote};

const ONE_HUNDRED_PERCENT: f64 = 100.0;

#[cfg_attr(test, automock)]
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

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Api {
    async fn get_random_quote(&self) -> Result<Quotes>;
}

pub struct Service {
    cfg: QuotesConfig,
    db: Arc<dyn Database + Send + Sync>,
    api: Arc<dyn Api + Send + Sync>,
}

impl Service {
    pub async fn get_quote(&self, user_id: String) -> Result<structs::Quote> {
        let quotes = self
            .db
            .get_quotes(user_id.clone())
            .await
            .context("failed to get quotes")?;

        let quote = self
            .randomize_quote(quotes)
            .await
            .context("failed to get random quote")?;

        self.db
            .mark_as_viewed(user_id, quote.clone().id)
            .await
            .context("failed to mark as viewed")?;

        Ok(from_database_quote_to_quote(quote))
    }

    pub async fn like_quote(&self, user_id: String, quote_id: String) -> Result<()> {
        let view = self
            .db
            .get_view(user_id.clone(), quote_id.clone())
            .await
            .context("failed to get view")?;

        if view.liked {
            return Ok(());
        }

        self.db
            .like_quote(quote_id.clone())
            .await
            .context("failed to like quote")?;

        self.db
            .mark_as_liked(user_id, quote_id)
            .await
            .context("failed to mark as liked")?;

        Ok(())
    }

    pub async fn get_same_quote(
        &self,
        user_id: String,
        quote_id: String,
    ) -> Result<structs::Quote> {
        let quote = self
            .db
            .get_quote(quote_id.clone())
            .await
            .context("failed to get viewed quote")?;

        let quote = match self.db.get_same_quote(user_id.clone(), quote).await {
            Ok(quote) => quote,
            Err(err) => match err.downcast_ref::<Errors>() {
                Some(Errors::ErrNotFound) => self
                    .api
                    .get_random_quote()
                    .await
                    .context("failed to get random quote")?,
                None => return Err(err.context("failed to get same quote")),
            },
        };

        self.db
            .mark_as_viewed(user_id, quote_id)
            .await
            .context("failed to mark as viewed")?;

        Ok(from_database_quote_to_quote(quote))
    }

    pub fn new(
        cfg: QuotesConfig,
        db: Arc<dyn Database + Send + Sync>,
        api: Arc<dyn Api + Send + Sync>,
    ) -> Self {
        Service { cfg, db, api }
    }

    async fn randomize_quote(&self, quotes: Vec<Quotes>) -> Result<Quotes> {
        let random_percent = rand::thread_rng().gen_range(0.0..101.0);
        if (ONE_HUNDRED_PERCENT - self.cfg.random_quote_chance) > random_percent
            && !quotes.is_empty()
        {
            let mut likes_count: f64 = 0.0;
            for mut q in quotes.clone() {
                if q.likes == 0 {
                    q.likes += 1;
                }

                likes_count += q.likes as f64;
            }

            let mut accumulator: f64 = 0.0;
            let del = likes_count * ONE_HUNDRED_PERCENT
                / (ONE_HUNDRED_PERCENT - self.cfg.random_quote_chance);

            for (i, q) in quotes.iter().enumerate() {
                let mut likes = q.likes;
                if likes == 0 {
                    likes += 1;
                }

                let percent = likes as f64 / del * ONE_HUNDRED_PERCENT;
                if percent + accumulator >= random_percent {
                    return Ok(quotes[i].to_owned());
                }

                accumulator += percent
            }
        }

        self.api.get_random_quote().await
    }
}

#[cfg(test)]
mod tests {
    use crate::database::structs::quotes::Model as quote_model;
    use crate::database::structs::views::Model as view_model;

    use super::*;

    use enclose::enclose;
    use fake::{
        faker::{lorem, name},
        uuid, Fake, Faker,
    };
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_get_quote_success() {
        let user_id: String = uuid::UUIDv4.fake();
        let quote_id: String = uuid::UUIDv4.fake();
        let quote: quote_model = quote_model {
            id: quote_id.clone(),
            quote: lorem::en::Sentence(5..10).fake(),
            author: name::en::Name().fake(),
            likes: Faker.fake(),
            tags: Faker.fake(),
        };

        let mut db = MockDatabase::new();

        db.expect_get_quotes()
            .with(eq(user_id.clone()))
            .returning(enclose! { (quote) move |_| Ok(vec![quote.clone()])});

        db.expect_mark_as_viewed()
            .with(eq(user_id.clone()), eq(quote_id))
            .returning(|_, _| Ok(()));

        let service = new_service(None, (db, MockApi::new()));
        let res = service.get_quote(user_id).await;
        assert!(!res.is_err());
        assert_eq!(res.unwrap(), structs::from_database_quote_to_quote(quote));
    }

    #[tokio::test]
    async fn test_get_quote_success_random() {
        let user_id: String = uuid::UUIDv4.fake();
        let quote_id: String = uuid::UUIDv4.fake();
        let quote: quote_model = quote_model {
            id: quote_id.clone(),
            quote: lorem::en::Sentence(5..10).fake(),
            author: name::en::Name().fake(),
            likes: Faker.fake(),
            tags: Faker.fake(),
        };

        let mut db = MockDatabase::new();

        db.expect_get_quotes()
            .with(eq(user_id.clone()))
            .returning(enclose! { (quote) move |_| Ok(vec![quote.clone()])});

        db.expect_mark_as_viewed()
            .with(eq(user_id.clone()), eq(quote_id))
            .returning(|_, _| Ok(()));

        let mut api = MockApi::new();

        api.expect_get_random_quote()
            .returning(enclose! { (quote) move || Ok(quote.clone())});

        let service = new_service(
            Some(QuotesConfig {
                random_quote_chance: 100.0,
            }),
            (db, api),
        );

        let res = service.get_quote(user_id).await;
        assert!(!res.is_err());
        assert_eq!(res.unwrap(), structs::from_database_quote_to_quote(quote));
    }

    #[tokio::test]
    async fn test_like_quote_success() {
        let user_id: String = uuid::UUIDv4.fake();
        let quote_id: String = uuid::UUIDv4.fake();
        let view = view_model {
            user_id: user_id.clone(),
            quote_id: quote_id.clone(),
            liked: false,
        };

        let mut db = MockDatabase::new();

        db.expect_get_view()
            .with(eq(user_id.clone()), eq(quote_id.clone()))
            .returning(move |_, _| Ok(view.clone()));

        db.expect_like_quote()
            .with(eq(quote_id.clone()))
            .returning(|_| Ok(()));

        db.expect_mark_as_liked()
            .with(eq(user_id.clone()), eq(quote_id.clone()))
            .returning(|_, _| Ok(()));

        let service = new_service(None, (db, MockApi::new()));
        let res = service.like_quote(user_id, quote_id).await;
        assert!(!res.is_err());
    }

    #[tokio::test]
    async fn test_like_quote_already_liked() {
        let user_id: String = uuid::UUIDv4.fake();
        let quote_id: String = uuid::UUIDv4.fake();
        let view = view_model {
            user_id: user_id.clone(),
            quote_id: quote_id.clone(),
            liked: true,
        };

        let mut db = MockDatabase::new();

        db.expect_get_view()
            .with(eq(user_id.clone()), eq(quote_id.clone()))
            .returning(move |_, _| Ok(view.clone()));

        let service = new_service(None, (db, MockApi::new()));
        let res = service.like_quote(user_id, quote_id).await;
        assert!(!res.is_err());
    }

    #[tokio::test]
    async fn test_get_same_quote_success() {
        let user_id: String = uuid::UUIDv4.fake();
        let quote_id: String = uuid::UUIDv4.fake();
        let quote: quote_model = quote_model {
            id: quote_id.clone(),
            quote: lorem::en::Sentence(5..10).fake(),
            author: name::en::Name().fake(),
            likes: Faker.fake(),
            tags: Faker.fake(),
        };

        let mut db = MockDatabase::new();

        db.expect_get_quote()
            .with(eq(quote_id.clone()))
            .returning(enclose! { (quote) move |_| Ok(quote.clone())});

        db.expect_get_same_quote()
            .with(eq(user_id.clone()), eq(quote.clone()))
            .returning(enclose! { (quote) move |_,_| Ok(quote.clone())});

        db.expect_mark_as_viewed()
            .with(eq(user_id.clone()), eq(quote_id.clone()))
            .returning(|_, _| Ok(()));

        let service = new_service(None, (db, MockApi::new()));
        let res = service.get_same_quote(user_id, quote_id).await;
        assert!(!res.is_err());
        assert_eq!(res.unwrap(), structs::from_database_quote_to_quote(quote));
    }

    #[tokio::test]
    async fn test_get_same_quote_random() {
        let user_id: String = uuid::UUIDv4.fake();
        let quote_id: String = uuid::UUIDv4.fake();
        let quote: quote_model = quote_model {
            id: quote_id.clone(),
            quote: lorem::en::Sentence(5..10).fake(),
            author: name::en::Name().fake(),
            likes: Faker.fake(),
            tags: Faker.fake(),
        };

        let mut db = MockDatabase::new();

        db.expect_get_quote()
            .with(eq(quote_id.clone()))
            .returning(enclose! { (quote) move |_| Ok(quote.clone())});

        db.expect_get_same_quote()
            .with(eq(user_id.clone()), eq(quote.clone()))
            .returning(|_, _| Err(Errors::ErrNotFound.into()));

        db.expect_mark_as_viewed()
            .with(eq(user_id.clone()), eq(quote_id.clone()))
            .returning(|_, _| Ok(()));

        let mut api = MockApi::new();

        api.expect_get_random_quote()
            .returning(enclose! { (quote) move || Ok(quote.clone())});

        let service = new_service(None, (db, api));
        let res = service.get_same_quote(user_id, quote_id).await;
        assert!(!res.is_err());
        assert_eq!(res.unwrap(), structs::from_database_quote_to_quote(quote));
    }

    fn new_service(cfg: Option<QuotesConfig>, mocks: (MockDatabase, MockApi)) -> Service {
        let cfg = match cfg {
            Some(cfg) => cfg,
            None => QuotesConfig {
                random_quote_chance: 0.0,
            },
        };

        Service::new(cfg, Arc::new(mocks.0), Arc::new(mocks.1))
    }
}
