
#[derive(Debug)]
pub struct Args {
    pub port: u16,
    #[cfg(feature = "sqlite_db")]
    pub filename: String,
    pub repo_url: String,
    pub tg_token: String,
    pub tg_chat_id: String,
}

pub fn parse_args() -> anyhow::Result<Args> {
    Ok(Args {
        port: std::env::var("PORT")
            .unwrap_or("8080".to_string())
            .parse()?,
        #[cfg(feature = "sqlite_db")]
        filename: std::env::var("DB_FILENAME")
            .unwrap_or("db.sqlite".to_string()),
        repo_url: std::env::var("REPO_URL")
            .unwrap_or("https://github.com/miko089/wall".to_string()),
        tg_token: std::env::var("TG_TOKEN")?,
        tg_chat_id: std::env::var("TG_CHAT_ID")?,
    })
}

