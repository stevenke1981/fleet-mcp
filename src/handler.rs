use rmcp::{
    ServerHandler, handler::server::router::tool::ToolRouter, handler::server::wrapper::Parameters,
    model::*, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::fleet::client::{FleetClient, MAX_REPORT_RESULTS};
use crate::fleet::types::{FleetConfig, Host, Policy, Query, QueryReport, Software, Vulnerability};

const MAX_REPORT_COLUMNS: usize = 32;
const MAX_REPORT_VALUE_CHARS: usize = 512;

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
    /// Page number for pagination
    pub page: Option<u64>,
    /// Results per page (1-50)
    pub per_page: Option<u64>,
}

#[derive(Deserialize, JsonSchema, Default)]
pub struct ListPageParams {
    /// Page number for pagination (default: 1)
    pub page: Option<u64>,
    /// Results per page, capped at 50 (default: 20)
    pub per_page: Option<u64>,
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
    /// Include the saved SQL text only when explicitly requested
    pub include_query: Option<bool>,
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

#[derive(Debug, Serialize)]
struct PageEnvelope<T> {
    items: Vec<T>,
    page: u64,
    per_page: u64,
    returned: usize,
    limit_reached: bool,
}

#[derive(Debug, Serialize)]
struct HostSummary {
    id: Option<u64>,
    hostname: Option<String>,
    display_name: Option<String>,
    platform: Option<String>,
    os_version: Option<String>,
    status: Option<String>,
    seen_time: Option<String>,
    team_id: Option<u64>,
    team_name: Option<String>,
    software_count: Option<u64>,
}

#[derive(Debug, Serialize)]
struct QuerySummary {
    id: Option<u64>,
    name: Option<String>,
    description: Option<String>,
    query: Option<String>,
    platform: Option<String>,
    observer_can_run: Option<bool>,
}

#[derive(Debug, Serialize)]
struct PolicySummary {
    id: Option<u64>,
    name: Option<String>,
    description: Option<String>,
    platform: Option<String>,
    resolution: Option<String>,
    critical: Option<bool>,
    passing_host_count: Option<u64>,
    failing_host_count: Option<u64>,
}

#[derive(Debug, Serialize)]
struct SoftwareSummary {
    id: Option<u64>,
    name: Option<String>,
    version: Option<String>,
    source: Option<String>,
    bundle_identifier: Option<String>,
    hosts_count: Option<u64>,
    versions_count: Option<u64>,
}

#[derive(Debug, Serialize)]
struct VulnerabilitySummary {
    cve: Option<String>,
    details_link: Option<String>,
    cvss_score: Option<f64>,
    epss_probability: Option<f64>,
    cisa_known_exploit: Option<bool>,
    cve_published: Option<String>,
    hosts_count: Option<u64>,
}

#[derive(Debug, Serialize)]
struct QueryReportSummary {
    query_id: Option<u64>,
    report_id: Option<u64>,
    report_clipped: Option<bool>,
    results: Vec<crate::fleet::types::QueryResult>,
    returned: usize,
    truncated: bool,
}

#[derive(Debug, Serialize)]
struct ServerSettingsSummary {
    live_query_disabled: Option<bool>,
    enable_analytics: Option<bool>,
}

#[derive(Debug, Serialize)]
struct FleetConfigSummary {
    org_name: Option<String>,
    server_settings: Option<ServerSettingsSummary>,
    mdm_enabled: Option<bool>,
    mdm_apple_enabled: Option<bool>,
    mdm_windows_enabled: Option<bool>,
}

fn page_values(params: &ListPageParams) -> Result<(u64, u64), String> {
    FleetClient::normalize_pagination(params.page, params.per_page)
        .map_err(|error| error.to_string())
}

fn page_envelope<T>(items: Vec<T>, page: u64, per_page: u64) -> PageEnvelope<T> {
    let returned = items.len();
    PageEnvelope {
        items,
        page,
        per_page,
        returned,
        // The API response types intentionally omit Fleet's optional total/meta
        // fields, so this means "the page limit was reached" rather than
        // claiming that another page definitely exists.
        limit_reached: returned == per_page as usize,
    }
}

fn host_summary(host: Host) -> HostSummary {
    HostSummary {
        id: host.id,
        hostname: host.hostname,
        display_name: host.display_name,
        platform: host.platform,
        os_version: host.os_version,
        status: host.status,
        seen_time: host.seen_time,
        team_id: host.team_id,
        team_name: host.team_name,
        software_count: host.software_count,
    }
}

fn query_summary(query: Query, include_query: bool) -> QuerySummary {
    QuerySummary {
        id: query.id,
        name: query.name,
        description: query.description,
        query: include_query.then_some(query.query).flatten(),
        platform: query.platform,
        observer_can_run: query.observer_can_run,
    }
}

fn policy_summary(policy: Policy) -> PolicySummary {
    PolicySummary {
        id: policy.id,
        name: policy.name,
        description: policy.description,
        platform: policy.platform,
        resolution: policy.resolution,
        critical: policy.critical,
        passing_host_count: policy.passing_host_count,
        failing_host_count: policy.failing_host_count,
    }
}

fn software_summary(software: Software) -> SoftwareSummary {
    SoftwareSummary {
        id: software.id,
        name: software.name,
        version: software.version,
        source: software.source,
        bundle_identifier: software.bundle_identifier,
        hosts_count: software.hosts_count,
        versions_count: software.versions_count,
    }
}

fn vulnerability_summary(vulnerability: Vulnerability) -> VulnerabilitySummary {
    VulnerabilitySummary {
        cve: vulnerability.cve,
        details_link: vulnerability.details_link,
        cvss_score: vulnerability.cvss_score,
        epss_probability: vulnerability.epss_probability,
        cisa_known_exploit: vulnerability.cisa_known_exploit,
        cve_published: vulnerability.cve_published,
        hosts_count: vulnerability.hosts_count,
    }
}

fn query_report_summary(report: QueryReport) -> QueryReportSummary {
    let mut results = report.results.unwrap_or_default();
    let truncated = results.len() > MAX_REPORT_RESULTS;
    results.truncate(MAX_REPORT_RESULTS);
    for result in &mut results {
        if let Some(columns) = result.columns.as_mut() {
            let mut retained = 0;
            columns.retain(|_, value| {
                if retained >= MAX_REPORT_COLUMNS {
                    return false;
                }
                retained += 1;
                *value = value.chars().take(MAX_REPORT_VALUE_CHARS).collect();
                true
            });
        }
    }
    QueryReportSummary {
        query_id: report.query_id,
        report_id: report.report_id,
        report_clipped: report.report_clipped,
        returned: results.len(),
        results,
        truncated,
    }
}

fn config_summary(config: FleetConfig) -> FleetConfigSummary {
    FleetConfigSummary {
        org_name: config.org_name,
        server_settings: config
            .server_settings
            .map(|settings| ServerSettingsSummary {
                live_query_disabled: settings.live_query_disabled,
                enable_analytics: settings.enable_analytics,
            }),
        mdm_enabled: config.mdm.as_ref().and_then(|mdm| mdm.enabled),
        mdm_apple_enabled: config.mdm.as_ref().and_then(|mdm| mdm.apple_bm_enabled),
        mdm_windows_enabled: config.mdm.as_ref().and_then(|mdm| mdm.windows_enabled),
    }
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
    allowed_tools: Option<HashSet<String>>,
}

#[tool_router(router = tool_router)]
impl FleetHandler {
    pub fn new(client: FleetClient) -> Self {
        let allowed_tools = std::env::var("FLEET_ALLOWED_TOOLS").ok().map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|tool| !tool.is_empty())
                .map(ToOwned::to_owned)
                .collect::<HashSet<_>>()
        });
        Self {
            client,
            tool_router: Self::tool_router(),
            allowed_tools,
        }
    }

    fn ensure_tool_allowed(&self, tool_name: &str) -> Result<(), String> {
        if self
            .allowed_tools
            .as_ref()
            .is_some_and(|allowed| !allowed.contains(tool_name))
        {
            return Err(format!(
                "tool {tool_name} is disabled by FLEET_ALLOWED_TOOLS"
            ));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Host tools
    // -----------------------------------------------------------------------

    #[tool(
        description = "List a bounded page of minimal host summaries from Fleet.\n\nSupports optional filters and page/per_page (default 20, maximum 50).",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_list_hosts(
        &self,
        params: Parameters<ListHostsParams>,
    ) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_list_hosts")?;
        let p = params.0;
        let (page, per_page) = FleetClient::normalize_pagination(p.page, p.per_page)
            .map_err(|error| error.to_string())?;
        tool_json(
            self.client
                .list_hosts(
                    p.query.as_deref(),
                    p.platform.as_deref(),
                    p.status.as_deref(),
                    p.team_id,
                    Some(page),
                    Some(per_page),
                )
                .await
                .map(|hosts| {
                    page_envelope(
                        hosts.into_iter().map(host_summary).collect(),
                        page,
                        per_page,
                    )
                }),
        )
    }

    #[tool(
        description = "Get a minimal, redacted summary for a specific host by Fleet ID",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_get_host(&self, params: Parameters<GetHostParams>) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_get_host")?;
        tool_json(
            self.client
                .get_host(params.0.host_id)
                .await
                .map(host_summary),
        )
    }

    #[tool(
        description = "Search a bounded page of minimal host summaries by hostname, UUID, or serial number",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_search_hosts(
        &self,
        params: Parameters<SearchHostsParams>,
    ) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_search_hosts")?;
        let p = params.0;
        let (page, per_page) = FleetClient::normalize_pagination(p.page, p.per_page)
            .map_err(|error| error.to_string())?;
        tool_json(
            self.client
                .search_hosts_page(&p.query, Some(page), Some(per_page))
                .await
                .map(|hosts| {
                    page_envelope(
                        hosts.into_iter().map(host_summary).collect(),
                        page,
                        per_page,
                    )
                }),
        )
    }

    #[tool(
        description = "Get a minimal, redacted host summary by hostname, UUID, or serial number",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_get_host_by_identifier(
        &self,
        params: Parameters<GetHostByIdentifierParams>,
    ) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_get_host_by_identifier")?;
        tool_json(
            self.client
                .get_host_by_identifier(&params.0.identifier)
                .await
                .map(host_summary),
        )
    }

    // -----------------------------------------------------------------------
    // Report tools
    // -----------------------------------------------------------------------

    #[tool(
        description = "List a bounded page of saved osquery report summaries. SQL text is omitted by default.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_list_reports(
        &self,
        params: Parameters<ListPageParams>,
    ) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_list_reports")?;
        let p = params.0;
        let (page, per_page) = page_values(&p)?;
        tool_json(
            self.client
                .list_reports_page(Some(page), Some(per_page))
                .await
                .map(|queries| {
                    page_envelope(
                        queries
                            .into_iter()
                            .map(|query| query_summary(query, false))
                            .collect(),
                        page,
                        per_page,
                    )
                }),
        )
    }

    #[tool(
        description = "Get a saved osquery report summary by ID; include_query opt-in returns its pre-defined SQL text",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_get_report(
        &self,
        params: Parameters<GetReportParams>,
    ) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_get_report")?;
        let p = params.0;
        tool_json(
            self.client
                .get_report(p.report_id)
                .await
                .map(|query| query_summary(query, p.include_query.unwrap_or(false))),
        )
    }

    #[tool(
        description = "Get at most 50 stored rows for a saved report by ID",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_get_report_data(
        &self,
        params: Parameters<GetReportDataParams>,
    ) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_get_report_data")?;
        tool_json(
            self.client
                .get_report_data(params.0.report_id)
                .await
                .map(query_report_summary),
        )
    }

    // -----------------------------------------------------------------------
    // Policy tools
    // -----------------------------------------------------------------------

    #[tool(
        description = "List a bounded page of compliance policy summaries",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_list_policies(
        &self,
        params: Parameters<ListPageParams>,
    ) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_list_policies")?;
        let p = params.0;
        let (page, per_page) = page_values(&p)?;
        tool_json(
            self.client
                .list_policies_page(Some(page), Some(per_page))
                .await
                .map(|policies| {
                    page_envelope(
                        policies.into_iter().map(policy_summary).collect(),
                        page,
                        per_page,
                    )
                }),
        )
    }

    #[tool(
        description = "Get a compliance policy summary by ID; SQL and author metadata are omitted",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_get_policy(
        &self,
        params: Parameters<GetPolicyParams>,
    ) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_get_policy")?;
        tool_json(
            self.client
                .get_policy(params.0.policy_id)
                .await
                .map(policy_summary),
        )
    }

    // -----------------------------------------------------------------------
    // Software & Vulnerability tools
    // -----------------------------------------------------------------------

    #[tool(
        description = "List a bounded page of software inventory summaries without installed paths",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_list_software(
        &self,
        params: Parameters<ListPageParams>,
    ) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_list_software")?;
        let p = params.0;
        let (page, per_page) = page_values(&p)?;
        tool_json(
            self.client
                .list_software_page(Some(page), Some(per_page))
                .await
                .map(|software| {
                    page_envelope(
                        software.into_iter().map(software_summary).collect(),
                        page,
                        per_page,
                    )
                }),
        )
    }

    #[tool(
        description = "List a bounded page of vulnerability summaries",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_list_vulnerabilities(
        &self,
        params: Parameters<ListPageParams>,
    ) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_list_vulnerabilities")?;
        let p = params.0;
        let (page, per_page) = page_values(&p)?;
        tool_json(
            self.client
                .list_vulnerabilities_page(Some(page), Some(per_page))
                .await
                .map(|vulnerabilities| {
                    page_envelope(
                        vulnerabilities
                            .into_iter()
                            .map(vulnerability_summary)
                            .collect(),
                        page,
                        per_page,
                    )
                }),
        )
    }

    #[tool(
        description = "Get a vulnerability summary for a validated CVE identifier",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_get_cve(&self, params: Parameters<GetCveParams>) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_get_cve")?;
        tool_json(
            self.client
                .get_cve(&params.0.cve_id)
                .await
                .map(vulnerability_summary),
        )
    }

    // -----------------------------------------------------------------------
    // System / Config tools
    // -----------------------------------------------------------------------

    #[tool(
        description = "Get a redacted Fleet configuration summary; server URLs and logos are omitted",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_get_config(&self) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_get_config")?;
        tool_json(self.client.get_config().await.map(config_summary))
    }

    #[tool(
        description = "Get Fleet server version information",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn fleet_get_version(&self) -> Result<String, String> {
        self.ensure_tool_allowed("fleet_get_version")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn report_summary_caps_rows_columns_and_values() {
        let mut columns = HashMap::new();
        for index in 0..(MAX_REPORT_COLUMNS + 8) {
            columns.insert(
                format!("column-{index}"),
                "x".repeat(MAX_REPORT_VALUE_CHARS + 32),
            );
        }
        let report = QueryReport {
            query_id: Some(1),
            report_id: Some(1),
            report_clipped: Some(false),
            results: Some(
                (0..(MAX_REPORT_RESULTS + 1))
                    .map(|_| crate::fleet::types::QueryResult {
                        host_id: Some(1),
                        host_name: Some("host".to_string()),
                        last_fetched: None,
                        columns: Some(columns.clone()),
                    })
                    .collect(),
            ),
        };

        let summary = query_report_summary(report);
        assert!(summary.truncated);
        assert_eq!(summary.returned, MAX_REPORT_RESULTS);
        assert_eq!(summary.results.len(), MAX_REPORT_RESULTS);
        let first_columns = summary.results[0].columns.as_ref().unwrap();
        assert_eq!(first_columns.len(), MAX_REPORT_COLUMNS);
        assert!(
            first_columns
                .values()
                .all(|value| value.chars().count() <= MAX_REPORT_VALUE_CHARS)
        );
    }
}
