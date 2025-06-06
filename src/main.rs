mod args;
mod database;
mod routers;
#[cfg(feature = "sqlite_db")]
mod entities;
mod integration;
mod utils;

use axum::Router;
use std::sync::Arc;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = args::parse_args()?;
    tracing_subscriber::fmt().init();

    tracing::info!("Args: {:#?}", args);
    tracing::info!("Current dir: {}", std::env::current_dir()?.display());

    #[cfg(not(feature = "sqlite_db"))]
    let db = database::mock::MockBase::new();

    #[cfg(feature = "sqlite_db")]
    let db =
        database::sqlite::Sqlite::new(args.filename)
            .await?;

    let telegram = Arc::new(integration::Telegram::new(
        args.tg_token,
        args.tg_chat_id,
    ));
    
    let integrations: Arc<[Arc<dyn integration::Integration>]> = 
        Arc::new([telegram]);

    let app =
        Router::new()
            .merge(routers::static_files::static_paths())
            .merge(routers::msgs::msgs(db, integrations))
            .merge(routers::git_info::git_info(args.repo_url));

    let listener  = tokio::net::TcpListener::bind(("0.0.0.0", args.port)).await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>()
    ).await?;

    Ok(())
}