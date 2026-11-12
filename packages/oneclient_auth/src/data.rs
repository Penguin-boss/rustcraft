use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The local profile RustCraft launches with.
///
/// There is no Microsoft account any more: `id` is always the deterministic
/// offline UUID derived from `username`, so the same name keeps the same
/// player data on every offline-mode server and in singleplayer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinecraftAccount {
    pub id: Uuid,
    pub username: String,
}

impl MinecraftAccount {
    #[must_use]
    pub fn new(username: impl Into<String>) -> Self {
        let username = username.into();
        Self {
            id: crate::offline::offline_uuid(&username),
            username,
        }
    }
}
