# Fleet MCP 實作計畫

1. 以 Fleet 官方 REST API 校準 read-only endpoints 與 response envelopes。
2. 加固 token、URL、timeout 與上游錯誤處理。
3. 以 loopback HTTP contract tests 固定 routes、headers 與 payload shapes。
4. 同步 CLI、README、spec、test plan、todo 與 CI。
5. 通過 format/check/clippy/test/docs/release/package gates。
6. 初始化 Git、建立 `stevenke1981/fleet-mcp`、推送並確認 remote SHA parity。

詳細 findings、完成狀態與後續項目以 `updates.md` 為準。
