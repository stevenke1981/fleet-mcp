# Fleet MCP (Rust)

[![CI](https://github.com/stevenke1981/fleet-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/stevenke1981/fleet-mcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A read-only Model Context Protocol (MCP) server for [Fleet Device Management](https://fleetdm.com), implemented in Rust with the official `rmcp` SDK.

The server uses stdio transport and exposes 14 read-only tools for hosts, reports, policies, software, vulnerabilities, and Fleet server information. API routes and response envelopes are aligned with Fleet's current official REST API documentation.

## Build and install

Rust 1.85 or newer is required.

```text
git clone https://github.com/stevenke1981/fleet-mcp.git
cd fleet-mcp
cargo build --release --locked
```

The binary is written to `target/release/fleet-mcp` on Linux/macOS and `target/release/fleet-mcp.exe` on Windows.

GitHub Actions automatically checks every branch push and pull request, builds release binaries for Linux, Windows, and macOS, and publishes them as workflow artifacts. You can also start the workflow manually from the **Actions → CI → Run workflow** menu.

## Install a released version

Version tags matching `v*` start the [Release workflow](.github/workflows/release.yml). It verifies that the tag matches `Cargo.toml`, runs the quality gates, builds each supported runner, packages the binary with this README and the MIT license, and publishes SHA-256 checksums to a GitHub Release.

Download the archive for your operating system from [Releases](https://github.com/stevenke1981/fleet-mcp/releases), verify its `.sha256` file, extract it, and place the binary on your `PATH`. The archives are portable and do not require Rust.

For a source install, use:

```bash
cargo install --git https://github.com/stevenke1981/fleet-mcp --tag v0.1.0 --locked
```

To publish a new version, update `version` in `Cargo.toml`, commit and push the matching tag (for example `v0.2.0`). The release workflow also supports manually packaging an existing tag from GitHub Actions.

## Configuration

Set configuration through CLI arguments or environment variables. Use a Fleet API token with only the permissions needed for these read-only endpoints.

| CLI | Environment | Default | Description |
|---|---|---:|---|
| `--url` / `-u` | `FLEET_SERVER_URL` | required | Fleet server base URL |
| `--token` / `-t` | `FLEET_API_TOKEN` | required | Fleet API token |
| `--verify-ssl` | `FLEET_VERIFY_SSL` | `true` | Verify TLS certificates |
| `--timeout` | `FLEET_TIMEOUT` | `30` | Request timeout in seconds |

HTTPS is required except for loopback development URLs. URLs containing credentials, query strings, or fragments are rejected. The process does not load `.env` files itself; `.env.example` is a template for shells or launchers that support environment files.

PowerShell:

```powershell
$env:FLEET_SERVER_URL = 'https://fleet.example.com'
$env:FLEET_API_TOKEN = 'replace-with-a-read-only-token'
.\target\release\fleet-mcp.exe
```

Bash:

```bash
export FLEET_SERVER_URL='https://fleet.example.com'
export FLEET_API_TOKEN='replace-with-a-read-only-token'
./target/release/fleet-mcp
```

## Tools

| Domain | Tools |
|---|---|
| Hosts | `fleet_list_hosts`, `fleet_get_host`, `fleet_search_hosts`, `fleet_get_host_by_identifier` |
| Reports | `fleet_list_reports`, `fleet_get_report`, `fleet_get_report_data` |
| Policies | `fleet_list_policies`, `fleet_get_policy` |
| Software and CVEs | `fleet_list_software`, `fleet_list_vulnerabilities`, `fleet_get_cve` |
| Server | `fleet_get_config`, `fleet_get_version` |

The project intentionally does not expose write operations, live SQL execution, or undocumented osquery-schema routes.

## MCP client example

```json
{
  "mcpServers": {
    "fleet": {
      "command": "C:\\path\\to\\fleet-mcp.exe",
      "env": {
        "FLEET_SERVER_URL": "https://fleet.example.com",
        "FLEET_API_TOKEN": "replace-with-a-read-only-token"
      }
    }
  }
}
```

Never commit a real token. Prefer the client or operating system's secret-management facility when available.

## For AI agents and maintainers

Read [AGENTS.md](AGENTS.md) before editing. It contains the compact repository contract, canonical commands, source map, API invariants, and commit attribution rule.

The normal agent loop is: inspect the relevant symbol, make the smallest focused patch, run the narrowest meaningful Cargo checks, then run the full CI-equivalent commands before committing. Public tool names and Fleet API routes are part of the compatibility surface; update `updates.md`, `README.md`, `test.md`, and `spec.md` when changing them. Keep this server read-only, never log or persist API tokens, and do not add undocumented Fleet routes based on guesses.

Useful local commands:

```text
cargo fmt --all --check
cargo check --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --no-deps --all-features
cargo build --release --locked
```

For GitHub status, use `gh run list --repo stevenke1981/fleet-mcp` and inspect failed jobs with `gh run view <run-id> --log-failed`.

## Verification

```text
cargo fmt --all --check
cargo check --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --no-deps --all-features
cargo build --release --locked
```

Contract tests use a local mock HTTP server and verify official Fleet routes, required response envelopes, Bearer authentication, Host payload shape, and that upstream error bodies are not reflected into model-facing output.

## Status and scope

This is version `0.1.0`. See [updates.md](updates.md) for the completed hardening work and the remaining non-blocking roadmap. A real Fleet deployment is still required for environment-specific compatibility testing.

## License

[MIT](LICENSE)
