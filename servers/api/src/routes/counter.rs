//! Counter REST API routes.

use crate::state::AppState;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use feature_counter::CounterService;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/counter/increment", post(increment))
        .route("/counter/decrement", post(decrement))
        .route("/counter/reset", post(reset))
        .route("/counter/value", get(get_value))
}

fn get_db(state: &AppState) -> Result<storage_libsql::EmbeddedLibSql, Json<serde_json::Value>> {
    state
        .embedded_db
        .clone()
        .ok_or_else(|| Json(serde_json::json!({ "error": "Embedded database not initialized" })))
}

async fn increment(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let service = usecases::counter_service::LibSqlCounterService::new(db);
    match service.increment().await {
        Ok(value) => Json(serde_json::json!({ "value": value })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn decrement(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let service = usecases::counter_service::LibSqlCounterService::new(db);
    match service.decrement().await {
        Ok(value) => Json(serde_json::json!({ "value": value })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn reset(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let service = usecases::counter_service::LibSqlCounterService::new(db);
    match service.reset().await {
        Ok(value) => Json(serde_json::json!({ "value": value })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn get_value(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = match get_db(&state) {
        Ok(db) => db,
        Err(e) => return e,
    };
    let service = usecases::counter_service::LibSqlCounterService::new(db);
    match service.get_value().await {
        Ok(value) => Json(serde_json::json!({ "value": value })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
