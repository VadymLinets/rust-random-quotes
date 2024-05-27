use anyhow::{Context, Result};
use config::{Config, File};
use serde_derive::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalConfig {
    pub server_config: ServerConfig,
    pub orm_config: ORMConfig,
    pub quotes_config: QuotesConfig,
}

impl GlobalConfig {
    pub fn get() -> Result<Self> {
        let s = Config::builder()
            .add_source(File::with_name("config").required(true))
            .build()
            .context("Failed to build context")?;

        s.try_deserialize().context("Failed to deserialize")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub addr: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ORMConfig {
    pub dsn: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuotesConfig {
    pub random_quote_chance: f64,
}
