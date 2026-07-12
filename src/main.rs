mod config;
mod fleet;
mod handler;

use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

use crate::config::CliConfig;
use crate::fleet::client::FleetClient;
use crate::handler::FleetHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Parse CLI configuration
    let config = CliConfig::parse_with_timeout_alias();

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {e}");
        std::process::exit(1);
    }

    tracing::info!("Fleet MCP server starting — url={}", config.fleet_url);

    // Create Fleet API client
    let client = FleetClient::from_config(&config)?;
    tracing::info!("Fleet API client configured for {}", config.fleet_url);

    // Build MCP handler
    let handler = FleetHandler::new(client);

    // Serve via stdio transport
    let server = handler.serve(stdio()).await?;

    tracing::info!("Fleet MCP server ready (stdio transport)");

    // Wait for the MCP server to finish (client disconnects)
    server.waiting().await?;

    tracing::info!("Fleet MCP server shut down");
    Ok(())
}
