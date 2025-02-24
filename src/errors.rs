use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;