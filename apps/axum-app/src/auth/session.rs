use crate::entity::prelude::*;
use crate::entity::{session, user, user_app};
use crate::error::AppError;
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

pub struct CreatedSession {
    pub id: Uuid,
}

pub async fn create_session(
    db: &DatabaseConnection,
    user_id: Uuid,
    ttl: Duration,
) -> Result<CreatedSession, AppError> {
    let now = Utc::now().fixed_offset();
    let expires_at = now + ttl;
    let id = Uuid::new_v4();

    session::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        expires_at: Set(expires_at),
        created_at: Set(now),
    }
    .insert(db)
    .await?;

    Ok(CreatedSession { id })
}

pub async fn delete_session(db: &DatabaseConnection, session_id: Uuid) -> Result<(), AppError> {
    session::Entity::delete_by_id(session_id).exec(db).await?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub session_id: Uuid,
    pub user: user::Model,
    pub apps: Vec<AppKind>,
}

pub async fn load_auth_context(
    db: &DatabaseConnection,
    session_id: Uuid,
) -> Result<Option<AuthContext>, AppError> {
    let Some(sess) = session::Entity::find_by_id(session_id).one(db).await? else {
        return Ok(None);
    };

    let now = Utc::now().fixed_offset();
    if sess.expires_at <= now {
        session::Entity::delete_by_id(session_id).exec(db).await?;
        return Ok(None);
    }

    let Some(user) = user::Entity::find_by_id(sess.user_id).one(db).await? else {
        return Ok(None);
    };

    if user.status != UserStatus::Active {
        return Ok(None);
    }

    let apps = user_app::Entity::find()
        .filter(user_app::Column::UserId.eq(user.id))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.app)
        .collect();

    Ok(Some(AuthContext {
        session_id,
        user,
        apps,
    }))
}
