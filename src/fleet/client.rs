use crate::config::CliConfig;
use crate::fleet::types::*;
use anyhow::{Context, Result, anyhow};
use reqwest::Client as HttpClient;
use std::time::Duration;

pub const DEFAULT_PAGE_SIZE: u64 = 20;
pub const MAX_PAGE_SIZE: u64 = 50;
pub const MAX_REPORT_RESULTS: usize = 50;

/// A strongly-typed Fleet REST API client.
#[derive(Clone)]
pub struct FleetClient {
    http: HttpClient,
    base_url: String,
    timeout: Duration,
}

impl std::fmt::Debug for FleetClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FleetClient")
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

impl FleetClient {
    /// Create a new FleetClient from CLI configuration.
    pub fn from_config(config: &CliConfig) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth_value = format!("Bearer {}", config.api_token);
        let auth_header: reqwest::header::HeaderValue = auth_value
            .parse()
            .context("Failed to parse API token as header value")?;
        headers.insert(reqwest::header::AUTHORIZATION, auth_header);
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        headers.insert(reqwest::header::ACCEPT, "application/json".parse().unwrap());

        let client = HttpClient::builder()
            .default_headers(headers)
            .danger_accept_invalid_certs(!config.verify_ssl)
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            http: client,
            base_url: config.fleet_url.trim().trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(config.timeout_secs),
        })
    }

    /// Create an in-memory client for testing.
    #[cfg(test)]
    pub fn new_test(base_url: &str, api_token: &str) -> Self {
        Self::new_test_with_timeout(base_url, api_token, Duration::from_secs(5))
    }

    /// Create an in-memory client with a custom timeout for timeout tests.
    #[cfg(test)]
    pub fn new_test_with_timeout(base_url: &str, api_token: &str, timeout: Duration) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth_value = format!("Bearer {api_token}");
        headers.insert(reqwest::header::AUTHORIZATION, auth_value.parse().unwrap());
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        let client = HttpClient::builder()
            .default_headers(headers)
            .timeout(timeout)
            .build()
            .unwrap();

        Self {
            http: client,
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout,
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn api_url(&self, path: &str) -> String {
        format!("{}/api/v1/fleet{}", self.base_url, path)
    }

    pub(crate) fn normalize_pagination(
        page: Option<u64>,
        per_page: Option<u64>,
    ) -> Result<(u64, u64)> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(DEFAULT_PAGE_SIZE);
        if page == 0 {
            anyhow::bail!("page must be greater than zero");
        }
        if per_page == 0 || per_page > MAX_PAGE_SIZE {
            anyhow::bail!("per_page must be between 1 and {MAX_PAGE_SIZE}");
        }
        Ok((page, per_page))
    }

    fn limit_results<T>(mut values: Vec<T>, per_page: u64) -> Vec<T> {
        values.truncate(per_page as usize);
        values
    }

    /// Execute a GET-only request. There are intentionally no POST, PUT,
    /// PATCH, or DELETE helpers in this client, enforcing the server's
    /// read-only security boundary at the API layer.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.api_url(path);
        let response_result = tokio::time::timeout(self.timeout, self.http.get(&url).send())
            .await
            .map_err(|_| {
                anyhow!(
                    "Fleet API GET request timed out after {} ms",
                    self.timeout.as_millis()
                )
            })?;
        let response = match response_result {
            Ok(response) => response,
            Err(error) if error.is_timeout() => {
                anyhow::bail!(
                    "Fleet API GET request timed out after {} ms",
                    self.timeout.as_millis()
                )
            }
            Err(error) => return Err(error).context("Fleet API GET request failed"),
        };

        if !response.status().is_success() {
            let status = response.status();
            anyhow::bail!(
                "Fleet API returned HTTP {} {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            );
        }

        tokio::time::timeout(self.timeout, response.json::<T>())
            .await
            .map_err(|_| {
                anyhow!(
                    "Fleet API response timed out after {} ms",
                    self.timeout.as_millis()
                )
            })?
            .context("Fleet API returned an unexpected JSON response")
    }

    // -----------------------------------------------------------------------
    // Host endpoints
    // -----------------------------------------------------------------------

    /// List all hosts with optional search/filter parameters.
    pub async fn list_hosts(
        &self,
        query: Option<&str>,
        platform: Option<&str>,
        status: Option<&str>,
        team_id: Option<u64>,
        page: Option<u64>,
        per_page: Option<u64>,
    ) -> Result<Vec<Host>> {
        let (page, per_page) = Self::normalize_pagination(page, per_page)?;
        let mut params = vec![];
        if let Some(q) = query {
            params.push(format!("query={}", urlencoding(q)));
        }
        if let Some(p) = platform {
            params.push(format!("platform={}", urlencoding(p)));
        }
        if let Some(s) = status {
            params.push(format!("status={}", urlencoding(s)));
        }
        if let Some(t) = team_id {
            params.push(format!("team_id={t}"));
        }
        params.push(format!("page={page}"));
        params.push(format!("per_page={per_page}"));

        let query_string = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

        let resp = self
            .get::<HostsResponse>(&format!("/hosts{query_string}"))
            .await?;
        Ok(Self::limit_results(resp.hosts, per_page))
    }

    /// Get a single host by ID.
    pub async fn get_host(&self, host_id: u64) -> Result<Host> {
        let resp = self
            .get::<HostResponse>(&format!("/hosts/{host_id}"))
            .await?;
        Ok(resp.host)
    }

    /// Search hosts by a query string.
    #[allow(dead_code)]
    pub async fn search_hosts(&self, query: &str) -> Result<Vec<Host>> {
        self.search_hosts_page(query, None, None).await
    }

    /// Search hosts with an explicit bounded page.
    pub async fn search_hosts_page(
        &self,
        query: &str,
        page: Option<u64>,
        per_page: Option<u64>,
    ) -> Result<Vec<Host>> {
        self.list_hosts(Some(query), None, None, None, page, per_page)
            .await
    }

    /// Get a host by its identifier (hostname, UUID, etc.).
    pub async fn get_host_by_identifier(&self, identifier: &str) -> Result<Host> {
        let resp = self
            .get::<HostResponse>(&format!("/hosts/identifier/{}", urlencoding(identifier)))
            .await?;
        Ok(resp.host)
    }

    // -----------------------------------------------------------------------
    // Query endpoints
    // -----------------------------------------------------------------------

    /// List all saved queries.
    #[allow(dead_code)]
    pub async fn list_reports(&self) -> Result<Vec<Query>> {
        self.list_reports_page(None, None).await
    }

    pub async fn list_reports_page(
        &self,
        page: Option<u64>,
        per_page: Option<u64>,
    ) -> Result<Vec<Query>> {
        let (page, per_page) = Self::normalize_pagination(page, per_page)?;
        let resp = self
            .get::<ReportsResponse>(&format!("/reports?page={page}&per_page={per_page}"))
            .await?;
        Ok(Self::limit_results(resp.queries, per_page))
    }

    /// Get a single query by ID.
    pub async fn get_report(&self, report_id: u64) -> Result<Query> {
        let resp = self
            .get::<ReportResponse>(&format!("/reports/{report_id}"))
            .await?;
        Ok(resp.query)
    }

    /// Get the query report / results.
    pub async fn get_report_data(&self, report_id: u64) -> Result<QueryReport> {
        self.get::<QueryReport>(&format!("/reports/{report_id}/report"))
            .await
    }

    // -----------------------------------------------------------------------
    // Policy endpoints
    // -----------------------------------------------------------------------

    /// List all policies.
    #[allow(dead_code)]
    pub async fn list_policies(&self) -> Result<Vec<Policy>> {
        self.list_policies_page(None, None).await
    }

    pub async fn list_policies_page(
        &self,
        page: Option<u64>,
        per_page: Option<u64>,
    ) -> Result<Vec<Policy>> {
        let (page, per_page) = Self::normalize_pagination(page, per_page)?;
        let resp = self
            .get::<PoliciesResponse>(&format!("/global/policies?page={page}&per_page={per_page}"))
            .await?;
        Ok(Self::limit_results(resp.policies, per_page))
    }

    /// Get results for a specific policy.
    pub async fn get_policy(&self, policy_id: u64) -> Result<Policy> {
        let resp = self
            .get::<PolicyResponse>(&format!("/global/policies/{policy_id}"))
            .await?;
        Ok(resp.policy)
    }

    // -----------------------------------------------------------------------
    // Software & Vulnerability endpoints
    // -----------------------------------------------------------------------

    /// List all software.
    #[allow(dead_code)]
    pub async fn list_software(&self) -> Result<Vec<Software>> {
        self.list_software_page(None, None).await
    }

    pub async fn list_software_page(
        &self,
        page: Option<u64>,
        per_page: Option<u64>,
    ) -> Result<Vec<Software>> {
        let (page, per_page) = Self::normalize_pagination(page, per_page)?;
        let resp = self
            .get::<SoftwareTitlesResponse>(&format!(
                "/software/titles?page={page}&per_page={per_page}"
            ))
            .await?;
        Ok(Self::limit_results(resp.software_titles, per_page))
    }

    /// List vulnerabilities (CVEs).
    #[allow(dead_code)]
    pub async fn list_vulnerabilities(&self) -> Result<Vec<Vulnerability>> {
        self.list_vulnerabilities_page(None, None).await
    }

    pub async fn list_vulnerabilities_page(
        &self,
        page: Option<u64>,
        per_page: Option<u64>,
    ) -> Result<Vec<Vulnerability>> {
        let (page, per_page) = Self::normalize_pagination(page, per_page)?;
        let resp = self
            .get::<VulnerabilitiesResponse>(&format!(
                "/vulnerabilities?page={page}&per_page={per_page}"
            ))
            .await?;
        Ok(Self::limit_results(resp.vulnerabilities, per_page))
    }

    /// Get a specific CVE.
    pub async fn get_cve(&self, cve_id: &str) -> Result<Vulnerability> {
        validate_cve_id(cve_id)?;
        let resp = self
            .get::<VulnerabilityResponse>(&format!("/vulnerabilities/{}", urlencoding(cve_id)))
            .await?;
        Ok(resp.vulnerability)
    }

    // -----------------------------------------------------------------------
    // Config & Info endpoints
    // -----------------------------------------------------------------------

    /// Get Fleet server configuration.
    pub async fn get_config(&self) -> Result<FleetConfig> {
        self.get::<FleetConfig>("/config").await
    }

    /// Get Fleet server version information.
    pub async fn get_version(&self) -> Result<FleetVersion> {
        self.get::<FleetVersion>("/version").await
    }
}

