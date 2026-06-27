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

