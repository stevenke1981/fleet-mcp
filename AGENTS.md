# Agent Instructions

## Package Manager

- Use Cargo with the committed `Cargo.lock`.
- Rust MSRV: 1.85; edition: 2024.

## File-Scoped Commands

| Task | Command |
|---|---|
| Format Rust | `cargo fmt --all --check` |
| Type/check | `cargo check --locked --all-targets --all-features` |
| Lint | `cargo clippy --locked --all-targets --all-features -- -D warnings` |
| Tests | `cargo test --locked --all-targets --all-features` |
| Filter tests | `cargo test --locked fleet::client::tests::NAME` |
| Docs | `$env:RUSTDOCFLAGS='-D warnings'; cargo doc --locked --no-deps --all-features` |
| Release build | `cargo build --release --locked` |

## Source Map

- `src/config.rs`: CLI/env parsing and URL/token/timeout validation.
- `src/fleet/client.rs`: Fleet REST calls and HTTP contract tests.
- `src/fleet/types.rs`: response envelopes and Fleet payload types.
- `src/handler.rs`: MCP tool router; public tool names are compatibility API.
- `.github/workflows/ci.yml`: automatic push/PR/manual CI and release artifacts.

## Key Conventions

- Keep the server read-only. Do not add write or live-SQL tools without a separate security design.
- Use only documented Fleet REST routes and required response envelopes; add a contract test for each new route.
- Never store, print, or include API tokens in Debug output or errors.
- Do not reflect untrusted upstream error bodies into model-facing responses.
- Preserve the 14-tool public names unless the README, spec, tests, and migration notes are updated together.
- Keep `.codebase-memory/`, `.opencode/`, `target/`, and credentials out of Git.
- Use `apply_patch` for focused edits; preserve unrelated worktree changes.

## Commit Attribution

AI commits MUST include a `Co-Authored-By` trailer identifying the agent model, for example:

```text
Co-Authored-By: Codex <noreply@openai.com>
```
