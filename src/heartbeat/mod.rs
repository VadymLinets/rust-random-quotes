use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait Database {
    async fn ping(&self) -> Result<()>;
}

#[derive(Clone)]
pub struct Heartbeat {
    db: Arc<dyn Database + Send + Sync>,
}

impl Heartbeat {
    pub async fn ping_database(&self) -> Result<()> {
        self.db.ping().await
    }

    pub fn new(db: Arc<dyn Database + Send + Sync>) -> Self {
        Heartbeat { db }
    }
}
