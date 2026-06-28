use async_trait::async_trait;

use crate::obfuscation::Obfuscator;
use crate::packet::Packet;
use crate::transport::{Transport, TransportResult};

pub struct ObfuscatedTransport {
    inner: Box<dyn Transport + Send + Sync>,
    obfuscator: Box<dyn Obfuscator + Send + Sync>,
}

impl ObfuscatedTransport {
    pub fn new(
        inner: Box<dyn Transport + Send + Sync>,
        obfuscator: Box<dyn Obfuscator + Send + Sync>,
    ) -> Self {
        Self { inner, obfuscator }
    }
}

#[async_trait]
impl Transport for ObfuscatedTransport {
    async fn send(&self, mut packet: Packet) -> TransportResult<()> {
        packet.payload = self.obfuscator.obfuscate(&packet.payload);
        self.inner.send(packet).await
    }

    async fn recv(&self) -> TransportResult<Packet> {
        let mut packet = self.inner.recv().await?;
        let deobfuscated = self
            .obfuscator
            .deobfuscate(&packet.payload)
            .map_err(|e| {
                crate::transport::TransportError::Serialization(
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string()).into(),
                )
            })?;
        packet.payload = deobfuscated;
        Ok(packet)
    }

    async fn connect(&self) -> TransportResult<()> {
        self.inner.connect().await
    }

    async fn close(&self) -> TransportResult<()> {
        self.inner.close().await
    }
}
