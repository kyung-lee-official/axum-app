pub mod api_auth;
pub mod internal_auth;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn api_router() -> Router<AppState> {
    Router::new().merge(api_auth::router())
}

pub fn internal_router() -> Router<AppState> {
    Router::new().merge(internal_auth::router())
}

pub async fn health() -> &'static str {
    "ok"
}

pub fn app_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .nest("/api", api_router())
        .nest("/api/internal", internal_router())
}
