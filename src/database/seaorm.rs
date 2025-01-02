use anyhow::{Context, Result};
use async_trait::async_trait;
use migration::{Migrator, MigratorTrait};
use sea_orm::sea_query::SimpleExpr;
use sea_orm::ActiveValue::Set;
use sea_orm::{sea_query, QueryOrder};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect, QueryTrait,
};
use thiserror::Error;

use crate::config::ORMConfig;
use crate::{
    heartbeat as heartbeat_service, quote as quote_service, quote_api as quote_api_service,
};

use super::structs::prelude::Quotes as quotes;
use super::structs::prelude::Views as views;
use super::structs::quotes::ActiveModel as quotes_active_model;
use super::structs::quotes::Column as quotes_columns;
use super::structs::quotes::Model as quotes_model;
use super::structs::views::ActiveModel as views_active_model;
use super::structs::views::Column as views_columns;
use super::structs::views::Model as views_model;

#[derive(Error, Debug, PartialEq)]
pub enum Errors {
    #[error("Database record not found")]
    ErrNotFound,
}

pub struct SeaORM {
    db: DatabaseConnection,
}

impl SeaORM {
    async fn ping(&self) -> Result<()> {
        self.db.ping().await.context("failed to ping database")
    }

    pub async fn get_quote(&self, quote_id: &str) -> Result<quotes_model> {
        let quote = quotes::find_by_id(quote_id).one(&self.db).await?;
        match quote {
            Some(quote) => Ok(quote),
            None => Err(Errors::ErrNotFound.into()),
        }
    }

    async fn get_quotes(&self, user_id: &str) -> Result<Vec<quotes_model>> {
        let viewed = quotes::find()
            .select_only()
            .column(quotes_columns::Id)
            .inner_join(views)
            .filter(views_columns::UserId.eq(user_id));

        Ok(quotes::find()
            .filter(quotes_columns::Id.not_in_subquery(viewed.as_query().to_owned()))
            .order_by_desc(quotes_columns::Likes)
            .all(&self.db)
            .await?)
    }

    async fn get_same_quote(
        &self,
        user_id: &str,
        viewed_quote: &quotes_model,
    ) -> Result<quotes_model> {
        let viewed = quotes::find()
            .select_only()
            .column(quotes_columns::Id)
            .inner_join(views)
            .filter(views_columns::UserId.eq(user_id));

        let tags = format!(
            "cardinality(array(select unnest(quotes.tags) intersect select unnest(array['{}'])))",
            viewed_quote.tags.clone().join("', '")
        );

        let author = format!(
            "(case when quotes.author = '{}' then 1 else 2 end)",
            viewed_quote.author.clone()
        );

        let quote = quotes::find()
            .filter(quotes_columns::Id.not_in_subquery(viewed.as_query().to_owned()))
            .order_by_desc(SimpleExpr::Custom(tags))
            .order_by_asc(SimpleExpr::Custom(author))
            .order_by_desc(quotes_columns::Likes)
            .one(&self.db)
            .await?;

        match quote {
            Some(quote) => Ok(quote),
            None => Err(Errors::ErrNotFound.into()),
        }
    }

    async fn get_view(&self, user_id: &str, quote_id: &str) -> Result<views_model> {
        let view: Option<views_model> = views::find()
            .filter(views_columns::UserId.eq(user_id))
            .filter(views_columns::QuoteId.eq(quote_id))
            .one(&self.db)
            .await?;

        match view {
            Some(view) => Ok(view),
            None => Err(Errors::ErrNotFound.into()),
        }
    }

    pub async fn save_quote(&self, quote: quotes_active_model) -> Result<()> {
        quotes::insert(quote)
            .on_conflict(
                sea_query::OnConflict::column(quotes_columns::Id)
                    .update_columns(vec![
                        quotes_columns::Author,
                        quotes_columns::Quote,
                        quotes_columns::Tags,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await?;

        Ok(())
    }

    async fn mark_as_viewed(&self, user_id: &str, quote_id: &str) -> Result<()> {
        let view = views_active_model {
            user_id: Set(user_id.to_owned()),
            quote_id: Set(quote_id.to_owned()),
            liked: Set(false),
        };

        views::insert(view)
            .on_conflict(
                sea_query::OnConflict::columns(vec![views_columns::UserId, views_columns::QuoteId])
                    .update_column(views_columns::Liked)
                    .to_owned(),
            )
            .exec(&self.db)
            .await?;

        Ok(())
    }

    async fn mark_as_liked(&self, user_id: &str, quote_id: &str) -> Result<()> {
        let view = views_active_model {
            user_id: Set(user_id.to_owned()),
            quote_id: Set(quote_id.to_owned()),
            liked: Set(true),
        };

        view.update(&self.db).await?;
        Ok(())
    }

    async fn like_quote(&self, quote_id: &str) -> Result<()> {
        let quote = self
            .get_quote(quote_id)
            .await
            .context("failed to get old quote")?;

        let mut quote_active: quotes_active_model = quote.into();
        quote_active.likes = Set(quote_active.likes.take().unwrap_or_default() + 1);

        quote_active
            .update(&self.db)
            .await
            .context("failed to update quote")?;

        Ok(())
    }

    pub async fn new(cfg: &ORMConfig) -> Result<Self> {
        let connection = Database::connect(&cfg.dsn)
            .await
            .context("failed to connect to db")?;

        Migrator::up(&connection, None)
            .await
            .context("failed to migrate")?;

        Ok(SeaORM { db: connection })
    }
}

#[async_trait]
impl heartbeat_service::Database for SeaORM {
    async fn ping(&self) -> Result<()> {
        self.ping().await
    }
}

#[async_trait]
impl quote_service::Database for SeaORM {
    async fn get_quote(&self, quote_id: &str) -> Result<quotes_model> {
        self.get_quote(quote_id).await
    }

    async fn get_quotes(&self, user_id: &str) -> Result<Vec<quotes_model>> {
        self.get_quotes(user_id).await
    }

    async fn get_same_quote(
        &self,
        user_id: &str,
        viewed_quote: &quotes_model,
    ) -> Result<quotes_model> {
        self.get_same_quote(user_id, viewed_quote).await
    }

    async fn get_view(&self, user_id: &str, quote_id: &str) -> Result<views_model> {
        self.get_view(user_id, quote_id).await
    }

    async fn mark_as_viewed(&self, user_id: &str, quote_id: &str) -> Result<()> {
        self.mark_as_viewed(user_id, quote_id).await
    }

    async fn mark_as_liked(&self, user_id: &str, quote_id: &str) -> Result<()> {
        self.mark_as_liked(user_id, quote_id).await
    }

    async fn like_quote(&self, quote_id: &str) -> Result<()> {
        self.like_quote(quote_id).await
    }
}

#[async_trait]
impl quote_api_service::Database for SeaORM {
    async fn save_quote(&self, quote: quotes_model) -> Result<()> {
        self.save_quote(quote.into()).await
    }
}
