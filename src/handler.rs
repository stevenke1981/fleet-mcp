use rmcp::{
    ServerHandler, handler::server::router::tool::ToolRouter, handler::server::wrapper::Parameters,
    model::*, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::fleet::client::FleetClient;

fn tool_json<T: serde::Serialize>(result: anyhow::Result<T>) -> Result<String, String> {
    result
        .map_err(|error| error.to_string())
        .and_then(|value| serde_json::to_string_pretty(&value).map_err(|error| error.to_string()))
}

// ---------------------------------------------------------------------------
// Parameter structs for tool inputs
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct ListHostsParams {
    /// Optional search query
    pub query: Option<String>,
    /// Filter by platform (e.g. darwin, ubuntu, windows)
    pub platform: Option<String>,
    /// Filter by status (online, offline, mia)
    pub status: Option<String>,
    /// Filter by team ID
    pub team_id: Option<u64>,
    /// Page number for pagination
    pub page: Option<u64>,
    /// Results per page
    pub per_page: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetHostParams {
    /// Fleet host ID
    pub host_id: u64,
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchHostsParams {
    /// Search query
    pub query: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetHostByIdentifierParams {
    /// Host identifier (hostname, UUID, or serial number)
    pub identifier: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetReportParams {
    /// Report ID
    pub report_id: u64,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetReportDataParams {
    /// Report ID
    pub report_id: u64,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetPolicyParams {
    /// Policy ID
    pub policy_id: u64,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetCveParams {
    /// CVE identifier (e.g. CVE-2024-XXXXX)
    pub cve_id: String,
}

// ---------------------------------------------------------------------------
// FleetHandler — MCP server handler exposing all Fleet tools
// ---------------------------------------------------------------------------

/// The main MCP handler that exposes all Fleet tools.
///
/// Uses rmcp 0.16 `#[tool_router]` to register all tools on a single impl block
/// and `#[tool_handler]` to route incoming tool calls through the router.
pub struct FleetHandler {
    pub client: FleetClient,
    pub tool_router: ToolRouter<Self>,
}

#[tool_router(router = tool_router)]
impl FleetHandler {
    pub fn new(client: FleetClient) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    // -----------------------------------------------------------------------
    // Host tools
    // -----------------------------------------------------------------------

    #[tool(
        description = "List all hosts in Fleet Device Management.\n\nSupports optional filters:\n- query: free-text search across hostname, UUID, etc.\n- platform: filter by OS (darwin, ubuntu, windows, etc.)\n- status: filter by online/offline/mia status\n- team_id: filter by team ID\n- page / per_page: pagination control"
    )]
    async fn fleet_list_hosts(
        &self,
        params: Parameters<ListHostsParams>,
    ) -> Result<String, String> {
        let p = params.0;
        tool_json(
            self.client
                .list_hosts(
                    p.query.as_deref(),
                    p.platform.as_deref(),
                    p.status.as_deref(),
                    p.team_id,
                    p.page,
                    p.per_page,
                )
                .await,
        )
    }

    #[tool(description = "Get detailed information about a specific host by its Fleet ID")]
    async fn fleet_get_host(&self, params: Parameters<GetHostParams>) -> Result<String, String> {
        tool_json(self.client.get_host(params.0.host_id).await)
    }

    #[tool(description = "Search hosts by query string (hostname, UUID, serial number, etc.)")]
    async fn fleet_search_hosts(
        &self,
        params: Parameters<SearchHostsParams>,
    ) -> Result<String, String> {
        tool_json(self.client.search_hosts(&params.0.query).await)
    }

    #[tool(description = "Get a host by its identifier (hostname, UUID, serial number, etc.)")]
    async fn fleet_get_host_by_identifier(
        &self,
        params: Parameters<GetHostByIdentifierParams>,
    ) -> Result<String, String> {
        tool_json(
            self.client
                .get_host_by_identifier(&params.0.identifier)
                .await,
        )
    }

    // -----------------------------------------------------------------------
    // Report tools
    // -----------------------------------------------------------------------

    #[tool(description = "List all saved osquery reports in Fleet")]
    async fn fleet_list_reports(&self) -> Result<String, String> {
        tool_json(self.client.list_reports().await)
    }

    #[tool(description = "Get details of a specific saved osquery report by ID")]
    async fn fleet_get_report(
        &self,
        params: Parameters<GetReportParams>,
    ) -> Result<String, String> {
        tool_json(self.client.get_report(params.0.report_id).await)
    }

    #[tool(description = "Get stored data for a saved report by ID")]
    async fn fleet_get_report_data(
        &self,
        params: Parameters<GetReportDataParams>,
    ) -> Result<String, String> {
        tool_json(self.client.get_report_data(params.0.report_id).await)
    }

    // -----------------------------------------------------------------------
    // Policy tools
    // -----------------------------------------------------------------------

    #[tool(description = "List all compliance policies in Fleet")]
    async fn fleet_list_policies(&self) -> Result<String, String> {
        tool_json(self.client.list_policies().await)
    }

    #[tool(description = "Get a specific compliance policy by ID")]
    async fn fleet_get_policy(
        &self,
        params: Parameters<GetPolicyParams>,
    ) -> Result<String, String> {
        tool_json(self.client.get_policy(params.0.policy_id).await)
    }

    // -----------------------------------------------------------------------
    // Software & Vulnerability tools
    // -----------------------------------------------------------------------

    #[tool(description = "List all software installed across all hosts in Fleet")]
    async fn fleet_list_software(&self) -> Result<String, String> {
        tool_json(self.client.list_software().await)
    }

    #[tool(description = "List all known vulnerabilities (CVEs) tracked in Fleet")]
    async fn fleet_list_vulnerabilities(&self) -> Result<String, String> {
        tool_json(self.client.list_vulnerabilities().await)
    }

    #[tool(description = "Get detailed information about a specific CVE")]
    async fn fleet_get_cve(&self, params: Parameters<GetCveParams>) -> Result<String, String> {
        tool_json(self.client.get_cve(&params.0.cve_id).await)
    }

    // -----------------------------------------------------------------------
    // System / Config tools
    // -----------------------------------------------------------------------

    #[tool(description = "Get the current Fleet server configuration")]
    async fn fleet_get_config(&self) -> Result<String, String> {
        tool_json(self.client.get_config().await)
    }

    #[tool(description = "Get Fleet server version and build information")]
    async fn fleet_get_version(&self) -> Result<String, String> {
        tool_json(self.client.get_version().await)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for FleetHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "fleet-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some(
                "Fleet MCP server for Fleet Device Management (FleetDM).\n\n\
                 Use this server to:\n\
                 - List, search, and get details about managed hosts\n\
                 - View saved osquery reports and their stored data\n\
                 - Inspect compliance policies\n\
                 - Browse software inventory and vulnerabilities (CVEs)\n\
                 - Inspect Fleet server configuration and version\n\n\
                 This server intentionally exposes read-only tools only.\n\
                 Requires FLEET_SERVER_URL and FLEET_API_TOKEN to be configured."
                    .to_string(),
            ),
        }
    }
}
