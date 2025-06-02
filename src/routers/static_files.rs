use axum::Router;
use axum::routing::get_service;
use tower_http::services::ServeFile;

pub fn static_paths() -> Router {
    let app_js = get_service(ServeFile::new("./public/app.js"));
    let styles_css = get_service(ServeFile::new("./public/styles.css"));

    let index = get_service(ServeFile::new("./public/index.html"));
    let not_found = get_service(ServeFile::new("./public/404.html"));

    Router::new()
        .route("/", index)
        .route("/styles.css", styles_css)
        .route("/app.js", app_js)
        .fallback(not_found)
}