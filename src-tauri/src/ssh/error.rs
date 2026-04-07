use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppError {
    InvalidSession,
    AuthenticationFailed,
    Message(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSession => write!(f, "invalid session"),
            Self::AuthenticationFailed => write!(f, "authentication failed"),
            Self::Message(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<russh::Error> for AppError {
    fn from(value: russh::Error) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<russh_keys::Error> for AppError {
    fn from(value: russh_keys::Error) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<russh_sftp::client::error::Error> for AppError {
    fn from(value: russh_sftp::client::error::Error) -> Self {
        Self::Message(value.to_string())
    }
}
