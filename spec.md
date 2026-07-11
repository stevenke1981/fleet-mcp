# Fleet MCP v0.1.0 規格

## 範圍

Rust 2024 Edition、Rust 1.85+、`rmcp` stdio transport、`reqwest` Fleet REST client。只提供 read-only tools，不執行 live SQL 或 Fleet 寫入操作。

## 模組

```text
src/main.rs          CLI、logging、stdio lifecycle
src/config.rs        CLI/env parsing 與安全驗證
src/handler.rs       MCP tool router（14 tools）
src/fleet/client.rs  Fleet REST client 與 contract tests
src/fleet/types.rs   Fleet API response types
```

## Tool domains

- Hosts：list/get/search/get-by-identifier（4）
- Reports：list/get/get-data（3）
- Policies：list/get（2）
- Software/Vulnerabilities：list software titles/list CVEs/get CVE（3）
- Fleet server：config/version（2）

## Security constraints

- API token 僅放入 HTTP Authorization header，不另存於 `FleetClient`，Debug 一律 redacted。
- HTTPS required；HTTP only for loopback development。
- URL 禁止 userinfo、query、fragment；timeout 必須大於零。
- 上游 HTTP error body 不回傳給 MCP client/model context。
- Endpoint envelope 缺少 required key 時視為契約錯誤，不當成空陣列。

## Out of scope

Write tools、live reports、undocumented osquery schema routes、TOML config、HTTP/SSE transport、metrics。後續加入前必須有官方契約、測試與安全審查。
