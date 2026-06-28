use thiserror::Error;

pub mod padding;
pub mod xor;
pub mod obfuscated_transport;

pub use padding::PaddingObfuscator;
pub use xor::XorObfuscator;
pub use obfuscated_transport::ObfuscatedTransport;

pub trait Obfuscator: Send + Sync {
    fn obfuscate(&self, data: &[u8]) -> Vec<u8>;
    fn deobfuscate(&self, data: &[u8]) -> Result<Vec<u8>, ObfuscationError>;
}

#[derive(Debug, Error)]
pub enum ObfuscationError {
    #[error("invalid data: {0}")]
    InvalidData(String),

    #[error("data too short")]
    TooShort,
}
