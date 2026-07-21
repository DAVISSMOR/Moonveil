use clap::{Parser, Subcommand};
use moonveil_core::{
    IpForwarder, AesGcmCipher, EncryptedTransport, MoonveilConfig, Multiplexer, Session,
    TcpListener, TcpTransport, TunDevice,
};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "moonveil-server")]
struct Cli {
    #[arg(long, default_value = "config/server.toml")]
    config: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Start,
    Sessions,
    Tun { tun_name: String, tun_addr: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = MoonveilConfig::load_from_file(&cli.config).unwrap_or_default();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(
                match config.log.level.as_str() {
                    "debug" => Level::DEBUG,
                    "warn" => Level::WARN,
                    "error" => Level::ERROR,
                    _ => Level::INFO,
                }
                .into(),
            ),
        )
        .init();

    match cli.command {
        Commands::Start => {
            let addr = config.addr();
            info!(%addr, "listening");

            let listener = TcpListener::bind(&addr).await?;
            let mux = Arc::new(Multiplexer::new().await);

            loop {
                let transport = listener.accept().await?;
                let mux = Arc::clone(&mux);

                tokio::spawn(async move {
                    let session = match Session::try_new(Box::new(transport)).await {
                        Ok(session) => session,
                        Err(error) => {
                            info!(error = %error, "session setup failed");
                            return;
                        }
                    };
                    let id = mux.add_session(session).await;
                    info!(%id, "session connected");

                    match mux.recv_from(id).await {
                        Ok(payload) => {
                            info!(%id, payload = ?String::from_utf8_lossy(&payload), "received");
                        }
                        Err(e) => {
                            info!(%id, error = %e, "recv failed");
                        }
                    }

                    let _ = mux.remove_session(id).await;
                    info!(%id, "session removed");
                });
            }
        }
        Commands::Sessions => {
            let mux = Multiplexer::new().await;
            println!("active sessions: {}", mux.session_count().await);
        }
        Commands::Tun { tun_name, tun_addr } => {
            #[cfg(target_os = "linux")]
            {
                let tun = Arc::new(TunDevice::new(&tun_name, 1500)?);
                tun.set_ip_address(&tun_addr)?;
                TunDevice::enable_ip_forward()?;
                TunDevice::setup_nat("10.8.0.0/24")?;

                let addr = config.addr();
                info!(%addr, "tunnel listener");

                let listener = TcpListener::bind(&addr).await?;

                loop {
                    let transport = listener.accept().await?;
                    let session = Session::try_new(Box::new(transport)).await?;
                    let forwarder = IpForwarder::new(Arc::clone(&tun), session).await;
                    tokio::spawn(async move {
                        let _ = forwarder.run().await;
                    });
                }
            }

            #[cfg(not(target_os = "linux"))]
            {
                return Err(moonveil_core::error::Error::Config(
                    "TUN interface is only supported on Linux".to_string(),
                )
                .into());
            }
        }
    }

    Ok(())
}
