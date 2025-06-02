use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DbBackend, Set};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use sea_orm::{Database, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use anyhow::Result;
use crate::database::{Database as TDatabase, GetMsgs, Msg, ReceiveMsg};
use crate::database::GetMsgs::{After, Before};
use crate::entities::msg;
use msg::Entity as Messages;

#[derive(Clone)]
pub struct Sqlite {
    db: Arc<DatabaseConnection>,
}

impl Sqlite {
    pub async fn new(filename: String) -> Result<Self> {
        let abs = std::env::current_dir()?.join(filename);
        let abs_str = abs.to_str().ok_or_else(|| anyhow::anyhow!("non-utf8 path"))?;
        
        let db = 
            Database::connect(format!("sqlite://{}", abs_str))
                .await?;
        db.execute(
            sea_orm::Statement::from_string(
                DbBackend::Sqlite,
                "CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY AUTOINCREMENT,
                    author TEXT NOT NULL,
                    content TEXT NOT NULL,
                    timestamp INTEGER NOT NULL);".to_string()
            )
        ).await?;
        Ok( Self { db: Arc::new(db) } )
    }
}

impl From<&msg::Model> for Msg {
    fn from(msg: &msg::Model) -> Self {
        Self {
            id: msg.id,
            author: Arc::from(msg.author.as_str()),
            content: Arc::from(msg.content.as_str()),
            timestamp: msg.timestamp.unsigned_abs()
        }
    }   
}

#[async_trait::async_trait]
impl TDatabase for Sqlite {
    async fn get_msgs(&self, count: GetMsgs, limit: u32) -> Result<Vec<Arc<Msg>>> {
        let db = self.db.clone();
        tracing::info!("get_msgs: {:?}, limit: {}", count, limit);
        match count {
            After(after) => {
                if after == 0 {
                    Ok(
                        Messages::find()
                            .order_by_desc(msg::Column::Id)
                            .limit(limit as u64)
                            .all(db.as_ref())
                            .await?
                            .iter()
                            .map(|msg| msg.into())
                            .map(Arc::new)
                            .collect()
                    )
                } else {
                    Ok(
                        Messages::find()
                            .filter(msg::Column::Id.gt(after as u32))
                            .order_by_desc(msg::Column::Id)
                            .limit(Some(limit as u64))
                            .all(db.as_ref())
                            .await?
                            .iter()
                            .map(|msg| msg.into())
                            .map(Arc::new)
                            .collect()
                    )
                }
            },
            Before(before) => Ok(
                Messages::find()
                    .filter(msg::Column::Id.lt(before as u32))
                    .order_by_desc(msg::Column::Id)
                    .limit(Some(limit as u64))
                    .all(db.as_ref())
                    .await?
                    .iter()
                    .map(|msg| msg.into())
                    .map(Arc::new)
                    .collect()
            )
        }
    }

    async fn send_msg(&self, msg: ReceiveMsg) -> Result<()> {
        let db = self.db.clone();
        msg::ActiveModel {
            author: Set(msg.author.to_string()),
            content: Set(msg.content.to_string()),
            timestamp: Set(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64),
            ..Default::default() }
            .insert(db.as_ref())
            .await?;
        Ok(())
    }

    async fn last_msg(&self) -> Result<u32> {
        let db = self.db.clone();
        Ok(
            Messages::find()
                .order_by_desc(msg::Column::Id)
                .one(db.as_ref())
                .await?
                .map(|msg| msg.id)
                .unwrap_or(
                    0
                )
        )
    }
}