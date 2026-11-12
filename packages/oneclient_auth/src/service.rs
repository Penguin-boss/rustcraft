use tokio::sync::Mutex;
use uuid::Uuid;

use crate::data::MinecraftAccount;
use crate::error::{AuthError, AuthResult};
use crate::store::CredentialsStore;

/// Placeholder access token used for every launch.
///
/// Offline-mode servers never validate it and vanilla only requires the
/// argument to be present, so the literal `0` is the conventional value.
pub const OFFLINE_ACCESS_TOKEN: &str = "0";

/// Owns the one local profile. Composition builds it once and passes it down;
/// nothing here reaches for a global or a database.
pub struct AuthService {
    store: Mutex<CredentialsStore>,
}

impl AuthService {
    /// A missing or unreadable `auth.json` yields an empty profile rather than
    /// failing, so a corrupt file means "pick a username" not "will not start".
    pub async fn load() -> AuthResult<Self> {
        Ok(Self::with_store(CredentialsStore::load().await?))
    }

    #[must_use]
    pub fn with_store(store: CredentialsStore) -> Self {
        Self {
            store: Mutex::new(store),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list_accounts(&self) -> Vec<MinecraftAccount> {
        self.store.lock().await.list_accounts()
    }

    #[tracing::instrument(level = "debug", skip(self), fields(%id))]
    pub async fn get_account(&self, id: Uuid) -> Option<MinecraftAccount> {
        self.store.lock().await.get_account(id)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn default_account(&self) -> AuthResult<Option<MinecraftAccount>> {
        Ok(self.store.lock().await.profile.clone())
    }

    /// Included for the account-switcher surface. With a single local profile
    /// `Some(id)` only validates that the profile still exists and `None`
    /// clears it.
    #[tracing::instrument(level = "debug", skip(self), fields(?id))]
    pub async fn set_default_account(&self, id: Option<Uuid>) -> AuthResult<()> {
        let mut store = self.store.lock().await;
        match id {
            Some(id) if store.profile.as_ref().is_some_and(|a| a.id == id) => Ok(()),
            Some(id) => Err(AuthError::AccountNotFound(id)),
            None => store.clear().await,
        }
    }

    #[tracing::instrument(level = "debug", skip(self), fields(%id))]
    pub async fn remove_account(&self, id: Uuid) -> AuthResult<()> {
        let mut store = self.store.lock().await;
        if store.profile.as_ref().is_some_and(|a| a.id == id) {
            store.clear().await?;
        }
        Ok(())
    }

    /// Validates and persists the offline username the launcher should play as.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn set_offline_username(
        &self,
        username: impl Into<String>,
    ) -> AuthResult<MinecraftAccount> {
        self.store.lock().await.set_username(username).await
    }

    /// Back-compat name for [`Self::set_offline_username`].
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn add_offline_account(
        &self,
        username: impl Into<String>,
    ) -> AuthResult<MinecraftAccount> {
        self.set_offline_username(username).await
    }

    #[tracing::instrument(level = "debug", skip(self), fields(%id))]
    pub async fn account_for_launch(&self, id: Uuid) -> AuthResult<MinecraftAccount> {
        self.get_account(id)
            .await
            .ok_or(AuthError::AccountNotFound(id))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn default_account_for_launch(&self) -> AuthResult<Option<MinecraftAccount>> {
        self.default_account().await
    }

    #[must_use]
    pub async fn has_account(&self) -> bool {
        self.store.lock().await.profile.is_some()
    }
}
