use crate::auth::cookie::SESSION_COOKIE;
use crate::auth::session::{load_auth_context, AuthContext};
use crate::entity::prelude::AppKind;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use tower_cookies::Cookies;
use uuid::Uuid;

fn session_id_from_cookies(cookies: &Cookies, key: &tower_cookies::Key) -> Option<Uuid> {
    cookies
        .private(key)
        .get(SESSION_COOKIE)
        .and_then(|c| Uuid::parse_str(c.value()).ok())
}

/// Requires a valid session cookie and active user.
pub struct RequireUser(pub AuthContext);

impl FromRequestParts<AppState> for RequireUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, &())
            .await
            .map_err(|_| AppError::Unauthorized("missing cookies".into()))?;

        let Some(session_id) = session_id_from_cookies(&cookies, &state.cookie_key) else {
            return Err(AppError::Unauthorized("not signed in".into()));
        };

        let Some(ctx) = load_auth_context(&state.db, session_id).await? else {
            return Err(AppError::Unauthorized("invalid or expired session".into()));
        };

        Ok(Self(ctx))
    }
}

/// Requires signed-in user with the `shop` app entitlement.
pub struct RequireShop(pub AuthContext);

impl FromRequestParts<AppState> for RequireShop {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let RequireUser(ctx) = RequireUser::from_request_parts(parts, state).await?;
        if !ctx.apps.contains(&AppKind::Shop) {
            return Err(AppError::Forbidden("shop access required".into()));
        }
        Ok(Self(ctx))
    }
}

/// Requires signed-in user with the `internal` app entitlement.
pub struct RequireInternal(pub AuthContext);

impl FromRequestParts<AppState> for RequireInternal {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let RequireUser(ctx) = RequireUser::from_request_parts(parts, state).await?;
        if !ctx.apps.contains(&AppKind::Internal) {
            return Err(AppError::Forbidden("internal access required".into()));
        }
        Ok(Self(ctx))
    }
}
