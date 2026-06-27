use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("transport error: {0}")]
    Transport(String),

    #[error(transparent)]
    Frame(#[from] FrameError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FrameError {
    #[error("frame decode error: {0}")]
    FrameDecodeError(String),

    #[error("checksum mismatch: expected {expected:#010x}, got {actual:#010x}")]
    ChecksumMismatch { expected: u32, actual: u32 },

    #[error("invalid frame version: {0}")]
    InvalidVersion(u8),
}
