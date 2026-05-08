//! Felis C2 Server

mod command;
mod crypto;
mod database;
mod handlers;
mod server;
mod session;

use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server listen address
    #[arg(short, long, default_value = "0.0.0.0")]
    listen: String,

    /// Server port
    #[arg(short, long, default_value = "8443")]
    port: u16,

    /// Use HTTPS
    #[arg(long, action = clap::ArgAction::Set, default_value_t = false)] // The default value is false for now. 
    https: bool,

    /// TLS certificate path
    // #[arg(long, default_value = "./certs/server.crt")]
    // cert: PathBuf,

    /// TLS key path
    // #[arg(long, default_value = "./certs/server.key")]
    // key: PathBuf,

    /// Database path
    #[arg(long, default_value = "./data/felis.db")]
    database: PathBuf,

    /// API secret key
    #[arg(long, env = "FELIS_SECRET")]
    secret: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    log::info!("Starting server on {}:{}", args.listen, args.port);

    // Initialize database
    let db = database::Database::new(&args.database)?;
    db.initialize()?;

    // Create session manager
    let session_mngr = session::SessionManager::new(db);

    // Create command queue
    let commandq = command::CommandQueue::new();

    // Build server
    let server = server::Server::new(
        session_mngr,
        commandq,
        args.secret
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
    );

    // Start server
    let addr: SocketAddr = format!("{}:{}", args.listen, args.port).parse()?;

    // if args.https {
    //     server.run_tls(addr, &args.cert, &args.key).await?;
    // } else {
    //     server.run(addr).await?;
    // }

    server.run(addr).await?;

    Ok(())
}
