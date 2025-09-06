use anyhow::Result;
use clap::Parser;
use std::io::{self, BufRead, BufReader, Write};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub mod mcp;
pub mod p4;

use mcp::{MCPMessage, MCPResponse, MCPServer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if args.debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting p4-mcp server");

    // Create MCP server
    let mut server = MCPServer::new();

    // Set up communication channels
    let (tx, mut rx) = mpsc::unbounded_channel::<MCPMessage>();

    // Spawn task to handle stdin
    let stdin_tx = tx.clone();
    tokio::spawn(async move {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if let Ok(message) = serde_json::from_str::<MCPMessage>(&line) {
                        if stdin_tx.send(message).is_err() {
                            break;
                        }
                    } else {
                        warn!("Failed to parse message: {}", line);
                    }
                }
                Err(e) => {
                    error!("Error reading stdin: {}", e);
                    break;
                }
            }
        }
    });

    // Main message processing loop
    while let Some(message) = rx.recv().await {
        match server.handle_message(message).await {
            Ok(Some(response)) => {
                let json = serde_json::to_string(&response)?;
                println!("{}", json);
                io::stdout().flush()?;
            }
            Ok(None) => {
                // No response needed
            }
            Err(e) => {
                error!("Error handling message: {}", e);
            }
        }
    }

    info!("p4-mcp server shutting down");
    Ok(())
}
