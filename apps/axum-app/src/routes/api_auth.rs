use crate::auth::cookie::{clear_session_cookie, set_session_cookie};
use crate::auth::password::{hash_password, verify_password};
use crate::auth::session::{create_session, delete_session};
use crate::entity::prelude::*;
use crate::entity::{user, user_app};
use crate::error::AppError;
use crate::extractors::RequireShop;
use crate::state::AppState;
use axum::{extract::State, routing::get, routing::post, Json, Router};
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, ActiveModelTrait};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignInBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthUserResponse {
    pub id: Uuid,
    pub email: String,
    pub apps: Vec<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/sign-in", post(sign_in))
        .route("/auth/sign-out", post(sign_out))
        .route("/auth/me", get(me))
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_credentials(email: &str, password: &str) -> Result<(), AppError> {
    if email.is_empty() || !email.contains('@') {
        return Err(AppError::BadRequest("invalid email".into()));
    }
    if password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
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

async fn register(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(body): Json<RegisterBody>,
) -> Result<Json<AuthUserResponse>, AppError> {
    let email = normalize_email(&body.email);
    validate_credentials(&email, &body.password)?;

    let existing = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&state.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::Conflict("email already registered".into()));
    }

    let now = Utc::now().fixed_offset();
    let user_id = Uuid::new_v4();
    let password_hash = hash_password(&body.password)?;

    user::ActiveModel {
        id: Set(user_id),
        email: Set(email.clone()),
        password_hash: Set(password_hash),
        status: Set(UserStatus::Active),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await?;

    user_app::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        app: Set(AppKind::Shop),
        created_at: Set(now),
    }
    .insert(&state.db)
    .await?;

    let session = create_session(&state.db, user_id, state.session_ttl).await?;
    set_session_cookie(
        &cookies.private(&state.cookie_key),
        &session.id.to_string(),
        state.session_ttl.num_seconds(),
        cookie_secure(),
    );

    Ok(Json(AuthUserResponse {
        id: user_id,
        email,
        apps: vec!["shop".into()],
    }))
}

async fn sign_in(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(body): Json<SignInBody>,
) -> Result<Json<AuthUserResponse>, AppError> {
    let email = normalize_email(&body.email);
    validate_credentials(&email, &body.password)?;

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

    if !apps.contains(&AppKind::Shop) {
        return Err(AppError::Forbidden("shop access required".into()));
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
    user: RequireShop,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_session(&state.db, user.0.session_id).await?;
    clear_session_cookie(
        &cookies.private(&state.cookie_key),
        cookie_secure(),
    );
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn me(user: RequireShop) -> Json<AuthUserResponse> {
    let ctx = user.0;
    Json(AuthUserResponse {
        id: ctx.user.id,
        email: ctx.user.email.clone(),
        apps: apps_as_strings(&ctx.apps),
    })
}
