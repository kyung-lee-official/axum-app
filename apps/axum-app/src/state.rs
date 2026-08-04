use sea_orm::DatabaseConnection;
use tower_cookies::Key;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    /// Master key for signed cookies (tower-cookies).
    pub cookie_key: Key,
    /// Session lifetime used when creating sessions.
    pub session_ttl: chrono::Duration,
}
