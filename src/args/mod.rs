pub struct Args {
    pub port: u16,
}

pub fn parse_args() -> anyhow::Result<Args> {
    Ok(Args {
        port: std::env::var("PORT")
            .unwrap_or("8080".to_string())
            .parse()?,
    })
}