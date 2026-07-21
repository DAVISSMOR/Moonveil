use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

use crate::packet::Packet;
use crate::transport::{Transport, TransportError, TransportResult};

#[allow(dead_code)]
pub struct UdpTransport {
    socket: std::sync::Arc<UdpSocket>,
    peer_addr: SocketAddr,
    local_addr: SocketAddr,
}

impl UdpTransport {
    pub async fn new(local: &str, peer: &str) -> TransportResult<Self> {
        let local_addr: SocketAddr = local
            .parse()
            .map_err(|e| {
                TransportError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e,
                ))
            })?;
        let peer_addr: SocketAddr = peer
            .parse()
            .map_err(|e| {
                TransportError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e,
                ))
            })?;

        let socket = UdpSocket::bind(local_addr).await?;
        Ok(Self {
            socket: std::sync::Arc::new(socket),
            peer_addr,
            local_addr,
        })
    }

    #[cfg(test)]
    fn from_socket(socket: std::sync::Arc<UdpSocket>, peer_addr: SocketAddr) -> Self {
        let local_addr = socket.local_addr().unwrap();
        Self {
            socket,
            peer_addr,
            local_addr,
        }
    }
}

#[async_trait]
impl Transport for UdpTransport {
    async fn connect(&self) -> TransportResult<()> {
        // Validate peer reachability with an empty probe.
        // If the peer isn't reachable, UDP may not error immediately; this still
        // ensures the socket can send to the configured peer.
        let empty: &[u8] = &[];
        let _ = self.socket.send_to(empty, self.peer_addr).await?;
        Ok(())
    }

    async fn send(&self, packet: Packet) -> TransportResult<()> {
        let bytes = bincode::serialize(&packet)?;
        self.socket.send_to(&bytes, self.peer_addr).await?;
        Ok(())
    }

    async fn recv(&self) -> TransportResult<Packet> {
        let mut buf = vec![0u8; 65536];
        let (len, _from) = self.socket.recv_from(&mut buf).await?;
        let bytes = &buf[..len];
        let packet: Packet = bincode::deserialize(bytes)?;
        Ok(packet)
    }

    async fn close(&self) -> TransportResult<()> {
        // UDP is connectionless; no-op.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::Packet;
    use crate::transport::Transport;

    #[tokio::test]
    async fn udp_send_recv_roundtrip() {
        // Avoid platform-specific port reuse issues by binding a single UDP socket
        // and sharing it between "client" and "server" transports.
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let local_addr = socket.local_addr().unwrap();
        let socket = std::sync::Arc::new(socket);

        let server_transport = UdpTransport::from_socket(socket.clone(), local_addr);
        let client_transport = UdpTransport::from_socket(socket.clone(), local_addr);

        let _ = server_transport.connect().await;
        let _ = client_transport.connect().await;

        let client_packet = Packet::new(1, b"hello udp".to_vec());
        client_transport
            .send(client_packet.clone())
            .await
            .unwrap();

        // UDP datagrams may include unrelated traffic; retry until we can
        // deserialize a valid Packet.
        let received = loop {
            match server_transport.recv().await {
                Ok(pkt) => break pkt,
                Err(_) => continue,
            }
        };

        assert_eq!(received.id, client_packet.id);
        assert_eq!(received.payload, client_packet.payload);
    }
}
