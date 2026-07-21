use std::sync::{atomic::{AtomicU64, Ordering}, Arc};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::packet::Packet;
use crate::transport::{Transport, TransportError, TransportResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Connecting, Active, Closing, Closed,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error("session is closed")]
    SessionClosed,
    #[error("invalid session state: {0}")]
    InvalidState(String),
}

pub struct Session {
    transport: Box<dyn Transport + Send + Sync>,
    session_id: Uuid,
    state: Mutex<SessionState>,
    created_at: std::time::Instant,
    packet_counter: Arc<AtomicU64>,
    bytes_sent: Arc<AtomicU64>,
    bytes_recv: Arc<AtomicU64>,
    packets_sent: Arc<AtomicU64>,
    packets_recv: Arc<AtomicU64>,
}

pub struct SessionMetrics {
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub packets_sent: u64,
    pub packets_recv: u64,
    pub uptime: std::time::Duration,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("session_id", &self.session_id)
            .finish()
    }
}

impl Session {
    pub async fn try_new(
        transport: Box<dyn Transport + Send + Sync>,
    ) -> Result<Self, SessionError> {
        transport.connect().await?;

        Ok(Self {
            transport,
            session_id: Uuid::new_v4(),
            state: Mutex::new(SessionState::Active),
            created_at: std::time::Instant::now(),
            packet_counter: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            bytes_recv: Arc::new(AtomicU64::new(0)),
            packets_sent: Arc::new(AtomicU64::new(0)),
            packets_recv: Arc::new(AtomicU64::new(0)),
        })
    }

    pub async fn new(transport: Box<dyn Transport + Send + Sync>) -> Self {
        Self::try_new(transport)
            .await
            .expect("transport connect failed in Session::new; use Session::try_new to handle errors")
    }

    pub async fn send(&self, payload: Vec<u8>) -> Result<(), SessionError> {
        let state = *self.state.lock().await;
        match state {
            SessionState::Active => {}
            SessionState::Closing | SessionState::Closed => {
                return Err(SessionError::SessionClosed)
            }
            SessionState::Connecting => {
                return Err(SessionError::InvalidState(
                    "send not allowed while Connecting".to_string(),
                ))
            }
        }
        let id = self.packet_counter.fetch_add(1, Ordering::Relaxed) + 1;

        self.packets_sent
            .fetch_add(1, Ordering::Relaxed);
        self.bytes_sent
            .fetch_add(payload.len() as u64, Ordering::Relaxed);

        self.transport.send(Packet::new(id, payload)).await?;
        Ok(())
    }

    pub async fn recv(&self) -> Result<Vec<u8>, SessionError> {
        let state = *self.state.lock().await;
        match state {
            SessionState::Active => {}
            SessionState::Closing | SessionState::Closed => {
                return Err(SessionError::SessionClosed)
            }
            SessionState::Connecting => {
                return Err(SessionError::InvalidState(
                    "recv not allowed while Connecting".to_string(),
                ))
            }
        }
        let payload = self.transport.recv().await?.payload;

        self.packets_recv
            .fetch_add(1, Ordering::Relaxed);
        self.bytes_recv
            .fetch_add(payload.len() as u64, Ordering::Relaxed);

        Ok(payload)
    }

    pub fn id(&self) -> Uuid { self.session_id }

    pub fn metrics(&self) -> SessionMetrics {
        SessionMetrics {
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_recv: self.bytes_recv.load(Ordering::Relaxed),
            packets_sent: self.packets_sent.load(Ordering::Relaxed),
            packets_recv: self.packets_recv.load(Ordering::Relaxed),
            uptime: self.created_at.elapsed(),
        }
    }

    pub fn state(&self) -> &SessionState {
        let _ = self.created_at;
        static CLOSED: SessionState = SessionState::Closed;
        static ACTIVE: SessionState = SessionState::Active;
        if let Ok(guard) = self.state.try_lock() {
            match *guard { SessionState::Closed => &CLOSED, _ => &ACTIVE }
        } else { &ACTIVE }
    }

    pub async fn close(&mut self) -> Result<(), SessionError> {
        {
            let mut st = self.state.lock().await;
            match *st {
                SessionState::Closed => return Err(SessionError::SessionClosed),
                SessionState::Closing => {}
                _ => { *st = SessionState::Closing; }
            }
        }
        self.transport.close().await?;
        *self.state.lock().await = SessionState::Closed;
        Ok(())
    }
}

#[allow(dead_code)]
fn _transport_result_alias(_r: TransportResult<()>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::TransportResult;
    use std::sync::Arc;

    struct DummyTransport {
        closed: Arc<std::sync::Mutex<bool>>,
        last_sent: Arc<std::sync::Mutex<Option<Vec<u8>>>>,
    }

    #[async_trait::async_trait]
    impl Transport for DummyTransport {
        async fn send(&self, packet: Packet) -> TransportResult<()> {
            *self.last_sent.lock().unwrap() = Some(packet.payload);
            Ok(())
        }
        async fn recv(&self) -> TransportResult<Packet> {
            let payload = self.last_sent.lock().unwrap().take().unwrap_or_default();
            Ok(Packet::new(1, payload))
        }
        async fn connect(&self) -> TransportResult<()> { Ok(()) }
        async fn close(&self) -> TransportResult<()> {
            *self.closed.lock().unwrap() = true;
            Ok(())
        }
    }

    fn make_session() -> Session {
        let t = DummyTransport {
            closed: Arc::new(std::sync::Mutex::new(false)),
            last_sent: Arc::new(std::sync::Mutex::new(None)),
        };
        Session {
            transport: Box::new(t),
            session_id: Uuid::new_v4(),
            state: tokio::sync::Mutex::new(SessionState::Active),
            created_at: std::time::Instant::now(),
            packet_counter: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            bytes_recv: Arc::new(AtomicU64::new(0)),
            packets_sent: Arc::new(AtomicU64::new(0)),
            packets_recv: Arc::new(AtomicU64::new(0)),
        }
    }

    #[tokio::test]
    async fn session_error_display_messages() {
        assert_eq!(SessionError::SessionClosed.to_string(), "session is closed");
        assert_eq!(
            SessionError::InvalidState("bad".to_string()).to_string(),
            "invalid session state: bad"
        );
    }

    #[tokio::test]
    async fn session_close_transitions_to_closed() {
        let mut session = make_session();
        assert_eq!(*session.state(), SessionState::Active);
        session.close().await.unwrap();
        assert_eq!(*session.state(), SessionState::Closed);
        let err = session.close().await.unwrap_err();
        assert!(matches!(err, SessionError::SessionClosed));
    }
}
