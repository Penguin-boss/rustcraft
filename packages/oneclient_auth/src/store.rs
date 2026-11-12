use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use oneclient_common::paths;

use crate::data::MinecraftAccount;
use crate::error::AuthResult;
use crate::offline::{offline_account, validate_offline_username};

/// `auth.json` holds the single local profile RustCraft launches with.
///
/// The file is intentionally tiny:
///
/// ```json
/// { "profile": { "id": "b50ad385-829d-3141-a216-7e7d7539ba7f", "username": "Notch" } }
/// ```
///
/// Older OneClient builds wrote a `users` map plus a `default_user`; [`load`]
/// migrates those files in place so existing offline names survive the update.
///
/// [`load`]: CredentialsStore::load
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CredentialsStore {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<MinecraftAccount>,
}

/// Shape of the pre-offline `auth.json` as well as the current one. Keeping a
/// single deserializer lets the migration live in one place.
#[derive(Debug, Deserialize)]
struct RawStore {
    #[serde(default)]
    profile: Option<MinecraftAccount>,
    #[serde(default)]
    users: HashMap<String, LegacyAccount>,
    #[serde(default)]
    default_user: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyAccount {
    username: String,
    #[serde(default)]
    kind: Option<String>,
}

impl CredentialsStore {
    #[tracing::instrument(level = "debug")]
    pub async fn load() -> AuthResult<Self> {
        let path = paths::auth_file()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw: RawStore = match polyio::read_json(&path).await {
            Ok(raw) => raw,
            Err(err) => {
                tracing::warn!("failed to read auth file: {err}");
                return Ok(Self::default());
            }
        };

        let legacy = raw.profile.is_none() && !raw.users.is_empty();
        // Recompute the id from the name so a hand-edited file can never drift
        // away from the UUID offline servers derive for that name.
        let profile = raw
            .profile
            .as_ref()
            .map(|profile| offline_account(profile.username.as_str()))
            .transpose()?
            .or_else(|| migrate_legacy(&raw));

        let store = Self { profile };

        if legacy && store.profile.is_some() {
            match store.save().await {
                Ok(()) => tracing::info!("migrated auth.json to the offline profile format"),
                Err(err) => tracing::warn!("failed to rewrite migrated auth file: {err}"),
            }
        }

        Ok(store)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn save(&self) -> AuthResult<()> {
        let path = paths::auth_file()?;

        polyio::write_json_atomic(&path, self).await?;
        Ok(())
    }

    #[must_use]
    pub fn list_accounts(&self) -> Vec<MinecraftAccount> {
        self.profile.iter().cloned().collect()
    }

    #[must_use]
    pub fn get_account(&self, id: Uuid) -> Option<MinecraftAccount> {
        self.profile
            .as_ref()
            .filter(|account| account.id == id)
            .cloned()
    }

    /// Replaces the profile with a new offline username and persists it.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn set_username(
        &mut self,
        username: impl Into<String>,
    ) -> AuthResult<MinecraftAccount> {
        let username = username.into();
        validate_offline_username(&username)?;

        tracing::debug!(%username, "saving offline profile");
        let account = MinecraftAccount::new(username);
        self.profile = Some(account.clone());
        self.save().await?;
        Ok(account)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn clear(&mut self) -> AuthResult<()> {
        if self.profile.take().is_some() {
            self.save().await?;
        }
        Ok(())
    }
}

/// Picks a name out of a legacy multi-account file: the default account first,
/// then an explicitly offline one, then the first name alphabetically so the
/// choice is stable regardless of map ordering.
fn migrate_legacy(raw: &RawStore) -> Option<MinecraftAccount> {
    let chosen = raw
        .default_user
        .as_deref()
        .and_then(|id| raw.users.get(id))
        .or_else(|| {
            raw.users
                .values()
                .find(|account| account.kind.as_deref() == Some("offline"))
        })
        .or_else(|| {
            raw.users
                .values()
                .min_by(|a, b| a.username.cmp(&b.username))
        })?;

    offline_account(chosen.username.clone()).ok()
}
