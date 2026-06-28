use clap::{Parser, Subcommand};
use moonveil_core::{MoonveilConfig, Session, TcpTransport};
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut config = MoonveilConfig::load_from_file(&cli.config)
        .unwrap_or_default();

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
            if let Some(h) = host { config.server.host = h; }
            if let Some(p) = port { config.server.port = p; }

            let addr = config.addr();
            info!(%addr, "connecting");

            let transport = Box::new(TcpTransport::new(&addr));
            let mut session = Session::new(transport).await;
            info!(session_id = %session.id(), "session created");

            session.send(b"hello moonveil".to_vec()).await
                .map_err(|e| e.to_string())?;
            session.close().await
                .map_err(|e| e.to_string())?;

            info!("done");
        }
        Commands::Status => {
            println!("moonveil-client: ready");
            println!("server: {}", config.addr());
        }
    }

    Ok(())
}