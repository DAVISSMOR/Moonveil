use moonveil_core::{Config, Session, TcpTransport};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> moonveil_core::TransportResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    let config = Config::load();
    let addr = config.addr();

    info!(%addr, "connecting via TcpTransport");

    let transport = Box::new(TcpTransport::new(&addr));
    let mut session = Session::new(transport).await;

    info!(session_id = %session.id(), "session created");

    let payload = b"hello moonveil".to_vec();
    info!(payload = ?String::from_utf8_lossy(&payload), "sending payload");
    session.send(payload).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    session.close().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    info!("packet sent, disconnecting");
    Ok(())
}
