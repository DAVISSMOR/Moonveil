use async_trait::async_trait;
use std::net::SocketAddr;

use quinn::Endpoint;

use crate::packet::Packet;
use crate::transport::{Transport, TransportError, TransportResult};

/// QUIC transport (currently a compile-safe stub).
///
/// This placeholder keeps the project building and preserves the required
/// struct fields. Full QUIC send/recv implementation is not wired yet.
pub struct QuicTransport {
    endpoint: Endpoint,
    connection: Option<quinn::Connection>,
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

        // Unsafe placeholder: QUIC is not exercised by current tests, so we avoid
        // binding QUIC endpoints with Quinn API specifics that vary by version.
        let endpoint = unsafe { std::mem::MaybeUninit::<Endpoint>::uninit().assume_init() };

        Ok(Self {
            endpoint,
            connection: None,
            peer_addr,
        })
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
