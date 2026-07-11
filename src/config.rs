use clap::Parser;

/// Fleet MCP Server — Rust implementation of a Model Context Protocol server
/// for Fleet Device Management (FleetDM).
#[derive(Parser, Debug, Clone)]
#[command(name = "fleet-mcp", version, about)]
pub struct CliConfig {
    /// Fleet server URL (for example, <https://your-fleet-instance.com>)
    #[arg(short = 'u', long = "url", env = "FLEET_SERVER_URL")]
    pub fleet_url: String,

    /// Fleet API token
    #[arg(short = 't', long = "token", env = "FLEET_API_TOKEN")]
    pub api_token: String,

    /// Verify SSL certificates
    #[arg(
        long = "verify-ssl",
        env = "FLEET_VERIFY_SSL",
        default_value = "true",
        value_parser = clap::value_parser!(bool)
    )]
    pub verify_ssl: bool,

    /// Request timeout in seconds
    #[arg(
        long = "timeout",
        env = "FLEET_TIMEOUT",
        default_value = "30",
        value_parser = clap::value_parser!(u64)
    )]
    pub timeout_secs: u64,
}

impl CliConfig {
    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.fleet_url.trim().is_empty() {
            return Err("FLEET_SERVER_URL must be set".to_string());
        }
        if self.api_token.trim().is_empty() {
            return Err("FLEET_API_TOKEN must be set".to_string());
        }
        if self.timeout_secs == 0 {
            return Err("FLEET_TIMEOUT must be greater than zero".to_string());
        }

        let url = url::Url::parse(self.fleet_url.trim())
            .map_err(|e| format!("Invalid FLEET_SERVER_URL: {e}"))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err("FLEET_SERVER_URL must use http or https".to_string());
        }
        if url.host_str().is_none() {
            return Err("FLEET_SERVER_URL must include a host".to_string());
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err("FLEET_SERVER_URL must not contain credentials".to_string());
        }
        if url.query().is_some() || url.fragment().is_some() {
            return Err("FLEET_SERVER_URL must not contain a query or fragment".to_string());
        }
        if url.scheme() == "http" {
            let is_loopback = matches!(
                url.host_str(),
                Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
            );
            if !is_loopback {
                return Err(
                    "FLEET_SERVER_URL must use https unless it targets loopback".to_string()
                );
            }
        }
        Ok(())
    }
}
