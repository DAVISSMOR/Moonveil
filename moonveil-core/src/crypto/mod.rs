pub mod cipher;
pub mod encrypted;
pub mod handshake;

pub use cipher::AesGcmCipher;
pub use encrypted::EncryptedTransport;
pub use handshake::perform;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    Encrypt(String),

    #[error("decryption failed: {0}")]
    Decrypt(String),

    #[error("key generation failed: {0}")]
    KeyGen(String),

    #[error("handshake failed: {0}")]
    Handshake(String),

    #[error("transport error: {0}")]
    Transport(String),
}

/// Symmetric cipher trait for encrypting and decrypting data.
pub trait Cipher: Send + Sync {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError>;
}
