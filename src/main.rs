mod args;
mod database;
mod routers;

use axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = args::parse_args()?;
    tracing_subscriber::fmt::init();
    
    let db = database::mock::MockBase::new();
    
    let app = 
        Router::new()
            .merge(routers::static_files::static_paths())
            .merge(routers::msgs::msgs(db));

    let listener  = tokio::net::TcpListener::bind(("0.0.0.0", args.port)).await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    
    Ok(())
}
