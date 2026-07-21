use async_trait::async_trait;
use std::net::SocketAddr;

use crate::packet::Packet;
use crate::transport::{Transport, TransportError, TransportResult};

/// QUIC transport (not yet implemented — returns NotConnected).
/// Full QUIC support is tracked in RFC-0004.
#[allow(dead_code)]
pub struct QuicTransport {
    peer_addr: SocketAddr,
}

impl QuicTransport {
    pub fn new(local: &str, peer: &str) -> TransportResult<Self> {
        let _local_addr: SocketAddr = local.parse().map_err(|e| {
            TransportError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
        })?;

        let peer_addr: SocketAddr = peer.parse().map_err(|e| {
            TransportError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
        })?;

        Ok(Self { peer_addr })
    }
}

#[async_trait]
impl Transport for QuicTransport {
    async fn send(&self, _packet: Packet) -> TransportResult<()> {
        Err(TransportError::NotConnected)
    }

    async fn recv(&self) -> TransportResult<Packet> {
        Err(TransportError::NotConnected)
    }

    async fn connect(&self) -> TransportResult<()> {
        Err(TransportError::NoAddress)
    }

    async fn close(&self) -> TransportResult<()> {
        Ok(())
    }
}
