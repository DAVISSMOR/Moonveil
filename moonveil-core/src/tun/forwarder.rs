use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use crate::session::Session;
use crate::transport::TransportResult;
use crate::tun::TunDevice;

pub struct IpForwarder {
    tun: Arc<TunDevice>,
    session: Arc<Mutex<Session>>,
    running: Arc<AtomicBool>,
}

impl IpForwarder {
    pub async fn new(tun: TunDevice, session: Session) -> Self {
        Self {
            tun: Arc::new(tun),
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
                    Err(e) => { eprintln!("tun read error: {e}"); break; }
                };
                if let Err(e) = session_a.lock().await.send(packet).await {
                    eprintln!("session send error: {e}"); break;
                }
            }
        });

        let running_b = self.running.clone();
        let tun_b = self.tun.clone();
        let session_b = self.session.clone();

        let task_b = tokio::spawn(async move {
            while running_b.load(Ordering::Relaxed) {
                let data = match session_b.lock().await.recv().await {
                    Ok(d) => d,
                    Err(e) => { eprintln!("session recv error: {e}"); break; }
                };
                if let Err(e) = tun_b.write_packet(&data).await {
                    eprintln!("tun write error: {e}"); break;
                }
            }
        });

        let _ = task_a.await;
        let _ = task_b.await;
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}