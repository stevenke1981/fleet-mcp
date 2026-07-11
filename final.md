# Fleet MCP v0.1.0 交付報告

> 日期：2026-07-12
> 實作：Rust 2024 Edition / rmcp 0.16 / stdio MCP

## 完成內容

- 將原型的 Fleet API 契約校準至官方 current REST API：Hosts、Reports、Policies、Software titles、Vulnerabilities。
- 修正 Host `memory`/CPU/disk/seen-time schema、CVE envelope/欄名、required response envelope。
- 將已更名的 Queries tools/routes 改為 Reports；移除未經官方契約確認的 health/osquery-schema tools。
- 對外提供 14 個明確 read-only MCP tools。
- API token 不保存在 `FleetClient`，Debug redacted；HTTP error body 不反射至模型內容。
- URL、credentials/query/fragment、HTTP loopback 與 timeout 邊界驗證。
- Handler upstream/serialization failures 會形成 MCP `is_error` result。
- 加入 loopback HTTP contract tests、stdio MCP acceptance、CI、MIT license、Cargo.lock、package allowlist 與完整文件。

## 驗收結果

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | PASS |
| `cargo check --locked --all-targets --all-features` | PASS |
| `cargo clippy ... -- -D warnings` | PASS |
| `cargo test --locked --all-targets --all-features` | PASS（30 tests） |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --locked --no-deps --all-features` | PASS |
| `cargo publish --dry-run --locked` | PASS（12 files） |
| `cargo build --release --locked` | PASS |
| MCP stdio initialize + tools/list | PASS（protocol 2025-03-26，14 tools） |

## Remaining external validation

缺少可用的真實 Fleet server URL/token，因此尚未對特定 Fleet deployment、RBAC 與 Premium response differences 做 live acceptance。這項工作保留在 `updates.md`，不影響本機契約、編譯、測試、stdio 與 package gates。

## Durable follow-up

後續優先處理一致分頁/輸出大小上限、真實 Fleet compatibility matrix、dependency/license audit 與 release provenance。任何 live query 或寫入操作都需另行安全設計，不會從 v0.1.0 read-only scope 默認擴張。
