use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait Database {
    async fn ping(&self) -> Result<()>;
}

pub struct Heartbeat {
    db: Arc<Mutex<dyn Database + Send>>,
}

impl Heartbeat {
    pub async fn ping_database(&self) -> Result<()> {
        self.db.lock().await.ping().await
    }

    pub fn new(db: Arc<Mutex<dyn Database + Send>>) -> Self {
        Heartbeat { db }
    }
}
