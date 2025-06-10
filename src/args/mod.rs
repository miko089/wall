#[derive(Debug)]
pub struct Args {
    pub port: u16,
    pub repo_url: String,

    #[cfg(feature = "sqlite_db")]
    pub filename: String,
}

pub fn parse_args() -> anyhow::Result<Args> {
    Ok(Args {
        port: std::env::var("PORT")
            .unwrap_or("8080".to_string())
            .parse()?,
        repo_url: std::env::var("REPO_URL")
            .unwrap_or("https://github.com/miko089/wall".to_string()),

        #[cfg(feature = "sqlite_db")]
        filename: std::env::var("DB_FILENAME")
            .unwrap_or("db.sqlite".to_string()),
    })
}

