mod quic;
mod tcp;
mod udp;

pub use quic::QuicTransport;
pub use tcp::{TcpListener, TcpTransport};
pub use udp::UdpTransport;

use async_trait::async_trait;
use thiserror::Error;

use crate::packet::Packet;

pub type TransportResult<T> = std::result::Result<T, TransportError>;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("not connected")]
    NotConnected,

    #[error("already connected")]
    AlreadyConnected,

    #[error("no address configured for connect")]
    NoAddress,

    #[error("packet too large")]
    PacketTooLarge,

    #[error("connection timed out after {0} seconds")]
    ConnectTimeout(u64),
}

/// Async transport abstraction for sending and receiving packets.
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, packet: Packet) -> TransportResult<()>;
    async fn recv(&self) -> TransportResult<Packet>;
    async fn connect(&self) -> TransportResult<()>;
    async fn close(&self) -> TransportResult<()>;
}
