mod auth;
mod entity;
mod error;
mod extractors;
mod routes;
mod state;

use crate::state::AppState;
use axum::http::{header, HeaderValue, Method};
use sea_orm::Database;
use std::net::SocketAddr;
use tokio::signal;
use tower_cookies::{CookieManagerLayer, Key};
use tower_http::cors::{AllowOrigin, CorsLayer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is required (see .env.example)")?;
    let db = Database::connect(&database_url).await?;
    tracing::info!("connected to database");

    let cookie_key = load_cookie_key()?;
    let session_ttl_hours: i64 = std::env::var("SESSION_TTL_HOURS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(168);
    let session_ttl = chrono::Duration::hours(session_ttl_hours);

    let state = AppState {
        db,
        cookie_key,
        session_ttl,
    };

    let app = routes::app_router()
        .layer(cors_layer()?)
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3002".into())
        .parse()?;
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn load_cookie_key() -> Result<Key, Box<dyn std::error::Error>> {
    if let Ok(secret) = std::env::var("COOKIE_KEY") {
        let bytes = secret.as_bytes();
        if bytes.len() < 64 {
            return Err("COOKIE_KEY must be at least 64 characters".into());
        }
        return Ok(Key::from(&bytes[..64]));
    }
    tracing::warn!(
        "COOKIE_KEY not set; using ephemeral key (sessions invalidate on restart)"
    );
    Ok(Key::generate())
}

fn cors_layer() -> Result<CorsLayer, Box<dyn std::error::Error>> {
    let origins_raw = std::env::var("CORS_ORIGINS").unwrap_or_else(|_| {
        "http://localhost:3000,http://127.0.0.1:3000,http://localhost:3002".into()
    });
    let origins: Vec<HeaderValue> = origins_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(HeaderValue::from_str)
        .collect::<Result<_, _>>()?;

    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION])
        .allow_credentials(true))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
