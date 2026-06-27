use moonveil_core::{Config, Packet, TcpTransport, Transport};
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

    let transport: Box<dyn Transport> = Box::new(TcpTransport::new(&addr));
    transport.connect().await?;

    let packet = Packet::new(1, b"hello moonveil");
    info!(id = packet.id, payload = ?packet.payload_str(), "sending packet");
    transport.send(packet).await?;
    transport.close().await?;

    info!("packet sent, disconnecting");
    Ok(())
}
