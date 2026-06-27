use moonveil_core::{Config, TcpListener, Transport};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> moonveil_core::TransportResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    let config = Config::load();
    let addr = config.addr();

    info!(%addr, "listening via TcpTransport");

    let listener = TcpListener::bind(&addr).await?;
    let transport = listener.accept().await?;
    let packet = transport.recv().await?;

    info!(
        id = packet.id,
        timestamp = packet.timestamp,
        payload = ?packet.payload_str(),
        bytes = packet.payload.len(),
        "received packet"
    );

    transport.close().await?;
    Ok(())
}
