use async_trait::async_trait;

use crate::packet::Packet;
use crate::transport::{Transport, TransportResult};

pub struct QuicTransport;

#[async_trait]
impl Transport for QuicTransport {
    async fn send(&self, _packet: Packet) -> TransportResult<()> {
        todo!()
    }

    async fn recv(&self) -> TransportResult<Packet> {
        todo!()
    }

    async fn connect(&self) -> TransportResult<()> {
        todo!()
    }

    async fn close(&self) -> TransportResult<()> {
        todo!()
    }
}
