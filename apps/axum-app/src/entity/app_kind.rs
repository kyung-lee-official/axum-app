use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Product / app surface (Google-like: shop ≈ YouTube, internal ≈ admin console).
#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum AppKind {
    #[sea_orm(string_value = "shop")]
    Shop,
    #[sea_orm(string_value = "internal")]
    Internal,
}
