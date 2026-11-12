use uuid::Uuid;

pub type AuthResult<T> = Result<T, AuthError>;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid offline username: {reason}")]
    InvalidOfflineUsername { reason: String },

    #[error("account {0} is not registered")]
    AccountNotFound(Uuid),

    #[error(transparent)]
    Io(#[from] polyio::IOError),

    #[error(transparent)]
    Paths(#[from] oneclient_common::PathsError),
}
