use enclose::enclose;
use fake::{uuid, Fake};
use std::{env, time::Duration};
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::postgres;
use tokio::time::sleep;

use quotes_rs::test_tools::Tools;

#[tokio::test]
async fn test_integration() {
    let db_name = "test_quotes";
    let db_user = "postgres";
    let db_password = "postgres";

    let db_container = postgres::Postgres::default()
        .with_db_name(db_name)
        .with_user(db_user)
        .with_password(db_password)
        .with_tag("latest")
        .start()
        .await
        .expect("postgres is not started properly");

    let runs_in_container = env::var("RUNS_IN_CONTAINER")
        .ok()
        .map_or(false, |value| value.eq("true"));

    let connection_string = match runs_in_container {
        true => format!(
            "postgres://{db_user}:{db_password}@{}:5432/{db_name}",
            db_container.get_bridge_ip_address().await.unwrap()
        ),
        false => format!(
            "postgres://{db_user}:{db_password}@{}:{}/{db_name}",
            db_container.get_host().await.unwrap(),
            db_container.get_host_port_ipv4(5432).await.unwrap()
        ),
    };

    let client = reqwest::Client::new();
    let tools = Tools::new(connection_string)
        .await
        .expect("failed to create tools");
    let cfg = tools.get_config();

    let server = tokio::spawn(enclose! {(cfg) async move { quotes_rs::start(cfg).await }});
    sleep(Duration::from_secs(2)).await; // wait until the server is ready

    let user_id: String = uuid::UUIDv4.fake();

    tools
        .save_quote(tools.get_main_quote())
        .await
        .expect("failed to save main quote");

    get_quote(&cfg.server_config.addr, &tools, &client, &user_id).await;
    like_quote(&cfg.server_config.addr, &tools, &client, &user_id).await;
    get_same_quote(&cfg.server_config.addr, &tools, &client, &user_id).await;

    server.abort();
    db_container
        .stop()
        .await
        .expect("failed to stop db_container");
}

async fn get_quote(addr: &str, tools: &Tools, client: &reqwest::Client, user_id: &str) {
    let quote = tools.get_main_quote();

    let resp = client
        .get(format!("http://{}/", addr))
        .query(&[("user_id", user_id)])
        .send()
        .await
        .expect("failed to receive random quote from site");
    assert_eq!(resp.status(), 200);

    let body = resp
        .text()
        .await
        .expect("failed to receive quote from server");

    tools.compare_quotes(body.as_str(), quote.clone());

    let database_quote = tools
        .get_quote(&quote.id)
        .await
        .expect("failed to get quote from database");
    assert_eq!(database_quote, quote);
}

async fn like_quote(addr: &str, tools: &Tools, client: &reqwest::Client, user_id: &str) {
    let quote = tools.get_main_quote();

    let resp = client
        .patch(format!("http://{}/like", addr))
        .query(&[("user_id", user_id), ("quote_id", &quote.id)])
        .send()
        .await
        .expect("failed to receive random quote from server");
    assert_eq!(resp.status(), 200);

    let database_quote = tools
        .get_quote(&quote.id)
        .await
        .expect("failed to get quote from database");
    assert_eq!(database_quote.likes.unwrap_or_default(), 1);
}

async fn get_same_quote(addr: &str, tools: &Tools, client: &reqwest::Client, user_id: &str) {
    let quote = tools.get_main_quote();
    let same_quote = tools.get_same_quote();
    let random_quote = tools.get_random_quote();

    tools
        .save_quote(same_quote.clone())
        .await
        .expect("failed to save same quote");

    tools
        .save_quote(random_quote.clone())
        .await
        .expect("failed to save random quote");

    let resp = client
        .get(format!("http://{}/same", addr))
        .query(&[("user_id", user_id), ("quote_id", &quote.id)])
        .send()
        .await
        .expect("failed to receive random quote from server");
    assert_eq!(resp.status(), 200);

    let body = resp
        .text()
        .await
        .expect("failed to receive quote from server");

    tools.compare_quotes(body.as_str(), same_quote);
}
