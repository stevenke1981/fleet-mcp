# Fleet MCP 待辦事項

## v0.1.0 交付

- [x] Rust stdio MCP server 與 14 個 read-only tools。
- [x] Hosts、Reports、Policies、Software、Vulnerabilities 官方 API routes/envelopes。
- [x] HTTP contract tests 與設定安全驗證。
- [x] Token redaction、上游 error-body 隔離。
- [x] CI：format、check、clippy、test、docs、package dry-run、三平台 release build。
- [x] MIT license、Cargo.lock、README、環境與 MCP client 範例。
- [ ] 真實 Fleet instance 相容性驗證（需要外部 Fleet URL/token）。

## 後續版本

- [x] 將上游失敗映射為 MCP `is_error` 結果並加入 transport integration tests。
- [x] 為所有 list tools 提供一致分頁與 response-size 上限。
- [x] 加入 read-only MCP annotations、GET-only client、CVE path validation、敏感欄位摘要與工具白名單。
- [x] 提供 Claude Desktop/Cursor 設定合併安裝器（預設不寫入 token）。
- [ ] 增加 cargo-deny/cargo-audit、Dependabot 與 release checksums/provenance。
- [ ] 依官方文件與真實環境驗證後，評估更多 read-only endpoints。
- [ ] 另行設計與審查任何 live report 或寫入操作；不在 v0.1.0 暗示支援。
