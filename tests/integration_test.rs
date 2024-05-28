use enclose::enclose;
use fake::{
    faker::{lorem, name},
    uuid, Fake, Faker,
};
use std::time::Duration;
use testcontainers::{runners::AsyncRunner, RunnableImage};
use testcontainers_modules::postgres;
use tokio::time::{sleep_until, Instant};

use quotes_rs::{config::cfg, database::seaorm};
use quotes_rs::{database::structs::quotes::Model as quote_model, quote};

#[tokio::test]
async fn test_integration() {
    let db_name = "test_quotes";
    let db_user = "postgres";
    let db_password = "postgres";

    let db_container = RunnableImage::from(
        postgres::Postgres::default()
            .with_db_name(db_name)
            .with_user(db_user)
            .with_password(db_password),
    )
    .with_tag("latest")
    .start()
    .await
    .expect("postgres is not available");

    let connection_string = format!(
        "postgres://{db_user}:{db_password}@{}:{}/{db_name}",
        db_container.get_host().await.unwrap(),
        db_container.get_host_port_ipv4(5432).await.unwrap()
    );

    let cfg = cfg::GlobalConfig {
        server_config: cfg::ServerConfig {
            addr: "0.0.0.0:1141".to_string(),
        },
        orm_config: cfg::ORMConfig {
            dsn: connection_string,
        },
        quotes_config: cfg::QuotesConfig {
            random_quote_chance: 0.0,
        },
    };

    let server = tokio::spawn(enclose! {(cfg) async move { quotes_rs::start(cfg).await }});
    sleep_until(Instant::now() + Duration::from_secs(2)).await;

    let mut client = reqwest::Client::new();
    let mut db = seaorm::SeaORM::new(cfg.orm_config)
        .await
        .expect("database creation failed");

    let user_id: String = uuid::UUIDv4.fake();
    let test_quote = quote_model {
        id: uuid::UUIDv4.fake(),
        quote: lorem::en::Sentence(5..10).fake(),
        author: name::en::Name().fake(),
        likes: 0,
        tags: Faker.fake(),
    };

    db.save_quote(test_quote.clone().into())
        .await
        .expect("failed to save quote");

    get_quote(
        cfg.server_config.clone(),
        &mut db,
        &mut client,
        user_id.clone(),
        test_quote.clone(),
    )
    .await;

    like_quote(
        cfg.server_config.clone(),
        &mut db,
        &mut client,
        user_id.clone(),
        test_quote.clone(),
    )
    .await;

    get_same_quote(
        cfg.server_config.clone(),
        &mut db,
        &mut client,
        user_id.clone(),
        test_quote.clone(),
    )
    .await;

    server.abort();
    db_container
        .stop()
        .await
        .expect("failed to stop db_container");
}

async fn get_quote(
    cfg: cfg::ServerConfig,
    db: &mut seaorm::SeaORM,
    client: &mut reqwest::Client,
    user_id: String,
    quote: quote_model,
) {
    let resp = client
        .get("http://".to_owned() + cfg.addr.as_str() + "/")
        .query(&[("user_id", user_id)])
        .send()
        .await
        .expect("failed to receive random quote from site");
    assert_eq!(resp.status(), 200);

    let body = resp
        .text()
        .await
        .expect("failed to receive quote from server");
    let received_quote: quote::structs::Quote =
        serde_json::from_str(body.as_str()).expect("failed to parse quote");
    assert_eq!(
        received_quote,
        quote::structs::from_database_quote_to_quote(quote.clone())
    );

    let database_quote = db
        .get_quote(quote.id.clone())
        .await
        .expect("failed to get quote from database");
    assert_eq!(database_quote, quote);
}

async fn like_quote(
    cfg: cfg::ServerConfig,
    db: &mut seaorm::SeaORM,
    client: &mut reqwest::Client,
    user_id: String,
    quote: quote_model,
) {
    let resp = client
        .patch("http://".to_owned() + cfg.addr.as_str() + "/like")
        .query(&[("user_id", user_id), ("quote_id", quote.id.clone())])
        .send()
        .await
        .expect("failed to receive random quote from server");
    assert_eq!(resp.status(), 200);

    let database_quote = db
        .get_quote(quote.id.clone())
        .await
        .expect("failed to get quote from database");
    assert_eq!(database_quote.likes, 1);
}

async fn get_same_quote(
    cfg: cfg::ServerConfig,
    db: &mut seaorm::SeaORM,
    client: &mut reqwest::Client,
    user_id: String,
    quote: quote_model,
) {
    let same_quote = quote_model {
        id: uuid::UUIDv4.fake(),
        quote: lorem::en::Sentence(5..10).fake(),
        author: quote.author.clone(),
        likes: 0,
        tags: quote.tags.clone(),
    };

    db.save_quote(same_quote.clone().into())
        .await
        .expect("failed to save same quote");

    let random_quote = quote_model {
        id: uuid::UUIDv4.fake(),
        quote: lorem::en::Sentence(5..10).fake(),
        author: name::en::Name().fake(),
        likes: 0,
        tags: Faker.fake(),
    };

    db.save_quote(random_quote.clone().into())
        .await
        .expect("failed to save random quote");

    let resp = client
        .get("http://".to_owned() + cfg.addr.as_str() + "/same")
        .query(&[("user_id", user_id), ("quote_id", quote.id.clone())])
        .send()
        .await
        .expect("failed to receive random quote from server");
    assert_eq!(resp.status(), 200);

    let body = resp
        .text()
        .await
        .expect("failed to receive quote from server");
    let received_quote: quote::structs::Quote =
        serde_json::from_str(body.as_str()).expect("failed to parse quote");
    assert_eq!(
        received_quote,
        quote::structs::from_database_quote_to_quote(same_quote.clone())
    );
}
