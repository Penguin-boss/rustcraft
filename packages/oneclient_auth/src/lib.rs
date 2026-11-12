//! Offline-only authentication for RustCraft.
//!
//! There is no Microsoft OAuth, no session token and no account syncing. The
//! crate stores a single local username in `auth.json` and derives the vanilla
//! offline UUID from it. Construct [`AuthService`] in the composition layer and
//! pass it down; nothing in here reaches for a global or a database.

mod data;
mod error;
mod offline;
mod service;
mod store;

pub use data::MinecraftAccount;
pub use error::{AuthError, AuthResult};
pub use offline::{offline_account, offline_uuid, validate_offline_username};
pub use service::{AuthService, OFFLINE_ACCESS_TOKEN};
pub use store::CredentialsStore;
