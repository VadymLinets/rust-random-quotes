use anyhow::{Context, Result};
use async_trait::async_trait;
use migration::{Migrator, MigratorTrait};
use sea_orm::ActiveValue::Set;
use sea_orm::{sea_query, QueryOrder};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect, QueryTrait,
};
use thiserror::Error;

use crate::config::cfg::ORMConfig;
use crate::{heartbeat, quote::service as quote_service, quoteapi::service as quoteapi_service};

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

    pub async fn get_quote(&self, quote_id: String) -> Result<quotes_model> {
        let quote = quotes::find_by_id(quote_id).one(&self.db).await?;
        match quote {
            Some(quote) => Ok(quote),
            None => Err(Errors::ErrNotFound.into()),
        }
    }

    async fn get_quotes(&self, user_id: String) -> Result<Vec<quotes_model>> {
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
        user_id: String,
        viewed_quote: quotes_model,
    ) -> Result<quotes_model> {
        let viewed = quotes::find()
            .select_only()
            .column(quotes_columns::Id)
            .inner_join(views)
            .filter(views_columns::UserId.eq(user_id));

        let tags = "cardinality(array(select unnest(quotes.tags) intersect select unnest(array['"
            .to_owned()
            + viewed_quote.tags.join("', '").as_str()
            + "'])))";

        let author = "(case when quotes.author = '".to_owned()
            + viewed_quote.author.as_str()
            + "' then 1 else 2 end)";

        let quote = quotes::find()
            .filter(quotes_columns::Id.not_in_subquery(viewed.as_query().to_owned()))
            .order_by_desc(migration::SimpleExpr::Custom(tags))
            .order_by_asc(migration::SimpleExpr::Custom(author))
            .order_by_desc(quotes_columns::Likes)
            .one(&self.db)
            .await?;

        match quote {
            Some(quote) => Ok(quote),
            None => Err(Errors::ErrNotFound.into()),
        }
    }

    async fn get_view(&self, user_id: String, quote_id: String) -> Result<views_model> {
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

    async fn mark_as_viewed(&self, user_id: String, quote_id: String) -> Result<()> {
        let view = views_active_model {
            user_id: Set(user_id),
            quote_id: Set(quote_id),
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

    async fn mark_as_liked(&self, user_id: String, quote_id: String) -> Result<()> {
        let view = views_active_model {
            user_id: Set(user_id),
            quote_id: Set(quote_id),
            liked: Set(true),
        };
        view.update(&self.db).await?;
        Ok(())
    }

    async fn like_quote(&self, quote_id: String) -> Result<()> {
        let quote = self.get_quote(quote_id).await?;
        let mut quote_active: quotes_active_model = quote.clone().into();
        quote_active.likes = Set(quote.likes + 1);
        quote_active.update(&self.db).await?;
        Ok(())
    }

    pub async fn new(cfg: ORMConfig) -> Result<Self> {
        let connection = Database::connect(cfg.dsn.as_str())
            .await
            .context("failed to connect to db")?;

        Migrator::up(&connection, None)
            .await
            .context("failed to migrate")?;

        Ok(SeaORM { db: connection })
    }
}

#[async_trait]
impl heartbeat::service::Database for SeaORM {
    async fn ping(&self) -> Result<()> {
        self.ping().await
    }
}

#[async_trait]
impl quote_service::Database for SeaORM {
    async fn get_quote(&self, quote_id: String) -> Result<quotes_model> {
        self.get_quote(quote_id).await
    }

    async fn get_quotes(&self, user_id: String) -> Result<Vec<quotes_model>> {
        self.get_quotes(user_id).await
    }

    async fn get_same_quote(
        &self,
        user_id: String,
        viewed_quote: quotes_model,
    ) -> Result<quotes_model> {
        self.get_same_quote(user_id, viewed_quote).await
    }

    async fn get_view(&self, user_id: String, quote_id: String) -> Result<views_model> {
        self.get_view(user_id, quote_id).await
    }

    async fn mark_as_viewed(&self, user_id: String, quote_id: String) -> Result<()> {
        self.mark_as_viewed(user_id, quote_id).await
    }

    async fn mark_as_liked(&self, user_id: String, quote_id: String) -> Result<()> {
        self.mark_as_liked(user_id, quote_id).await
    }

    async fn like_quote(&self, quote_id: String) -> Result<()> {
        self.like_quote(quote_id).await
    }
}

#[async_trait]
impl quoteapi_service::Database for SeaORM {
    async fn save_quote(&self, quote: quotes_model) -> Result<()> {
        self.save_quote(quote.into()).await
    }
}