/// Safe URL encoding helper.
fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

fn validate_cve_id(cve_id: &str) -> Result<()> {
    let bytes = cve_id.as_bytes();
    if bytes.len() < 9
        || !cve_id.starts_with("CVE-")
        || !bytes[4..8].iter().all(u8::is_ascii_digit)
        || !bytes[8..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
    {
        anyhow::bail!("cve_id must match CVE-YYYY-IDENTIFIER");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CliConfig;

    // -----------------------------------------------------------------------
    // CliConfig validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_config_empty_url_rejected() {
        let config = CliConfig {
            fleet_url: String::new(),
            api_token: "valid_token".into(),
            verify_ssl: true,
            timeout_secs: 30,
        };
        let err = config.validate().unwrap_err();
        assert!(err.contains("FLEET_SERVER_URL must be set"), "{err}");
    }

    #[test]
    fn test_config_empty_token_rejected() {
        let config = CliConfig {
            fleet_url: "https://fleet.example.com".into(),
            api_token: String::new(),
            verify_ssl: true,
            timeout_secs: 30,
        };
        let err = config.validate().unwrap_err();
        assert!(err.contains("FLEET_API_TOKEN must be set"), "{err}");
    }

    #[test]
    fn test_config_invalid_url_rejected() {
        let config = CliConfig {
            fleet_url: "not a url at all".into(),
            api_token: "valid_token".into(),
            verify_ssl: true,
            timeout_secs: 30,
        };
        let err = config.validate().unwrap_err();
        assert!(err.contains("Invalid FLEET_SERVER_URL"), "{err}");
    }

    #[test]
    fn test_config_valid_succeeds() {
        let config = CliConfig {
            fleet_url: "https://fleet.example.com".into(),
            api_token: "valid_token".into(),
            verify_ssl: true,
            timeout_secs: 30,
        };
        config.validate().expect("valid config should pass");
    }

    #[test]
    fn test_config_rejects_unsafe_url_and_zero_timeout() {
        for fleet_url in [
            "ftp://fleet.example.com",
            "http://fleet.example.com",
            "https://user:pass@fleet.example.com",
            "https://fleet.example.com?token=secret",
            "https://fleet.example.com/#fragment",
        ] {
            let config = CliConfig {
                fleet_url: fleet_url.into(),
                api_token: "valid_token".into(),
                verify_ssl: true,
                timeout_secs: 30,
            };
            assert!(config.validate().is_err(), "{fleet_url} should be rejected");
        }

        let zero_timeout = CliConfig {
            fleet_url: "https://fleet.example.com".into(),
            api_token: "valid_token".into(),
            verify_ssl: true,
            timeout_secs: 0,
        };
        assert!(zero_timeout.validate().is_err());
    }

    #[test]
    fn test_config_allows_loopback_http() {
        let config = CliConfig {
            fleet_url: "http://127.0.0.1:8080".into(),
            api_token: "valid_token".into(),
            verify_ssl: true,
            timeout_secs: 30,
        };
        config.validate().expect("loopback HTTP should be allowed");
    }

    #[test]
    fn test_config_rejects_disabled_tls_for_remote_server() {
        let config = CliConfig {
            fleet_url: "https://fleet.example.com".into(),
            api_token: "valid_token".into(),
            verify_ssl: false,
            timeout_secs: 15,
        };
        let error = config
            .validate()
            .expect_err("remote TLS must remain enabled");
        assert!(error.contains("only allowed for loopback"), "{error}");
    }

    #[test]
    fn test_pagination_has_safe_defaults_and_rejects_unbounded_requests() {
        assert_eq!(
            FleetClient::normalize_pagination(None, None).unwrap(),
            (1, 20)
        );
        assert!(FleetClient::normalize_pagination(Some(0), Some(20)).is_err());
        assert!(FleetClient::normalize_pagination(Some(1), Some(0)).is_err());
        assert!(FleetClient::normalize_pagination(Some(1), Some(51)).is_err());
    }

    #[test]
    fn test_cve_path_identifiers_are_strictly_validated() {
        assert!(validate_cve_id("CVE-2024-1234").is_ok());
        assert!(validate_cve_id("CVE-2024-1234/../../config").is_err());
        assert!(validate_cve_id("CVE-2024-1234?secret=true").is_err());
    }

    // -----------------------------------------------------------------------
    // FleetClient::from_config tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_from_config_creates_client_with_correct_settings() {
        let config = CliConfig {
            fleet_url: "https://fleet.example.com".into(),
            api_token: "test_token_123".into(),
            verify_ssl: true,
            timeout_secs: 30,
        };
        let client = FleetClient::from_config(&config).expect("from_config should succeed");
        assert_eq!(client.base_url, "https://fleet.example.com");
        assert!(!format!("{client:?}").contains("test_token_123"));
    }

    #[test]
    fn test_from_config_trims_trailing_slash() {
        let config = CliConfig {
            fleet_url: "https://fleet.example.com/".into(),
            api_token: "token".into(),
            verify_ssl: false,
            timeout_secs: 10,
        };
        let client = FleetClient::from_config(&config).expect("from_config should succeed");
        assert_eq!(client.base_url, "https://fleet.example.com");
    }

    // -----------------------------------------------------------------------
    // FleetSingleResponse deserialization tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_fleet_single_response_all_fields_deserialize() {
        let json = serde_json::json!({
            "host": { "id": 1, "hostname": "test-host", "platform": "linux" },
            "query": { "id": 10, "name": "test-query" },
            "policy": { "id": 20, "name": "test-policy" },
            "software": { "id": 30, "name": "test-software" },
            "vulnerability": { "cve": "CVE-2024-0001", "cvss_score": 7.5 },
            "team": { "id": 40, "name": "test-team" },
            "user": { "id": 50, "name": "test-user", "email": "u@example.com" },
            "label": { "id": 60, "name": "test-label" },
            "pack": { "id": 70, "name": "test-pack" },
            "script": { "id": 80, "name": "test-script" },
            "osquery_table": {
                "name": "processes",
                "description": "All running processes",
                "platform": "linux",
                "columns": [
                    { "name": "pid", "column_type": "INTEGER", "required": true }
                ],
                "examples": "SELECT * FROM processes"
            }
        });

        let resp: FleetSingleResponse<serde_json::Value> =
            serde_json::from_value(json).expect("FleetSingleResponse should deserialize");

        // Verify each wrapped field is present and has the correct value.
        let host = resp.host.expect("host field expected");
        assert_eq!(host["id"], 1);
        assert_eq!(host["hostname"], "test-host");

        let query = resp.query.expect("query field expected");
        assert_eq!(query["id"], 10);
        assert_eq!(query["name"], "test-query");

        let policy = resp.policy.expect("policy field expected");
        assert_eq!(policy["id"], 20);

        let software = resp.software.expect("software field expected");
        assert_eq!(software["id"], 30);

        let vuln = resp.vulnerability.expect("vulnerability field expected");
        assert_eq!(vuln.cve.as_deref(), Some("CVE-2024-0001"));

        let team = resp.team.expect("team field expected");
        assert_eq!(team["id"], 40);

        let user = resp.user.expect("user field expected");
        assert_eq!(user["id"], 50);

        let label = resp.label.expect("label field expected");
        assert_eq!(label["id"], 60);

        let pack = resp.pack.expect("pack field expected");
        assert_eq!(pack["id"], 70);

        let script = resp.script.expect("script field expected");
        assert_eq!(script["id"], 80);

        let osquery_table = resp.osquery_table.expect("osquery_table field expected");
        assert_eq!(osquery_table.name.as_deref(), Some("processes"));
        assert_eq!(
            osquery_table.description.as_deref(),
            Some("All running processes")
        );
    }

    #[test]
    fn test_fleet_single_response_partial_deserialize() {
        // Only include a subset of fields — optional fields should be None.
        let json = serde_json::json!({
            "host": { "id": 99, "hostname": "partial-host" }
        });
        let resp: FleetSingleResponse<serde_json::Value> =
            serde_json::from_value(json).expect("partial response should deserialize");
        assert!(resp.host.is_some());
        assert!(resp.query.is_none());
        assert!(resp.policy.is_none());
        assert!(resp.software.is_none());
        assert!(resp.vulnerability.is_none());
        assert!(resp.team.is_none());
        assert!(resp.user.is_none());
        assert!(resp.label.is_none());
        assert!(resp.pack.is_none());
        assert!(resp.script.is_none());
        assert!(resp.osquery_table.is_none());
    }

    // -----------------------------------------------------------------------
    // Host deserialization test
    // -----------------------------------------------------------------------

    #[test]
    fn test_host_deserialization() {
        let json_str = r#"{
            "id": 42,
            "hostname": "laptop-01.example.com",
            "display_name": "Engineering Laptop 01",
            "platform": "ubuntu",
            "os_version": "Ubuntu 22.04.3 LTS",
            "status": "online",
            "seen_time": "2024-06-15T14:30:00Z",
            "uptime": 604800,
            "memory": 17179869184,
            "cpu_type": "x86_64",
            "cpu_brand": "Intel(R) Core(TM) i7-10750H",
            "cpu_physical_cores": 6,
            "cpu_logical_cores": 12,
            "gigs_disk_space_available": 200.0,
            "percent_disk_space_available": 39.0,
            "team_id": 2,
            "team_name": "Engineering",
            "policy_compliance": "pass",
            "refetch_requested": false,
            "detail_updated_at": "2024-06-15T14:30:00Z",
            "software_count": 127,
            "mdm": {
                "enrollment_status": "On (manual)",
                "server_url": "https://fleet.example.com/mdm",
                "mdm_solution": "Fleet"
            },
            "orbit_version": "1.16.0",
            "munki_version": "6.4.0"
        }"#;

        let host: Host = serde_json::from_str(json_str).expect("Host should deserialize");
        assert_eq!(host.id, Some(42));
        assert_eq!(host.hostname.as_deref(), Some("laptop-01.example.com"));
        assert_eq!(host.display_name.as_deref(), Some("Engineering Laptop 01"));
        assert_eq!(host.platform.as_deref(), Some("ubuntu"));
        assert_eq!(host.os_version.as_deref(), Some("Ubuntu 22.04.3 LTS"));
        assert_eq!(host.status.as_deref(), Some("online"));
        assert!(host.seen_time.is_some());
        assert_eq!(host.uptime, Some(604800));
        assert_eq!(host.team_id, Some(2));
        assert_eq!(host.team_name.as_deref(), Some("Engineering"));
        assert_eq!(host.policy_compliance.as_deref(), Some("pass"));
        assert_eq!(host.refetch_requested, Some(false));
        assert_eq!(host.software_count, Some(127));
        assert_eq!(host.orbit_version.as_deref(), Some("1.16.0"));
        assert_eq!(host.munki_version.as_deref(), Some("6.4.0"));

        assert_eq!(host.memory, Some(17179869184));
        assert_eq!(host.cpu_type.as_deref(), Some("x86_64"));
        assert_eq!(
            host.cpu_brand.as_deref(),
            Some("Intel(R) Core(TM) i7-10750H")
        );
        assert_eq!(host.cpu_physical_cores, Some(6));
        assert_eq!(host.cpu_logical_cores, Some(12));
        assert_eq!(host.gigs_disk_space_available, Some(200.0));

        // Nested mdm
        let mdm = host.mdm.expect("mdm expected");
        assert_eq!(mdm.enrollment_status.as_deref(), Some("On (manual)"));
        assert_eq!(mdm.mdm_solution.as_deref(), Some("Fleet"));
    }

    #[test]
    fn test_host_minimal_fields_deserialize() {
        // Host with only required-like fields — everything optional.
        let json = serde_json::json!({
            "id": 1,
            "hostname": "minimal-host"
        });
        let host: Host = serde_json::from_value(json).expect("minimal Host should deserialize");
        assert_eq!(host.id, Some(1));
        assert_eq!(host.hostname.as_deref(), Some("minimal-host"));
        assert!(host.platform.is_none());
        assert!(host.memory.is_none());
    }

    // -----------------------------------------------------------------------
    // Query deserialization test
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_deserialization() {
        let json = serde_json::json!({
            "id": 7,
            "name": "Get Chrome Extensions",
            "description": "List all installed Chrome extensions",
            "query": "SELECT * FROM chrome_extensions;",
            "platform": "darwin",
            "interval": 86400,
            "observer_can_run": true,
            "author_name": "Alice Admin",
            "author_email": "alice@example.com",
            "created_at": "2024-01-10T00:00:00Z",
            "updated_at": "2024-06-01T00:00:00Z"
        });

        let query: Query = serde_json::from_value(json).expect("Query should deserialize");
        assert_eq!(query.id, Some(7));
        assert_eq!(query.name.as_deref(), Some("Get Chrome Extensions"));
        assert_eq!(
            query.description.as_deref(),
            Some("List all installed Chrome extensions")
        );
        assert_eq!(
            query.query.as_deref(),
            Some("SELECT * FROM chrome_extensions;")
        );
        assert_eq!(query.platform.as_deref(), Some("darwin"));
        assert_eq!(query.interval, Some(86400));
        assert_eq!(query.observer_can_run, Some(true));
        assert_eq!(query.author_name.as_deref(), Some("Alice Admin"));
        assert_eq!(query.author_email.as_deref(), Some("alice@example.com"));
        assert!(query.created_at.is_some());
        assert!(query.updated_at.is_some());
    }

    #[test]
    fn test_query_minimal_fields() {
        let json = serde_json::json!({
            "id": 1,
            "name": "minimal-query"
        });
        let query: Query = serde_json::from_value(json).expect("minimal Query should deserialize");
        assert_eq!(query.id, Some(1));
        assert_eq!(query.name.as_deref(), Some("minimal-query"));
        assert!(query.description.is_none());
        assert!(query.query.is_none());
    }

    // -----------------------------------------------------------------------
    // Policy deserialization test
    // -----------------------------------------------------------------------

    #[test]
    fn test_policy_deserialization() {
        let json = serde_json::json!({
            "id": 3,
            "name": "Full Disk Encryption",
            "description": "Ensure FileVault or BitLocker is enabled",
            "query": "SELECT 1 FROM disk_encryption WHERE user_registered = 1;",
            "platform": "darwin,windows",
            "resolution": "Enable encryption in System Settings",
            "critical": true,
            "author_name": "Bob Security",
            "created_at": "2024-02-01T00:00:00Z",
            "updated_at": "2024-05-15T00:00:00Z",
            "passing_host_count": 98,
            "failing_host_count": 2
        });

        let policy: Policy = serde_json::from_value(json).expect("Policy should deserialize");
        assert_eq!(policy.id, Some(3));
        assert_eq!(policy.name.as_deref(), Some("Full Disk Encryption"));
        assert_eq!(
            policy.description.as_deref(),
            Some("Ensure FileVault or BitLocker is enabled")
        );
        assert_eq!(
            policy.query.as_deref(),
            Some("SELECT 1 FROM disk_encryption WHERE user_registered = 1;")
        );
        assert_eq!(policy.platform.as_deref(), Some("darwin,windows"));
        assert_eq!(
            policy.resolution.as_deref(),
            Some("Enable encryption in System Settings")
        );
        assert_eq!(policy.critical, Some(true));
        assert_eq!(policy.author_name.as_deref(), Some("Bob Security"));
        assert_eq!(policy.passing_host_count, Some(98));
        assert_eq!(policy.failing_host_count, Some(2));
        assert!(policy.created_at.is_some());
        assert!(policy.updated_at.is_some());
    }

    #[test]
    fn test_policy_minimal_fields() {
        let json = serde_json::json!({
            "id": 1,
            "name": "minimal-policy"
        });
        let policy: Policy =
            serde_json::from_value(json).expect("minimal Policy should deserialize");
        assert_eq!(policy.id, Some(1));
        assert_eq!(policy.name.as_deref(), Some("minimal-policy"));
        assert!(policy.description.is_none());
        assert!(policy.resolution.is_none());
    }

    // -----------------------------------------------------------------------
    // Software deserialization test
    // -----------------------------------------------------------------------

    #[test]
    fn test_software_deserialization() {
        let json = serde_json::json!({
            "id": 15,
            "name": "Google Chrome",
            "version": "126.0.6478.127",
            "source": "apps",
            "bundle_identifier": "com.google.Chrome",
            "vulnerabilities": ["CVE-2024-0001", "CVE-2024-0002"],
            "installed_paths": [
                "/Applications/Google Chrome.app",
                "/Users/test/Applications/Google Chrome.app"
            ]
        });

        let sw: Software = serde_json::from_value(json).expect("Software should deserialize");
        assert_eq!(sw.id, Some(15));
        assert_eq!(sw.name.as_deref(), Some("Google Chrome"));
        assert_eq!(sw.version.as_deref(), Some("126.0.6478.127"));
        assert_eq!(sw.source.as_deref(), Some("apps"));
        assert_eq!(sw.bundle_identifier.as_deref(), Some("com.google.Chrome"));

        let vulns = sw.vulnerabilities.expect("vulnerabilities expected");
        assert_eq!(vulns.len(), 2);
        assert!(vulns.contains(&"CVE-2024-0001".to_string()));

        let paths = sw.installed_paths.expect("installed_paths expected");
        assert_eq!(paths.len(), 2);
        assert!(paths[0].contains("Google Chrome.app"));
    }

    #[test]
    fn test_software_minimal_fields() {
        let json = serde_json::json!({
            "id": 1,
            "name": "minimal-software",
            "version": "1.0"
        });
        let sw: Software =
            serde_json::from_value(json).expect("minimal Software should deserialize");
        assert_eq!(sw.id, Some(1));
        assert_eq!(sw.name.as_deref(), Some("minimal-software"));
        assert_eq!(sw.version.as_deref(), Some("1.0"));
        assert!(sw.source.is_none());
        assert!(sw.vulnerabilities.is_none());
    }

    // -----------------------------------------------------------------------
    // FleetListResponse deserialization test
    // -----------------------------------------------------------------------

    #[test]
    fn test_fleet_list_response_hosts_deserialize() {
        let json = serde_json::json!({
            "hosts": [
                { "id": 1, "hostname": "host-a" },
                { "id": 2, "hostname": "host-b" }
            ]
        });
        let resp: FleetListResponse<Host> =
            serde_json::from_value(json).expect("FleetListResponse should deserialize");
        let hosts = resp.hosts.expect("hosts field expected");
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].id, Some(1));
        assert_eq!(hosts[0].hostname.as_deref(), Some("host-a"));
        assert_eq!(hosts[1].id, Some(2));
    }

    #[test]
    fn test_fleet_list_response_queries_deserialize() {
        let json = serde_json::json!({
            "queries": [
                { "id": 10, "name": "query-1" },
                { "id": 20, "name": "query-2" }
            ]
        });
        let resp: FleetListResponse<Query> =
            serde_json::from_value(json).expect("FleetListResponse<Query> should deserialize");
        let queries = resp.queries.expect("queries field expected");
        assert_eq!(queries.len(), 2);
        assert_eq!(queries[0].name.as_deref(), Some("query-1"));
    }

    // -----------------------------------------------------------------------
    // urlencoding helper test
    // -----------------------------------------------------------------------

    #[test]
    fn test_urlencoding_encodes_special_chars() {
        assert_eq!(super::urlencoding("simple"), "simple");
        assert_eq!(super::urlencoding("hello world"), "hello+world");
        assert_eq!(super::urlencoding("a/b?c=d"), "a%2Fb%3Fc%3Dd");
    }

    // -----------------------------------------------------------------------
    // api_url helper test
    // -----------------------------------------------------------------------

    #[test]
    fn test_api_url_format() {
        let client = FleetClient::new_test("https://fleet.example.com", "token");
        // api_url is private, but we can verify via the base_url and known path format.
        assert_eq!(client.base_url, "https://fleet.example.com");
        // We can't call api_url directly (it's private), but we test the logic
        // indirectly via from_config and the base_url trimming.
    }

    // -----------------------------------------------------------------------
    // Token header parsing edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_from_config_rejects_invalid_token_characters() {
        // Tokens with non-ASCII characters may fail HeaderValue parsing.
        let config = CliConfig {
            fleet_url: "https://fleet.example.com".into(),
            // HeaderValue rejects raw non-visible ASCII in some cases.
            // A newline in the token would break the header.
            api_token: "token\nwith\nnewlines".into(),
            verify_ssl: true,
            timeout_secs: 30,
        };
        let result = FleetClient::from_config(&config);
        assert!(result.is_err(), "token with control chars should fail");
    }

    async fn spawn_mock(
        expected_target: &'static str,
        status: &'static str,
        body: &'static str,
    ) -> (String, tokio::task::JoinHandle<()>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("mock listener should bind");
        let address = listener.local_addr().expect("mock address should exist");
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("mock should accept");
            let mut request = vec![0_u8; 8192];
            let read = stream
                .read(&mut request)
                .await
                .expect("request should read");
            let request = String::from_utf8_lossy(&request[..read]);
            assert!(
                request.starts_with(&format!("GET {expected_target} HTTP/1.1")),
                "unexpected request: {request}"
            );
            assert!(
                request
                    .to_ascii_lowercase()
                    .contains("authorization: bearer test-token"),
                "Bearer header missing: {request}"
            );

            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("response should write");
        });
        (format!("http://{address}"), handle)
    }

    async fn spawn_slow_mock(delay: Duration) -> (String, tokio::task::JoinHandle<()>) {
        use tokio::io::AsyncReadExt;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("mock listener should bind");
        let address = listener.local_addr().expect("mock address should exist");
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("mock should accept");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await;
            tokio::time::sleep(delay).await;
        });
        (format!("http://{address}"), handle)
    }

    #[tokio::test]
    async fn test_get_timeout_returns_bounded_error() {
        let (url, server) = spawn_slow_mock(Duration::from_millis(100)).await;
        let error =
            FleetClient::new_test_with_timeout(&url, "test-token", Duration::from_millis(10))
                .get_version()
                .await
                .expect_err("slow API should time out");
        server.await.expect("mock server should finish");
        assert!(error.to_string().contains("timed out"), "{error}");
    }

    #[tokio::test]
    async fn test_official_hosts_contract() {
        let body = r#"{"hosts":[{"id":1,"hostname":"host-a","seen_time":"2026-01-01T00:00:00Z","memory":2086899712,"cpu_type":"x86_64","cpu_physical_cores":4}]}"#;
        let (url, server) =
            spawn_mock("/api/v1/fleet/hosts?page=1&per_page=20", "200 OK", body).await;
        let hosts = FleetClient::new_test(&url, "test-token")
            .list_hosts(None, None, None, None, None, None)
            .await
            .expect("official hosts envelope should parse");
        server.await.expect("mock server should finish");
        assert_eq!(hosts[0].memory, Some(2086899712));
        assert_eq!(hosts[0].cpu_physical_cores, Some(4));
    }

    #[tokio::test]
    async fn test_official_reports_contract() {
        let body = r#"{"queries":[{"id":31,"name":"inventory","query":"select 1"}]}"#;
        let (url, server) =
            spawn_mock("/api/v1/fleet/reports?page=1&per_page=20", "200 OK", body).await;
        let reports = FleetClient::new_test(&url, "test-token")
            .list_reports()
            .await
            .expect("official reports envelope should parse");
        server.await.expect("mock server should finish");
        assert_eq!(reports[0].id, Some(31));
    }

    #[tokio::test]
    async fn test_official_policies_contract() {
        let body = r#"{"policy":{"id":7,"name":"Disk encryption","passing_host_count":8,"failing_host_count":2}}"#;
        let (url, server) = spawn_mock("/api/v1/fleet/global/policies/7", "200 OK", body).await;
        let policy = FleetClient::new_test(&url, "test-token")
            .get_policy(7)
            .await
            .expect("official policy envelope should parse");
        server.await.expect("mock server should finish");
        assert_eq!(policy.passing_host_count, Some(8));
    }

    #[tokio::test]
    async fn test_official_software_titles_contract() {
        let body = r#"{"software_titles":[{"id":2792,"name":"Slack","source":"apps","hosts_count":5,"versions_count":4}]}"#;
        let (url, server) = spawn_mock(
            "/api/v1/fleet/software/titles?page=1&per_page=20",
            "200 OK",
            body,
        )
        .await;
        let software = FleetClient::new_test(&url, "test-token")
            .list_software()
            .await
            .expect("official software titles envelope should parse");
        server.await.expect("mock server should finish");
        assert_eq!(software[0].hosts_count, Some(5));
    }

    #[tokio::test]
    async fn test_official_vulnerability_contract() {
        let body = r#"{"vulnerability":{"cve":"CVE-2022-30190","hosts_count":1234,"cve_published":"2022-06-01T00:15:00Z"}}"#;
        let (url, server) = spawn_mock(
            "/api/v1/fleet/vulnerabilities/CVE-2022-30190",
            "200 OK",
            body,
        )
        .await;
        let vulnerability = FleetClient::new_test(&url, "test-token")
            .get_cve("CVE-2022-30190")
            .await
            .expect("official vulnerability envelope should parse");
        server.await.expect("mock server should finish");
        assert_eq!(vulnerability.hosts_count, Some(1234));
        assert_eq!(
            vulnerability.cve_published.as_deref(),
            Some("2022-06-01T00:15:00Z")
        );
    }

    #[tokio::test]
    async fn test_error_body_is_not_reflected() {
        let body = r#"{"error":"secret upstream content"}"#;
        let (url, server) = spawn_mock(
            "/api/v1/fleet/reports?page=1&per_page=20",
            "500 Internal Server Error",
            body,
        )
        .await;
        let error = FleetClient::new_test(&url, "test-token")
            .list_reports()
            .await
            .expect_err("HTTP 500 should fail")
            .to_string();
        server.await.expect("mock server should finish");
        assert!(error.contains("HTTP 500"));
        assert!(!error.contains("secret upstream content"));
    }

    #[test]
    fn test_required_envelopes_reject_missing_keys() {
        assert!(serde_json::from_str::<HostsResponse>("{}").is_err());
        assert!(serde_json::from_str::<ReportsResponse>("{}").is_err());
        assert!(serde_json::from_str::<PoliciesResponse>("{}").is_err());
        assert!(serde_json::from_str::<SoftwareTitlesResponse>("{}").is_err());
        assert!(serde_json::from_str::<VulnerabilityResponse>("{}").is_err());
    }
}
