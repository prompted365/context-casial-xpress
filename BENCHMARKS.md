# BENCHMARKS

This directory contains repeatable load and soak tests for the
Meta-Orchestration Protocol (MOP) server. The scripts are written for
[k6](https://k6.io/) and exercise the primary transports and tool
execution paths.

> **Note:** All scripts assume the public demo API key. Set the
> `MOP_API_KEY` environment variable to point at a private deployment.

## Prerequisites

1. Install k6 (`brew install k6` or download from k6.io).
2. Export the target endpoint variables:
   ```bash
   export MOP_HTTP_BASE="http://localhost:8000/mcp"
   export MOP_BASE_URL="ws://localhost:8000/ws"
   export MOP_API_KEY="${MOP_API_KEY:-DEMO_KEY_PUBLIC}"
   ```
3. Run the server in release mode for accurate results:
   ```bash
   RUST_LOG=info cargo run -p casial-server --release -- start
   ```

## Scripts

| Script | Description | Key Metrics |
| ------ | ----------- | ----------- |
| `bench/ws_connect_storm.js` | Opens and closes a large number of WebSocket MCP sessions to validate connection churn resilience. | Failure rate (`ws_failures`), connection errors | 
| `bench/tools_call_rps.js` | Measures sustained `tools/list` throughput over HTTP after repeated session initialization. | Failure rate (`tools_call_failures`), response status |
| `bench/sse_keep_alive.js` | Verifies Server-Sent Events keep-alive behaviour by requesting the HTTP transport and measuring first byte latency. | Failure rate (`sse_failures`), `sse_first_byte_latency` |

## Running the tests

Each script can be executed with k6. Example:

```bash
k6 run bench/ws_connect_storm.js \
  --vus 100 \
  --duration 2m \
  -e MOP_BASE_URL="ws://localhost:8000/ws"
```

Override the environment variables to point at staging or production
endpoints. The scripts read the following overrides:

- `MOP_BASE_URL` – WebSocket endpoint (defaults to `ws://localhost:8000/ws`).
- `MOP_HTTP_BASE` – HTTP/SSE endpoint (defaults to `http://localhost:8000/mcp`).
- `MOP_API_KEY` – API key used for authentication.
- Duration, VU, and pacing parameters (see each script for the
  corresponding `MOP_*` environment variable).

## Reporting

After each run k6 prints latency and error metrics. Capture the output
and store alongside build artifacts for reproducibility. The trend and
rate metrics exported by the scripts can also be scraped by k6 Cloud or
any TSDB-compatible sink.

For formal releases include:

1. The git commit hash of the server.
2. Command lines used for each scenario.
3. Peak CPU / memory observations (e.g., from `docker stats` or `top`).
4. A summary table with throughput, error rate, and P95/P99 latencies.

## Extending the suite

- Add scenarios for federated tool calls once downstream MCP servers are
  available in staging environments.
- Parameterise mission loading via `MOP_MISSION_PATH` to benchmark complex
  orchestration profiles.
- Export Prometheus metrics during runs to correlate transport-level and
  application-level observations.
