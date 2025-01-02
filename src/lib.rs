mod config;
mod database;
mod heartbeat;
mod quote;
mod quote_api;
mod server;

use std::sync::Arc;

use config::GlobalConfig;
use database::seaorm::SeaORM;

pub async fn start(cfg: GlobalConfig) {
    let db = SeaORM::new(&cfg.orm_config)
        .await
        .expect("failed to start database");

    let db = Arc::new(db);
    let heartbeat = heartbeat::Heartbeat::new(db.clone());
    let quote_api = quote_api::Service::new(db.clone());
    let quote = quote::Service::new(&cfg.quotes_config, db, Arc::new(quote_api));

    if cfg.server_config.service_type.eq("actix") {
        server::start_actix(&cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to create server")
            .await
            .expect("failed to start server");
    } else if cfg.server_config.service_type.eq("rocket") {
        server::start_rocket(&cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to create server");
    } else if cfg.server_config.service_type.eq("axum") {
        let (listener, app) = server::start_axum(&cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to create server");

        axum::serve(listener, app)
            .await
            .expect("failed to start server");
    } else if cfg.server_config.service_type.eq("grpc") {
        server::start_grpc(&cfg.server_config, heartbeat, quote)
            .await
            .expect("failed to create server");
    }
}

pub mod test_tools {
    use anyhow::{Context, Result};
    use fake::{
        faker::{lorem, name},
        uuid, Fake, Faker,
    };
    use rand::seq::SliceRandom;

    use super::config::{GlobalConfig, ORMConfig, QuotesConfig, ServerConfig};
    use super::database::seaorm::SeaORM;
    use super::database::structs::quotes::Model as quote_model;
    use super::quote::structs as quote_structs;

    pub struct Tools {
        cfg: GlobalConfig,
        db: SeaORM,
        main_quote: quote_model,
    }

    impl Tools {
        pub fn get_config(&self) -> GlobalConfig {
            self.cfg.clone()
        }

        pub fn get_main_quote(&self) -> quote_model {
            self.main_quote.clone()
        }

        pub fn get_random_quote(&self) -> quote_model {
            get_random_quote()
        }

        pub fn get_same_quote(&self) -> quote_model {
            quote_model {
                author: self.main_quote.author.clone(),
                likes: 0i32,
                tags: self.main_quote.tags.clone(),
                ..get_random_quote()
            }
        }

        pub async fn get_quote(&self, id: &str) -> Result<quote_model> {
            self.db.get_quote(id).await
        }

        pub async fn save_quote(&self, quote: quote_model) -> Result<()> {
            self.db.save_quote(quote.into()).await
        }

        pub fn compare_quotes(&self, received_quote: &str, expected_quote: quote_model) {
            let received_quote: quote_structs::Quote =
                serde_json::from_str(received_quote).expect("failed to parse quote");

            assert_eq!(
                received_quote,
                quote_structs::from_database_quote_to_quote(expected_quote)
            );
        }

        pub async fn new(connection_string: String) -> Result<Self> {
            let cfg = GlobalConfig {
                server_config: ServerConfig {
                    addr: "0.0.0.0:1141".to_string(),
                    service_type: ["actix", "rocket"]
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .to_string(),
                },
                orm_config: ORMConfig {
                    dsn: connection_string,
                },
                quotes_config: QuotesConfig {
                    random_quote_chance: 0.0,
                },
            };

            let db = SeaORM::new(&cfg.orm_config)
                .await
                .context("failed to init database")?;

            Ok(Tools {
                cfg,
                db,
                main_quote: get_random_quote(),
            })
        }
    }

    fn get_random_quote() -> quote_model {
        quote_model {
            id: uuid::UUIDv4.fake(),
            quote: lorem::en::Sentence(5..10).fake(),
            author: name::en::Name().fake(),
            likes: 0i32,
            tags: Faker.fake(),
        }
    }
}
