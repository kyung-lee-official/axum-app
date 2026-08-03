use super::app_kind::AppKind;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// In-app role definition (runtime CRUD). `name` is an arbitrary string per app
/// (e.g. internal: "admin", "manager").
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub app: AppKind,
    pub name: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_app_role::Entity")]
    UserAppRoles,
}

impl Related<super::user_app_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAppRoles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
