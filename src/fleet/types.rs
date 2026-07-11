use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Fleet API response wrappers
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HostsResponse {
    pub hosts: Vec<Host>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HostResponse {
    pub host: Host,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportsResponse {
    pub queries: Vec<Query>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportResponse {
    pub query: Query,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoliciesResponse {
    pub policies: Vec<Policy>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolicyResponse {
    pub policy: Policy,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoftwareTitlesResponse {
    pub software_titles: Vec<Software>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VulnerabilitiesResponse {
    pub vulnerabilities: Vec<Vulnerability>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VulnerabilityResponse {
    pub vulnerability: Vulnerability,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct FleetListResponse<T> {
    pub hosts: Option<Vec<T>>,
    pub queries: Option<Vec<T>>,
    pub policies: Option<Vec<T>>,
    pub software: Option<Vec<T>>,
    pub vulnerabilities: Option<Vec<Vulnerability>>,
    pub teams: Option<Vec<Team>>,
    pub users: Option<Vec<User>>,
    pub labels: Option<Vec<Label>>,
    pub packs: Option<Vec<Pack>>,
    pub scripts: Option<Vec<Script>>,
    pub osquery_tables: Option<Vec<OsqueryTable>>,
    pub activities: Option<Vec<Activity>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct FleetSingleResponse<T> {
    pub host: Option<T>,
    pub query: Option<T>,
    pub policy: Option<T>,
    pub software: Option<T>,
    pub vulnerability: Option<Vulnerability>,
    pub team: Option<T>,
    pub user: Option<T>,
    pub label: Option<T>,
    pub pack: Option<T>,
    pub script: Option<T>,
    pub osquery_table: Option<OsqueryTable>,
}

// ---------------------------------------------------------------------------
// Host types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct Host {
    pub id: Option<u64>,
    pub hostname: Option<String>,
    pub display_name: Option<String>,
    pub platform: Option<String>,
    pub os_version: Option<String>,
    pub status: Option<String>,
    #[serde(alias = "last_seen")]
    pub seen_time: Option<String>,
    pub uptime: Option<u64>,
    pub memory: Option<u64>,
    pub cpu_type: Option<String>,
    pub cpu_subtype: Option<String>,
    pub cpu_brand: Option<String>,
    pub cpu_physical_cores: Option<u32>,
    pub cpu_logical_cores: Option<u32>,
    pub gigs_disk_space_available: Option<f64>,
    pub percent_disk_space_available: Option<f64>,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub policy_compliance: Option<String>,
    pub refetch_requested: Option<bool>,
    pub detail_updated_at: Option<String>,
    pub software_count: Option<u64>,
    pub mdm: Option<HostMdm>,
    pub orbit_version: Option<String>,
    pub munki_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct HostMemory {
    pub total_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct HostCpu {
    pub brand: Option<String>,
    pub physical_cores: Option<u32>,
    pub logical_cores: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct HostDisk {
    pub total_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct HostMdm {
    pub enrollment_status: Option<String>,
    pub server_url: Option<String>,
    pub mdm_solution: Option<String>,
}

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct Query {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<String>,
    pub platform: Option<String>,
    pub interval: Option<u32>,
    pub observer_can_run: Option<bool>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct QueryReport {
    pub query_id: Option<u64>,
    pub report_id: Option<u64>,
    pub report_clipped: Option<bool>,
    pub results: Option<Vec<QueryResult>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct QueryResult {
    pub host_id: Option<u64>,
    #[serde(alias = "hostname")]
    pub host_name: Option<String>,
    pub last_fetched: Option<String>,
    pub columns: Option<std::collections::HashMap<String, String>>,
}

// ---------------------------------------------------------------------------
// Policy types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct Policy {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<String>,
    pub platform: Option<String>,
    pub resolution: Option<String>,
    pub critical: Option<bool>,
    pub author_name: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub passing_host_count: Option<u64>,
    pub failing_host_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct PolicyResult {
    pub policy_id: Option<u64>,
    pub hostname: Option<String>,
    pub host_id: Option<u64>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// Software & Vulnerability types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct Software {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub source: Option<String>,
    pub bundle_identifier: Option<String>,
    pub vulnerabilities: Option<Vec<String>>,
    pub installed_paths: Option<Vec<String>>,
    pub hosts_count: Option<u64>,
    pub versions_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct Vulnerability {
    pub cve: Option<String>,
    pub details_link: Option<String>,
    pub cvss_score: Option<f64>,
    pub epss_probability: Option<f64>,
    pub cisa_known_exploit: Option<bool>,
    pub cve_published: Option<String>,
    pub hosts_count: Option<u64>,
    pub hosts_count_updated_at: Option<String>,
    pub cve_description: Option<String>,
}

// ---------------------------------------------------------------------------
// Other types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct Team {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub host_count: Option<u64>,
    pub user_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct User {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub global_role: Option<String>,
    pub sso_enabled: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct Label {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub label_type: Option<String>,
    pub host_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct Pack {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub host_count: Option<u64>,
    pub query_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct Script {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub script_type: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct OsqueryTable {
    pub name: Option<String>,
    pub description: Option<String>,
    pub platform: Option<String>,
    pub columns: Option<Vec<OsqueryColumn>>,
    pub examples: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct OsqueryColumn {
    pub name: Option<String>,
    pub description: Option<String>,
    pub column_type: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct Activity {
    pub id: Option<u64>,
    pub action: Option<String>,
    pub actor_full_name: Option<String>,
    pub actor_email: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct FleetConfig {
    pub org_name: Option<String>,
    pub org_logo_url: Option<String>,
    pub server_settings: Option<ServerSettings>,
    pub mdm: Option<FleetMdmConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct ServerSettings {
    pub server_url: Option<String>,
    pub live_query_disabled: Option<bool>,
    pub enable_analytics: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct FleetMdmConfig {
    pub enabled: Option<bool>,
    pub apple_bm_enabled: Option<bool>,
    pub windows_enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema)]
pub struct FleetVersion {
    pub version: Option<String>,
    pub branch: Option<String>,
    pub revision: Option<String>,
    pub go_version: Option<String>,
    pub build_date: Option<String>,
    pub build_user: Option<String>,
}
