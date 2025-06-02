use std::sync::{RwLock, Arc};
use std::time;
use std::time::UNIX_EPOCH;
use anyhow::Result;
use crate::database::{Database, GetMsgs, Msg, ReceiveMsg};
use crate::database::GetMsgs::{Before, After};
use time::SystemTime;

#[derive(Clone)]
pub struct MockBase {
    base: Arc<RwLock<Vec<Arc<Msg>>>>,
}

impl MockBase {
    pub fn new() -> Self { 
        Self { base: Arc::new(RwLock::new(Vec::new())) } 
    }
}

#[async_trait::async_trait]
impl Database for MockBase{
    async fn get_msgs(&self, count: GetMsgs, limit: u32) -> Result<Vec<Arc<Msg>>> {
        let guard = self.base.read().unwrap();
        match count {
            After(after) => {
                if after == 0 {
                    Ok(guard.iter()
                        .rev()
                        .take(limit as usize)
                        .map(|x| x.clone())
                        .collect()
                    )
                } else {
                    Ok(guard.iter()
                        .skip(after)
                        .take(limit as usize)
                        .map(|x| x.clone())
                        .collect()
                    )
                }
            },
            Before(before) => {
                Ok(guard.iter()
                    .rev()
                    .skip(guard.len().saturating_sub(before) + 1)
                    .take(limit as usize)
                    .map(|x| x.clone())
                    .collect()
                )
            }
        }
    }

    async fn send_msg(&self, msg: ReceiveMsg) -> Result<()> {
        let mut guard = self.base.write().unwrap();
        let id = guard.len() as u32 + 1;
        if msg.author.len() > 20 {
            return Err(anyhow::anyhow!("Author name too long"));
        }
        if msg.content.len() > 250 {
            return Err(anyhow::anyhow!("Content too long"));
        }
        guard.push(Arc::new(Msg {
            id,
            author: msg.author,
            content: msg.content,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        }));
        Ok(())
    }

    async fn last_msg(&self) -> Result<u32> {
        let guard = self.base.read().unwrap();
        Ok(guard.len() as u32)
    }
}