use crate::auth::cookie::{clear_session_cookie, set_session_cookie};
use crate::auth::password::verify_password;
use crate::auth::session::{create_session, delete_session};
use crate::entity::prelude::*;
use crate::entity::{user, user_app};
use crate::error::AppError;
use crate::extractors::RequireInternal;
use crate::routes::api_auth::AuthUserResponse;
use crate::state::AppState;
use axum::{extract::State, routing::get, routing::post, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use tower_cookies::Cookies;

#[derive(Debug, Deserialize)]
pub struct SignInBody {
    pub email: String,
    pub password: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/sign-in", post(sign_in))
        .route("/auth/sign-out", post(sign_out))
        .route("/auth/me", get(me))
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn apps_as_strings(apps: &[AppKind]) -> Vec<String> {
    apps.iter()
        .map(|a| match a {
            AppKind::Shop => "shop".to_string(),
            AppKind::Internal => "internal".to_string(),
        })
        .collect()
}

fn cookie_secure() -> bool {
    std::env::var("COOKIE_SECURE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

async fn sign_in(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(body): Json<SignInBody>,
) -> Result<Json<AuthUserResponse>, AppError> {
    let email = normalize_email(&body.email);
    if email.is_empty() || body.password.len() < 8 {
        return Err(AppError::BadRequest("invalid credentials".into()));
    }

    let user = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid email or password".into()))?;

    if user.status != UserStatus::Active {
        return Err(AppError::Forbidden("account disabled".into()));
    }

    if !verify_password(&body.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("invalid email or password".into()));
    }

    let apps_models = user_app::Entity::find()
        .filter(user_app::Column::UserId.eq(user.id))
        .all(&state.db)
        .await?;
    let apps: Vec<AppKind> = apps_models.into_iter().map(|a| a.app).collect();

    if !apps.contains(&AppKind::Internal) {
        return Err(AppError::Forbidden("internal access required".into()));
    }

    let session = create_session(&state.db, user.id, state.session_ttl).await?;
    set_session_cookie(
        &cookies.private(&state.cookie_key),
        &session.id.to_string(),
        state.session_ttl.num_seconds(),
        cookie_secure(),
    );

    Ok(Json(AuthUserResponse {
        id: user.id,
        email: user.email,
        apps: apps_as_strings(&apps),
    }))
}

async fn sign_out(
    State(state): State<AppState>,
    cookies: Cookies,
    user: RequireInternal,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_session(&state.db, user.0.session_id).await?;
    clear_session_cookie(
        &cookies.private(&state.cookie_key),
        cookie_secure(),
    );
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn me(user: RequireInternal) -> Json<AuthUserResponse> {
    let ctx = user.0;
    Json(AuthUserResponse {
        id: ctx.user.id,
        email: ctx.user.email.clone(),
        apps: apps_as_strings(&ctx.apps),
    })
}
