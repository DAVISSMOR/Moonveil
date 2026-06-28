use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{Session, SessionError};


pub struct Multiplexer {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
}

#[derive(Debug, Error)]
pub enum MuxError {
    #[error("session not found: {0}")]
    SessionNotFound(Uuid),

    #[error(transparent)]
    Session(#[from] SessionError),
}

impl Multiplexer {
    pub async fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_session(&self, session: Session) -> Uuid {
        let id = session.id();
        let mut guard = self.sessions.write().await;
        guard.insert(id, session);
        id
    }

    pub async fn remove_session(&self, id: Uuid) -> Result<(), MuxError> {
        let mut guard = self.sessions.write().await;
        guard
            .remove(&id)
            .map(|_| ())
            .ok_or(MuxError::SessionNotFound(id))
    }

    pub async fn send_to(&self, id: Uuid, payload: Vec<u8>) -> Result<(), MuxError> {
        let guard = self.sessions.read().await;
        let session = guard.get(&id).ok_or(MuxError::SessionNotFound(id))?;
        session.send(payload).await?;
        Ok(())
    }

    pub async fn recv_from(&self, id: Uuid) -> Result<Vec<u8>, MuxError> {
        let guard = self.sessions.read().await;
        let session = guard.get(&id).ok_or(MuxError::SessionNotFound(id))?;
        let payload = session.recv().await?;
        Ok(payload)
    }

    pub async fn session_count(&self) -> usize {
        let guard = self.sessions.read().await;
        guard.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::packet::Packet;
    use crate::transport::{Transport, TransportError, TransportResult};

    use async_trait::async_trait;

    struct NoopTransport;

    #[async_trait]
    impl Transport for NoopTransport {
        async fn send(&self, _packet: Packet) -> TransportResult<()> {
            Ok(())
        }

        async fn recv(&self) -> TransportResult<Packet> {
            Err(TransportError::NotConnected)
        }

        async fn connect(&self) -> TransportResult<()> {
            Ok(())
        }

        async fn close(&self) -> TransportResult<()> {
            Ok(())
        }
    }

    async fn dummy_session() -> Session {
        Session::new(Box::new(NoopTransport)).await
    }

    #[tokio::test]
    async fn add_session_increases_count() {
        let mux = Multiplexer::new().await;
        assert_eq!(mux.session_count().await, 0);

        let s = dummy_session().await;
        let _id = mux.add_session(s).await;

        assert_eq!(mux.session_count().await, 1);
    }

    #[tokio::test]
    async fn remove_session_decreases_count() {
        let mux = Multiplexer::new().await;
        let s = dummy_session().await;
        let id = mux.add_session(s).await;
        assert_eq!(mux.session_count().await, 1);

        mux.remove_session(id).await.unwrap();
        assert_eq!(mux.session_count().await, 0);
    }

    #[tokio::test]
    async fn remove_nonexistent_session_returns_session_not_found() {
        let mux = Multiplexer::new().await;
        let missing = Uuid::new_v4();

        let err = mux.remove_session(missing).await.unwrap_err();
        assert!(matches!(err, MuxError::SessionNotFound(id) if id == missing));
    }
}


