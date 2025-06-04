#[cfg(not(feature = "sqlite_db"))]
pub mod mock;
#[cfg(feature = "sqlite_db")]
pub mod sqlite;

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Msg {
    id: u32,
    author: Arc<str>,
    content: Arc<str>,
    timestamp: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReceiveMsg {
    author: Arc<str>,
    content: Arc<str>,
}

impl ReceiveMsg {
    pub fn check_valid(&self) -> Result<()> {
        if self.author.is_empty() || self.content.is_empty() {
            return Err(anyhow::anyhow!("Invalid message"));
        }
        if self.author.chars().count() > 20 {
            return Err(anyhow::anyhow!("Author name too long"));
        }
        if self.content.chars().count() > 250 {
            return Err(anyhow::anyhow!("Content too long"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum GetMsgs {
    Before(usize),
    After(usize),
}

#[async_trait::async_trait]
pub trait Database: Clone + Send + Sync + 'static {
    async fn get_msgs(&self, count: GetMsgs, limit: u32) -> Result<Vec<Arc<Msg>>>;
    async fn send_msg(&self, msg: ReceiveMsg) -> Result<()>;
    async fn last_msg(&self) -> Result<u32>;
}