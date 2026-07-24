use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener as TokioListener, TcpStream};
use tokio::sync::Mutex;

use crate::packet::Packet;
use crate::transport::{Transport, TransportError, TransportResult};

pub struct TcpTransport {
    addr: Option<String>,
    stream: Mutex<Option<TcpStream>>,
}

impl TcpTransport {
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: Some(addr.into()),
            stream: Mutex::new(None),
        }
    }

    pub fn from_stream(stream: TcpStream) -> Self {
        Self {
            addr: None,
            stream: Mutex::new(Some(stream)),
        }
    }

    async fn read_packet(stream: &mut TcpStream) -> TransportResult<Packet> {
        let len = stream.read_u32_le().await? as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;
        Ok(bincode::deserialize(&buf)?)
    }

    async fn write_packet(stream: &mut TcpStream, packet: &Packet) -> TransportResult<()> {
        let bytes = bincode::serialize(packet)?;
        let len = u32::try_from(bytes.len()).map_err(|_| TransportError::PacketTooLarge)?;
        stream.write_u32_le(len).await?;
        stream.write_all(&bytes).await?;
        stream.flush().await?;
        Ok(())
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn send(&self, packet: Packet) -> TransportResult<()> {
        let mut guard = self.stream.lock().await;
        let stream = guard.as_mut().ok_or(TransportError::NotConnected)?;
        Self::write_packet(stream, &packet).await
    }

    async fn recv(&self) -> TransportResult<Packet> {
        let mut guard = self.stream.lock().await;
        let stream = guard.as_mut().ok_or(TransportError::NotConnected)?;
        Self::read_packet(stream).await
    }

    async fn connect(&self) -> TransportResult<()> {
        let mut guard = self.stream.lock().await;
        if guard.is_some() {
            return Ok(());
        }
        let addr = self
            .addr
            .as_ref()
            .ok_or(TransportError::NoAddress)?;

        let timeout_secs = 10;
        let stream = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            TcpStream::connect(addr.as_str()),
        )
        .await
        .map_err(|_| TransportError::ConnectTimeout(timeout_secs))??;

        *guard = Some(stream);
        Ok(())
    }

    async fn close(&self) -> TransportResult<()> {
        let mut guard = self.stream.lock().await;
        if let Some(stream) = guard.as_mut() {
            stream.shutdown().await?;
        }
        *guard = None;
        Ok(())
    }
}

pub struct TcpListener {
    inner: TokioListener,
}

impl TcpListener {
    pub async fn bind(addr: &str) -> TransportResult<Self> {
        let inner = TokioListener::bind(addr).await?;
        Ok(Self { inner })
    }

    pub async fn accept(&self) -> TransportResult<TcpTransport> {
        let (stream, _) = self.inner.accept().await?;
        Ok(TcpTransport::from_stream(stream))
    }
}
