use std::sync::Arc;
use axum::extract::{Query, State};
use axum::{Json, Router};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;
use axum::http::StatusCode;
use crate::database::{Database, ReceiveMsg};
use crate::database::GetMsgs::{After, Before};

#[derive(Deserialize)]
struct Pagination {
    before: Option<usize>,
    after: Option<usize>,
    limit: usize
}

#[derive(Clone)]
struct AppState<T: Database> {
    db: Arc<T>,
}


async fn send_msg<T: Database>(
    State(state): State<Arc<AppState<T>>>,
    Json(msg): Json<ReceiveMsg>
) -> Response {
    let db = state.db.clone();
    tracing::info!("send_msg: {:?}", msg);
    match db.send_msg(msg).await {
        Ok(()) => (StatusCode::OK, 
                   serde_json::json!({"msg": "ok"}).to_string())
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, 
                   serde_json::json!({"err": e.to_string()}).to_string())
            .into_response()
    }
}

async fn get_msgs<T: Database>(
    State(state): State<Arc<AppState<T>>>,
    query: Query<Pagination>
) -> Response {
    let db = state.db.clone();
    if let Some(before) = query.before {
        if let Some(_) = query.after {
            tracing::error!("before and after can't be set at the same time");
            (StatusCode::BAD_REQUEST, "before and after can't be set at the same time")
                    .into_response()
        } else {
            if let Ok(msgs) = db.get_msgs(Before(before), query.limit as u32).await {
                tracing::info!("get_msgs: {:?}", msgs);
                (StatusCode::OK, Json(msgs)).into_response()
            } else {
                tracing::error!("get_msgs error");
                (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response()
            }
        }
    } else {
        if let Some(after) = query.after {
            if let Ok(msgs) = db.get_msgs(After(after), query.limit as u32).await {
                tracing::info!("get_msgs: {:?}", msgs);
                (StatusCode::OK, Json(msgs)).into_response()
            } else {
                tracing::error!("get_msgs error");
                (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response()
            }
        } else {
            tracing::error!("before or after must be set");
            (StatusCode::BAD_REQUEST, "before or after must be set").into_response()
        }
    }
}

async fn last_msg<T: Database>(
    State(state): State<Arc<AppState<T>>>,
) -> Response {
    let db = state.db.clone();
    let last_id = db.last_msg().await.unwrap_or(0);
    tracing::info!("last_id: {}", last_id);
    serde_json::json!({"id": last_id}).to_string().into_response()
}


pub fn msgs<T: Database>(db: T) -> Router {
    let state = AppState {
        db: Arc::new(db)
    };
    Router::new()
        .route("/get_msgs", get(get_msgs))
        .route("/send_msg", post(send_msg))
        .route("/last_msg", get(last_msg))
        .with_state(Arc::new(state))

}