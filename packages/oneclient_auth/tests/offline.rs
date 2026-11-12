use std::sync::{Mutex, MutexGuard};

use oneclient_auth::{
    AuthService, CredentialsStore, OFFLINE_ACCESS_TOKEN, offline_account, offline_uuid,
    validate_offline_username,
};
use uuid::Uuid;

/// `paths::set_launcher_dir` only takes effect once per process, so every test
/// shares one directory and takes this lock for the length of the test.
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn isolated() -> MutexGuard<'static, ()> {
    let guard = TEST_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    oneclient_common::paths::set_launcher_dir(
        std::env::temp_dir().join("oneclient-auth-offline-tests"),
    );
    let _ = std::fs::remove_file(oneclient_common::paths::auth_file().unwrap());
    guard
}

#[test]
fn offline_uuid_matches_vanilla_algorithm() {
    let id = offline_uuid("Notch");

    assert_eq!(
        id,
        Uuid::parse_str("b50ad385-829d-3141-a216-7e7d7539ba7f").unwrap()
    );
    assert_eq!(id.get_version_num(), 3);
    assert_ne!(id, offline_uuid("Steve"));
    assert_eq!(id, offline_uuid("Notch"), "the UUID must be deterministic");
    assert_ne!(offline_uuid("Notch"), offline_uuid("notch"));
}

#[test]
fn usernames_are_validated_like_vanilla() {
    assert!(validate_offline_username("Steve").is_ok());
    assert!(validate_offline_username("A_1").is_ok());
    assert!(validate_offline_username("sixteen_chars_ab").is_ok());

    for bad in ["", "ab", "has space", "sébastien", "way_too_long_username"] {
        assert!(
            validate_offline_username(bad).is_err(),
            "{bad:?} must be rejected"
        );
    }
}

#[test]
fn offline_account_derives_the_uuid_from_the_name() {
    let account = offline_account("Notch").unwrap();

    assert_eq!(account.username, "Notch");
    assert_eq!(account.id, offline_uuid("Notch"));
}

#[tokio::test]
async fn a_profile_survives_a_save_and_reload() {
    let _guard = isolated();

    let mut store = CredentialsStore::default();
    assert!(store.profile.is_none());

    let saved = store.set_username("Steve").await.unwrap();
    assert_eq!(saved.id, offline_uuid("Steve"));

    let reloaded = CredentialsStore::load().await.unwrap();
    let profile = reloaded.profile.expect("the profile must persist");
    assert_eq!(profile.username, "Steve");
    assert_eq!(profile.id, offline_uuid("Steve"));
}

#[tokio::test]
async fn changing_the_name_replaces_the_profile_and_its_uuid() {
    let _guard = isolated();

    let mut store = CredentialsStore::default();
    store.set_username("Steve").await.unwrap();
    let new = store.set_username("Alex").await.unwrap();

    assert_eq!(new.username, "Alex");
    assert_eq!(store.list_accounts().len(), 1);
    assert_eq!(
        CredentialsStore::load().await.unwrap().profile.unwrap(),
        new
    );
}

#[tokio::test]
async fn invalid_names_are_rejected_without_touching_the_file() {
    let _guard = isolated();

    let mut store = CredentialsStore::default();
    let err = store.set_username("no spaces here").await.unwrap_err();
    assert!(
        err.to_string().contains("invalid offline username"),
        "{err}"
    );
    assert!(store.profile.is_none());
    assert!(CredentialsStore::load().await.unwrap().profile.is_none());
}

#[tokio::test]
async fn the_service_exposes_the_profile_for_launch() {
    let _guard = isolated();

    let service = AuthService::with_store(CredentialsStore::default());
    assert!(service.default_account().await.unwrap().is_none());
    assert!(!service.has_account().await);

    let account = service.set_offline_username("Steve").await.unwrap();
    assert_eq!(
        service.default_account_for_launch().await.unwrap(),
        Some(account.clone())
    );
    assert_eq!(
        service.account_for_launch(account.id).await.unwrap(),
        account
    );
    assert_eq!(service.list_accounts().await, vec![account.clone()]);
    assert_eq!(service.get_account(Uuid::nil()).await, None);
    assert!(service.account_for_launch(Uuid::nil()).await.is_err());
}

#[tokio::test]
async fn removing_the_profile_signs_the_launcher_out() {
    let _guard = isolated();

    let service = AuthService::with_store(CredentialsStore::default());
    let account = service.set_offline_username("Steve").await.unwrap();

    service.remove_account(account.id).await.unwrap();
    assert!(service.default_account().await.unwrap().is_none());
    assert!(!service.has_account().await);

    // Clearing through the switcher path works the same way.
    service.set_offline_username("Alex").await.unwrap();
    service.set_default_account(None).await.unwrap();
    assert!(service.default_account().await.unwrap().is_none());
}

#[tokio::test]
async fn setting_a_default_that_is_not_the_profile_fails() {
    let _guard = isolated();

    let service = AuthService::with_store(CredentialsStore::default());
    let account = service.set_offline_username("Steve").await.unwrap();

    assert!(service.set_default_account(Some(account.id)).await.is_ok());
    assert!(
        service
            .set_default_account(Some(Uuid::nil()))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn legacy_multi_account_files_migrate_to_one_profile() {
    let _guard = isolated();

    let dir = oneclient_common::paths::config_dir().unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    let msa_id = Uuid::new_v4();
    let offline_id = offline_uuid("Steve");

    let legacy = format!(
        r#"{{
            "users": {{
                "{msa_id}": {{"id":"{msa_id}","username":"MsaUser","access_token":"a","refresh_token":"r","expires":"2027-01-01T00:00:00Z","kind":"microsoft","signed_out":false}},
                "{offline_id}": {{"id":"{offline_id}","username":"Steve","access_token":"","refresh_token":"","expires":"2036-01-01T00:00:00Z","kind":"offline","signed_out":false}}
            }},
            "default_user": "{offline_id}"
        }}"#
    );
    std::fs::write(oneclient_common::paths::auth_file().unwrap(), legacy).unwrap();

    let store = CredentialsStore::load().await.unwrap();
    let profile = store.profile.expect("the default user must migrate");
    assert_eq!(profile.username, "Steve");
    assert_eq!(profile.id, offline_id);

    // The rewrite drops the Microsoft token fields entirely.
    let rewritten = std::fs::read_to_string(oneclient_common::paths::auth_file().unwrap()).unwrap();
    assert!(rewritten.contains("\"username\":\"Steve\""));
    assert!(!rewritten.contains("access_token"));
    assert!(!rewritten.contains("users"));
}

#[tokio::test]
async fn legacy_files_without_a_default_prefer_an_offline_account() {
    let _guard = isolated();

    let dir = oneclient_common::paths::config_dir().unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    let msa_id = Uuid::new_v4();
    let legacy = format!(
        r#"{{
            "users": {{
                "{msa_id}": {{"id":"{msa_id}","username":"MsaUser","kind":"microsoft"}},
                "00000000-0000-0000-0000-000000000001": {{"username":"Steve","kind":"offline"}}
            }}
        }}"#
    );
    std::fs::write(oneclient_common::paths::auth_file().unwrap(), legacy).unwrap();

    let profile = CredentialsStore::load().await.unwrap().profile.unwrap();
    assert_eq!(profile.username, "Steve");
}

#[test]
fn the_launch_token_is_the_conventional_offline_placeholder() {
    assert_eq!(OFFLINE_ACCESS_TOKEN, "0");
}
