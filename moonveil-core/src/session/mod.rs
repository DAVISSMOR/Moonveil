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
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("session_id", &self.session_id)
            .finish()
    }
}

impl Session {
    pub async fn new(transport: Box<dyn Transport + Send + Sync>) -> Self {
        let _ = transport.connect().await;
        Self {
            transport,
            session_id: Uuid::new_v4(),
            state: Mutex::new(SessionState::Active),
            created_at: std::time::Instant::now(),
            packet_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn send(&self, payload: Vec<u8>) -> Result<(), SessionError> {
        let state = *self.state.lock().await;
        match state {
            SessionState::Active => {}
            SessionState::Closing | SessionState::Closed => return Err(SessionError::SessionClosed),
            SessionState::Connecting => return Err(SessionError::InvalidState(
                "send not allowed while Connecting".to_string()
            )),
        }
        let id = self.packet_counter.fetch_add(1, Ordering::Relaxed) + 1;
        self.transport.send(Packet::new(id, payload)).await?;
        Ok(())
    }

    pub async fn recv(&self) -> Result<Vec<u8>, SessionError> {
        let state = *self.state.lock().await;
        match state {
            SessionState::Active => {}
            SessionState::Closing | SessionState::Closed => return Err(SessionError::SessionClosed),
            SessionState::Connecting => return Err(SessionError::InvalidState(
                "recv not allowed while Connecting".to_string()
            )),
        }
        Ok(self.transport.recv().await?.payload)
    }

    pub fn id(&self) -> Uuid { self.session_id }

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