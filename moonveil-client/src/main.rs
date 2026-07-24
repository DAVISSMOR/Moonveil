use clap::{Parser, Subcommand};
use moonveil_core::{
    AesGcmCipher, EncryptedTransport, IpForwarder, MoonveilConfig, Session, TcpTransport,
    TunDevice,
};
use std::sync::Arc;
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
            let mut session = Session::try_new(transport).await?;
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
                let tun = Arc::new(TunDevice::new(&tun_name, 1500)?);
                tun.set_ip_address(&tun_addr)?;

                let key = decode_key_hex_32(&config.crypto.preshared_key)?;
                let cipher = Box::new(AesGcmCipher::new(key));
                let transport = Box::new(TcpTransport::new(&server));
                let encrypted_transport = EncryptedTransport::new(transport, cipher);

                let session = match Session::try_new(Box::new(encrypted_transport)).await {
                    Ok(s) => s,
                    Err(error) => {
                        eprintln!("Failed to connect to Moonveil server at {server}: {error}");
                        eprintln!("Check that:");
                        eprintln!("  1. The server is running (moonveil-server tun ...)");
                        eprintln!("  2. The port is open in your firewall / cloud security group");
                        eprintln!("  3. The server address is correct");
                        std::process::exit(1);
                    }
                };

                // Capture the current default route before replacing it.
                let original_default_route = std::process::Command::new("ip")
                    .args(["route", "show", "default"])
                    .output()
                    .ok()
                    .and_then(|output| {
                        String::from_utf8_lossy(&output.stdout)
                            .lines()
                            .next()
                            .map(|s| s.to_string())
                    });

                // Define cleanup logic: delete TUN interface and restore original default route.
                let cleanup = |tun_name: &str, original_route: &Option<String>| {
                    let _ = std::process::Command::new("ip")
                        .args(["link", "delete", tun_name])
                        .status();
                    if let Some(route) = original_route {
                        let parts: Vec<&str> = route.split_whitespace().collect();
                        if parts.len() >= 5 {
                            let _ = std::process::Command::new("ip")
                                .args(["route", "replace", "default", "via", parts[2], "dev", parts[4]])
                                .status();
                        }
                    }
                };

                // Spawn ctrl_c handler for graceful cleanup.
                let tun_name_for_cleanup = tun_name.clone();
                let original_route_for_cleanup = original_default_route.clone();
                tokio::spawn(async move {
                    let _ = tokio::signal::ctrl_c().await;
                    tracing::info!("received ctrl-c, cleaning up");
                    cleanup(&tun_name_for_cleanup, &original_route_for_cleanup);
                    std::process::exit(0);
                });

                // Replace default route only AFTER the tunnel connection is established,
                // so the client can still reach the server via the original route.
                std::process::Command::new("ip")
                    .args(["route", "replace", "default", "dev", &tun_name])
                    .status()?;

                let forwarder = IpForwarder::new(Arc::clone(&tun), session).await;
                if let Err(e) = forwarder.run().await {
                    tracing::error!(error = %e, "tunnel forwarder failed, cleaning up");
                    cleanup(&tun_name, &original_default_route);
                    return Err(e.into());
                }
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
