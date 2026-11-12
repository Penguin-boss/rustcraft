use md5::{Digest, Md5};
use uuid::Uuid;

use super::data::MinecraftAccount;
use super::error::AuthError;
use crate::error::AuthResult;

/// Vanilla's offline UUID: MD5 of `OfflinePlayer:<username>` with the version
/// nibble forced to 3 and the variant bits to RFC 4122.
///
/// This is exactly what `java.util.UUID.nameUUIDFromBytes` produces, so
/// offline-mode servers derive the same UUID for the same name.
#[must_use]
pub fn offline_uuid(username: &str) -> Uuid {
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{username}").as_bytes());
    let digest = hasher.finalize();

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest);
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

pub fn validate_offline_username(username: &str) -> AuthResult<()> {
    let len = username.chars().count();
    if !(3..=16).contains(&len) {
        return Err(AuthError::InvalidOfflineUsername {
            reason: "username must be 3-16 characters".into(),
        });
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AuthError::InvalidOfflineUsername {
            reason: "username may only contain letters, digits, and underscores".into(),
        });
    }

    Ok(())
}

/// Builds the profile for a name after validating it.
#[tracing::instrument(level = "debug", skip_all)]
pub fn offline_account(username: impl Into<String>) -> AuthResult<MinecraftAccount> {
    let username = username.into();
    validate_offline_username(&username)?;

    tracing::info!(%username, "creating offline profile");
    Ok(MinecraftAccount::new(username))
}
