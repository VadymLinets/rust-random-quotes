use anyhow::{Context, Result};
use envconfig::Envconfig;
use serde_derive::Deserialize;

#[derive(Envconfig, Debug, Clone, Deserialize, Default)]
pub struct GlobalConfig {
    #[envconfig(nested)]
    pub server_config: ServerConfig,

    #[envconfig(nested)]
    pub orm_config: ORMConfig,

    #[envconfig(nested)]
    pub quotes_config: QuotesConfig,
}

impl GlobalConfig {
    pub fn get() -> Result<Self> {
        Self::init_from_env().context("Failed to deserialize")
    }
}

#[derive(Envconfig, Debug, Clone, Deserialize, Default)]
pub struct ServerConfig {
    #[envconfig(from = "ADDR")]
    pub addr: String,

    #[envconfig(from = "SERVICE_TYPE")]
    pub service_type: String,
}

#[derive(Envconfig, Debug, Clone, Deserialize, Default)]
pub struct ORMConfig {
    #[envconfig(from = "DSN")]
    pub dsn: String,
}

#[derive(Envconfig, Debug, Clone, Deserialize, Default)]
pub struct QuotesConfig {
    #[envconfig(from = "RANDOM_QUOTE_CHANCE")]
    pub random_quote_chance: f64,
}
