use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use axum::extract::{Query, State, ConnectInfo};
use axum::{Json, Router};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;
use axum::http::{StatusCode, HeaderMap};
use tokio::sync::Mutex;
use crate::database::{Database, ReceiveMsg};
use crate::database::GetMsgs::{After, Before};

#[derive(Deserialize)]
struct Pagination {
    before: Option<usize>,
    after: Option<usize>,
    limit: usize
}

struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    max_requests: usize,
    window_seconds: u64,
}

impl RateLimiter {
    fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window_seconds,
        }
    }

    fn check_request_limit(&mut self, ip: &str) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(self.window_seconds);
        
        if let Some(timestamps) = self.requests.get_mut(ip) {
            timestamps.retain(|&time| now.duration_since(time) < window);
            
            if timestamps.len() >= self.max_requests {
                return true;
            }
            
            timestamps.push(now);
        } else {
            self.requests.insert(ip.to_string(), vec![now]);
        }

        false
    }
}

#[derive(Clone)]
struct AppState<T: Database> {
    db: Arc<T>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}


fn get_client_ip(headers: &HeaderMap, conn_info: Option<&ConnectInfo<std::net::SocketAddr>>) -> String {
    // there exists obvious abuse, when service is not behind proxy, one can send fake ip, but 
    // I will use proxy, so good luck with that 
    if let Some(forwarded_for) = headers.get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(client_ip) = forwarded_str.split(',').next() {
                return client_ip.trim().to_string();
            }
        }
    }
    
    if let Some(real_ip) = headers.get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    if let Some(conn_info) = conn_info {
        return conn_info.0.ip().to_string();
    }
    
    "unknown".to_string()
}

async fn send_msg<T: Database>(
    State(state): State<Arc<AppState<T>>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    Json(msg): Json<ReceiveMsg>
) -> Response {
    let client_ip = get_client_ip(&headers, Some(&ConnectInfo(addr)));
    tracing::info!("Request from IP: {}", client_ip);
    
    let is_limited = {
        let mut rate_limiter = state.rate_limiter.lock().await;
        rate_limiter.check_request_limit(&client_ip)
    };

    if is_limited {
        tracing::warn!("Rate limit exceeded for IP: {}", client_ip);
        return (
            StatusCode::TOO_MANY_REQUESTS,
            serde_json::json!({
                "error": "Превышен лимит сообщений. Попробуй через минутку (и прекрати спамить)",
            }).to_string()
        ).into_response();
    }

    let db = state.db.clone();
    tracing::info!("send_msg: {:?}", msg);
    if let Err(e) = msg.check_valid() {
        return (StatusCode::BAD_REQUEST, 
                serde_json::json!({"err": e.to_string()}).to_string()).into_response();
    }
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
        db: Arc::new(db),
        rate_limiter: Arc::new(Mutex::new(RateLimiter::new(2, 60))), 
    };
    Router::new()
        .route("/get_msgs", get(get_msgs))
        .route("/send_msg", post(send_msg))
        .route("/last_msg", get(last_msg))
        .with_state(Arc::new(state))
}
