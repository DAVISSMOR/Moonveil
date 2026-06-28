use clap::{Parser, Subcommand};
use moonveil_core::{
    AesGcmCipher, EncryptedTransport, IpForwarder, MoonveilConfig, Session, TcpTransport,
    TunDevice,
};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "moonveil-client")]
struct Cli {
    #[arg(long, default_value = "config/client.toml")]
    config: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Connect {
        #[arg(long)]
        host: Option<String>,
        #[arg(long)]
        port: Option<u16>,
    },
    Status,
    Tun {
        tun_name: String,
        tun_addr: String,
        server: String,
    },
}

fn decode_key_hex_32(hex_str: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let s = hex_str.trim();
    if s.len() != 64 {
        return Err(format!("invalid preshared_key length: expected 64 hex chars, got {}", s.len()).into());
    }

    let mut out = [0u8; 32];
    for i in 0..32 {
        let byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)?;
        out[i] = byte;
    }
    Ok(out)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut config = MoonveilConfig::load_from_file(&cli.config).unwrap_or_default();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(match config.log.level.as_str() {
                    "debug" => Level::DEBUG,
                    "warn" => Level::WARN,
                    "error" => Level::ERROR,
                    _ => Level::INFO,
                }.into())
        )
        .init();

    match cli.command {
        Commands::Connect { host, port } => {
            if let Some(h) = host {
                config.server.host = h;
            }
            if let Some(p) = port {
                config.server.port = p;
            }

            let addr = config.addr();
            info!(%addr, "connecting");

            let transport = Box::new(TcpTransport::new(&addr));
            let mut session = Session::new(transport).await;
            info!(session_id = %session.id(), "session created");

            session
                .send(b"hello moonveil".to_vec())
                .await
                .map_err(|e| e.to_string())?;
            session
                .close()
                .await
                .map_err(|e| e.to_string())?;

            info!("done");
        }
        Commands::Status => {
            println!("moonveil-client: ready");
            println!("server: {}", config.addr());
        }
        Commands::Tun {
            tun_name,
            tun_addr,
            server,
        } => {
            #[cfg(target_os = "linux")]
            {
                let tun = TunDevice::new(&tun_name, 1500)?;
                tun.set_ip_address(&tun_addr)?;

                std::process::Command::new("ip")
                    .args(["route", "add", "default", "dev", &tun_name])
                    .status()?;

                let key = decode_key_hex_32(&config.crypto.preshared_key)?;
                let cipher = Box::new(AesGcmCipher::new(key));
                let transport = Box::new(TcpTransport::new(&server));
                let encrypted_transport = EncryptedTransport::new(transport, cipher);

                let session = Session::new(Box::new(encrypted_transport)).await;
                let forwarder = IpForwarder::new(tun, session).await;
                forwarder.run().await?;
            }

            #[cfg(not(target_os = "linux"))]
            {
                return Err(
                    "TUN interface is only supported on Linux".to_string().into()
                );
            }
        }
    }

    Ok(())
}
