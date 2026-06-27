use std::sync::Arc;

use moonveil_core::{Config, Multiplexer, Session, TcpListener};
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
    let mux = Arc::new(Multiplexer::new().await);

    loop {
        let transport = listener.accept().await?;
        let mux = mux.clone();

        tokio::spawn(async move {
            let mut session = Session::new(Box::new(transport)).await;
            let session_id = mux.add_session(session).await;

            match mux.recv_from(session_id).await {
                Ok(payload) => {
                    info!(
                        %session_id,
                        payload = ?std::str::from_utf8(&payload).ok(),
                        bytes = payload.len(),
                        "received payload"
                    );
                }
                Err(e) => {
                    info!(%session_id, error = %e, "session recv failed");
                }
            }

            if let Err(e) = mux.remove_session(session_id).await {
                info!(%session_id, error = %e, "session remove failed");
            }
        });
    }
}

