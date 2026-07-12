# Fleet MCP 驗證計畫

## 自動化品質閘門

```text
cargo fmt --all --check
cargo check --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --no-deps --all-features
cargo build --release --locked
cargo publish --dry-run --locked
```

## Release workflow

Pushing a tag such as `v0.1.0` starts `.github/workflows/release.yml`. It validates the tag/version match, reruns format/lint/test/package gates, builds Linux/Windows/macOS archives, creates SHA-256 files, and uploads them to a GitHub Release. A tag must be pushed only after the matching `Cargo.toml` version is committed.

執行文件檢查時設定 `RUSTDOCFLAGS=-Dwarnings`。目前單元與 HTTP contract tests 位於 `src/fleet/client.rs`，使用 loopback TCP mock 驗證：

- Fleet 官方 hosts/reports/policies/software/vulnerabilities routes。
- 必要 response envelope 不可缺少。
- Host memory/CPU 欄位與 Fleet 官方 payload 相容。
- Bearer header 存在，但 Debug 輸出不洩漏 token。
- HTTP error body 不會反射到 MCP/model-facing 錯誤內容。
- URL、token 與 timeout 設定邊界。

## Stdio smoke test

使用有效格式但不需要可連線的 Fleet URL 啟動 binary，透過 MCP stdio 依序傳送 `initialize`、`notifications/initialized`、`tools/list`；工具列舉不會呼叫 Fleet API。預期列出 14 個 read-only tools。

## 尚需真實環境驗證

- Fleet current release 與實際 RBAC token。
- Premium / non-Premium response 差異。
- 大型 inventory 的分頁與輸出上限。
- Claude Desktop、Codex、Cursor 等實際 MCP clients。
