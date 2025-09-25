# MOP Repository Guide

Welcome to the Meta-Orchestration Protocol (MOP) monorepo. This workspace houses the Rust
crates that implement the consciousness-aware MCP proxy/runtime alongside the WASM bindings,
benchmarks, docs, and deployment assets. Treat this document as the ground-truth orientation
when making changes.

## Layout
- `crates/casial-core` – paradox detection & core coordination logic (MIT/Apache-2.0).
- `crates/casial-server` – Axum-powered runtime exposing WebSocket + HTTP/SSE MCP endpoints,
  federation client, pitfall shim, metrics, and debug surfaces (Fair Use license).
- `crates/casial-wasm` – browser-friendly bindings for the orchestration runtime (Fair Use).
- `bench/` – k6 load scripts (WS connect storm, HTTP tools/call RPS, SSE soak).
- `docs/` – operator guides, integration notes, sampling contract, whitepaper.
- `missions/` – orchestration mission templates (e.g., Exa MCP playbooks).
- `examples/`, `scripts/`, `smithery.*` – developer tooling and integration configs.

## Coding Conventions
- Rust edition 2021. Always run `cargo fmt` before committing.
- Keep `clippy::all` clean where practical; new warnings introduced by a change should either be
  addressed or justified in the PR description.
- `casial-server` uses async Axum 0.7 – middleware signatures must use `axum::extract::Request`
  (type alias) and `Next` without generics.
- CORS is centralized via helpers in `http_mcp.rs`. Do not hand-roll headers elsewhere; use
  `build_cors_layer()` or `apply_cors_headers()` to ensure parity across MCP routes.
- Sampling features are guarded by the `MOP_ENABLE_SAMPLING` environment variable. When adding
  new capabilities, ensure capability manifests and docs reflect flag behavior.
- Federation backoff/circuit-breaker logic lives in `federation.rs`. New downstream handling
  should reuse the shared helpers (e.g., `compute_backoff_duration`, `record_failure_shared`).

## Security & Operations
- `/debug/*` routes require `MOP_ADMIN_TOKEN`; maintain this middleware when introducing new
  debug surfaces.
- API auth uses the Mop-Api-Key header or Smithery query param. All example configs should use
  `${MOP_API_KEY:-DEMO_KEY_PUBLIC}` to avoid leaking real secrets.
- When modifying CORS or auth flows, update `test_mop_curl_commands.sh` and the Smithery
  compatibility script accordingly.

## Testing & Tooling
Run these before submitting changes that touch Rust code:

```bash
cargo fmt
cargo test -p casial-server
```

Integration smoke tests (optional but recommended when touching transports):

```bash
bash ./test_mop_curl_commands.sh
bash ./test-smithery-compatibility.sh
```

For performance claims, keep `bench/` scripts in sync with documentation and update
`BENCHMARKS.md` when methodology changes.

## Documentation
- Update `docs/README.md` and relevant integration guides when altering runtime behavior.
- Sampling contract details belong in `docs/SAMPLING_CONTRACT.md`; reference it from other docs
  instead of duplicating instructions.
- Licenses: core crates remain dual MIT/Apache-2.0, server/WASM carry Fair Use banners – ensure
  new files reference the correct license in headers or module docs.

## Pull Request Expectations
- Describe security-impacting changes (auth, CORS, federation routing) explicitly.
- Include relevant benchmark updates when adjusting performance-sensitive code.
- Keep Railway/Docker instructions accurate if startup flags or ports change.

Happy orchestrating!
