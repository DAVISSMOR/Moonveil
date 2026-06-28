use async_trait::async_trait;

use super::Cipher;
use crate::packet::Packet;
use crate::transport::{Transport, TransportResult};

/// A transport wrapper that transparently encrypts outgoing packets and
/// decrypts incoming packets using the provided cipher.
///
/// Upper layers (Session, Multiplexer) interact with this exactly as they
/// would with any other `Transport` — encryption is invisible to them.
pub struct EncryptedTransport {
    inner: Box<dyn Transport + Send + Sync>,
    cipher: Box<dyn Cipher + Send + Sync>,
}

impl EncryptedTransport {
    /// Wrap an existing transport with encryption.
    pub fn new(
        inner: Box<dyn Transport + Send + Sync>,
        cipher: Box<dyn Cipher + Send + Sync>,
    ) -> Self {
        Self { inner, cipher }
    }
}

#[async_trait]
impl Transport for EncryptedTransport {
    async fn send(&self, mut packet: Packet) -> TransportResult<()> {
        let encrypted = self
            .cipher
            .encrypt(&packet.payload)
            .map_err(|e| crate::transport::TransportError::Serialization(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string()).into(),
            ))?;
        packet.payload = encrypted;
        self.inner.send(packet).await
    }

    async fn recv(&self) -> TransportResult<Packet> {
        let mut packet = self.inner.recv().await?;
        let decrypted = self
            .cipher
            .decrypt(&packet.payload)
            .map_err(|e| crate::transport::TransportError::Serialization(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string()).into(),
            ))?;
        packet.payload = decrypted;
        Ok(packet)
    }

    async fn connect(&self) -> TransportResult<()> {
        self.inner.connect().await
    }

    async fn close(&self) -> TransportResult<()> {
        self.inner.close().await
    }
}
