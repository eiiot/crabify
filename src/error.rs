use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Spotify API error: {0}")]
    Api(String),

    #[error("No active Spotify device found. Open Spotify on a device and try again.")]
    NoActiveDevice,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
