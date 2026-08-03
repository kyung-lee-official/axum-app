//! SeaORM entity models. Tables must be created via migrations you run manually.
//!
//! Identity model:
//! - `users` — one account (email)
//! - `user_apps` — which apps they may open (`shop`, `internal`, …)
//! - `roles` — in-app roles (runtime names, e.g. internal `admin` / `manager`)
//! - `user_app_roles` — role assignments
//! - `sessions` — server-side session rows

#![allow(dead_code)] // used once auth handlers land

pub mod app_kind;
pub mod prelude;
pub mod role;
pub mod session;
pub mod user;
pub mod user_app;
pub mod user_app_role;

pub use app_kind::AppKind;
pub use role::Entity as Role;
pub use session::Entity as Session;
pub use user::Entity as User;
pub use user_app::Entity as UserApp;
pub use user_app_role::Entity as UserAppRole;
