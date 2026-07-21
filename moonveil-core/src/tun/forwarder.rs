use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tokio::task::JoinError;
use crate::session::Session;
use crate::transport::{TransportError, TransportResult};
use crate::tun::TunDevice;
use tracing::{debug, error};

pub struct IpForwarder {
    tun: Arc<TunDevice>,
    session: Arc<Mutex<Session>>,
    running: Arc<AtomicBool>,
}

impl IpForwarder {
    pub async fn new(tun: Arc<TunDevice>, session: Session) -> Self {
        Self {
            tun,
            session: Arc::new(Mutex::new(session)),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub async fn run(&self) -> TransportResult<()> {
        let running_a = self.running.clone();
        let tun_a = self.tun.clone();
        let session_a = self.session.clone();

        let task_a = tokio::spawn(async move {
            while running_a.load(Ordering::Relaxed) {
                let packet = match tun_a.read_packet().await {
                    Ok(p) => p,
                    Err(e) => { error!(error = %e, "tun read failed"); break; }
                };
                if let Err(e) = session_a.lock().await.send(packet).await {
                    error!(error = %e, "session send failed"); break;
                }
            }
            debug!("tun-to-session forwarder stopped");
        });

        let running_b = self.running.clone();
        let tun_b = self.tun.clone();
        let session_b = self.session.clone();

        let task_b = tokio::spawn(async move {
            while running_b.load(Ordering::Relaxed) {
                let data = match session_b.lock().await.recv().await {
                    Ok(d) => d,
                    Err(e) => { error!(error = %e, "session recv failed"); break; }
                };
                if let Err(e) = tun_b.write_packet(&data).await {
                    error!(error = %e, "tun write failed"); break;
                }
            }
            debug!("session-to-tun forwarder stopped");
        });

        wait_forwarder_task(task_a.await)?;
        wait_forwarder_task(task_b.await)?;
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

fn wait_forwarder_task(result: Result<(), JoinError>) -> TransportResult<()> {
    result.map_err(|error| {
        TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("forwarder task failed: {error}"),
        ))
    })
}