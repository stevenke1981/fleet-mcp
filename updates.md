# Fleet MCP 改善與實作追蹤

> 審查日期：2026-07-12
> 依據：CBM 程式知識圖譜、Rust 靜態/測試驗證、Fleet 官方 REST API 文件

## 目標

把目前可編譯但多處 API 契約不正確的原型，修成具備可信任錯誤語意、官方 Fleet API 相容性、基本安全防護與可公開交付品質的 Rust MCP server。

## P0 — 本次必須完成

- [x] 修正 Host schema：`memory` 改為位元組整數，補上官方扁平 CPU/磁碟/seen-time 欄位。
- [x] 將舊 Queries 路由改為現行 Reports 路由（`/reports`、`/reports/:id`、`/reports/:id/report`），同步工具名稱與型別。
- [x] 修正 Policies 路由為 `/global/policies`，把錯誤的「policy results」工具改為取得單一 policy。
- [x] 修正 Software 路由為 `/software/titles` 並解析 `software_titles` envelope。
- [x] 修正 CVE detail 的 `vulnerability` envelope，以及 `hosts_count`、`cve_published` 等官方欄位。
- [x] 用 endpoint-specific response envelope 取代 client 的巨型全 Optional envelope；缺少必要 key 時必須回錯，不能靜默變成空陣列。
- [x] 新增官方格式 fixture/HTTP contract tests，驗證 route、query encoding、Bearer header、成功回應與錯誤回應。

## P1 — 本次完成的安全與可靠性改善

- [x] `FleetClient` 不再保存 plaintext API token，Debug 輸出不得洩漏 token。
- [x] 上游 HTTP 錯誤不再把未信任、無上限 response body 原樣送進 MCP/model context。
- [x] URL 僅允許 HTTP(S)，拒絕 userinfo/query/fragment；timeout 必須大於 0；HTTP 僅允許 loopback，避免 Bearer token 明文外送。
- [x] 移除未實作的 `--config` 與無效 security-mode 介面/宣稱，讓 CLI、README 與實際能力一致。
- [x] 修正 rustfmt、rustdoc、clippy、test、release build 品質閘門。
- [x] 增加 `.env.example`、MIT `LICENSE`、正確 `.gitignore`、Cargo.lock 追蹤與 GitHub Actions。
- [x] 修正 repository URL、CI badge、安裝與設定文件，避免指向無關的既有 Python 專案。
- [x] CI 在所有 branch push/PR 自動編譯；version tag 自動打包 Linux、Windows、macOS archives、SHA-256 checksums 並建立 GitHub Release。

## P2 — 後續增強（不阻擋 v0.1.0）

- [x] MCP stdio acceptance：initialize、tools/list；上游/serialization failure 使用 MCP `is_error` result。
- [ ] 所有 list tool 加入一致的 page/per_page/meta，並限制單次回應大小。
- [x] 將 handler 的上游失敗改成正式 MCP `is_error` result，而非成功文字內容。
- [ ] 以真實 Fleet instance 驗證 config/version；health 與 osquery schema 等未被官方 REST 文件確認的工具已自 v0.1.0 移除。
- [ ] 加入 dependency/license/security audit（cargo-deny 或 cargo-audit）與 release provenance。
- [ ] 評估 Streamable HTTP transport、寫入操作與 SELECT live report；實作前維持明確 read-only 範圍。

## 驗收標準

- `cargo fmt --all --check`
- `cargo check --all-targets --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`
- `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features`
- `cargo build --release --all-features`
- 本機 stdio MCP initialize + tools/list smoke test
- GitHub Actions 綠燈；本機 commit 與遠端 `main` SHA 完全一致
