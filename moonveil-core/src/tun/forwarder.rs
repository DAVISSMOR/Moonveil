#[cfg(target_os = "linux")]
mod imp {
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use crate::session::Session;
    use crate::transport::{TransportError, TransportResult};
    use crate::tun::TunDevice;

    pub struct IpForwarder {
        tun: Arc<TunDevice>,
        session: Arc<Mutex<Session>>,
        running: Arc<tokio::sync::atomic::AtomicBool>,
    }

    impl IpForwarder {
        pub async fn new(tun: TunDevice, session: Session) -> Self {
            Self {
                tun: Arc::new(tun),
                session: Arc::new(Mutex::new(session)),
                running: Arc::new(tokio::sync::atomic::AtomicBool::new(true)),
            }
        }

        pub async fn run(&self) -> TransportResult<()> {
            let running_a = self.running.clone();
            let tun_a = self.tun.clone();
            let session_a = self.session.clone();

            let task_a = tokio::spawn(async move {
                while running_a.load(tokio::sync::atomic::Ordering::Relaxed) {
                    let packet = tun_a.read_packet().await?;
                    session_a.lock().await.send(packet).await?;
                }
                Ok::<(), TransportError>(())
            });

            let running_b = self.running.clone();
            let tun_b = self.tun.clone();
            let session_b = self.session.clone();

            let task_b = tokio::spawn(async move {
                while running_b.load(tokio::sync::atomic::Ordering::Relaxed) {
                    let data = session_b.lock().await.recv().await?;
                    tun_b.write_packet(&data).await?;
                }
                Ok::<(), TransportError>(())
            });

            let a = task_a.await.map_err(|e| {
                TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;
            a?;

            let b = task_b.await.map_err(|e| {
                TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;
            b?;

            Ok(())
        }

        pub fn stop(&self) {
            self.running
                .store(false, tokio::sync::atomic::Ordering::Relaxed);
        }
    }
}

#[cfg(target_os = "linux")]
pub use imp::IpForwarder;

#[cfg(not(target_os = "linux"))]
pub struct IpForwarder;

#[cfg(not(target_os = "linux"))]
impl IpForwarder {
    pub async fn new(_tun: crate::tun::TunDevice, _session: crate::session::Session) -> Self {
        Self
    }

    pub async fn run(&self) -> crate::transport::TransportResult<()> {
        Err(crate::transport::TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "TUN interface is only supported on Linux",
        )))
    }

    pub fn stop(&self) {}
}
