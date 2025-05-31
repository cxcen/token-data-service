use crate::models::KLineInterval;
use crate::services::KLineService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct KLineQuery {
    interval: String,
    limit: Option<usize>,
}

pub async fn get_klines(
    Path(symbol): Path<String>,
    Query(query): Query<KLineQuery>,
    State(kline_service): State<Arc<KLineService>>,
) -> Response {
    let interval = match KLineInterval::from_str(&query.interval) {
        Some(interval) => interval,
        None => {
            return (StatusCode::BAD_REQUEST, "Invalid interval").into_response();
        }
    };

    let limit = query.limit.unwrap_or(100).min(1000);
    let klines = kline_service.get_klines(&symbol, interval, limit);

    Json(klines).into_response()
}

pub async fn health_check() -> Response {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
    .into_response()
} 