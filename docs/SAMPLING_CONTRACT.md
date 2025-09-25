# MOP Sampling Contract

The Meta-Orchestration Protocol (MOP) server does not ship with a built-in
language model. Sampling requests must be handled by the connected MCP
client. This document describes the contract between the server and the
client when sampling capabilities are advertised.

## Feature flag

Sampling support is controlled by the `MOP_ENABLE_SAMPLING` environment
variable.

| Value | Behaviour |
| ----- | --------- |
| unset / `0` / `false` | Sampling capability is disabled. The server will not advertise sampling support and the `sampling/createMessage` method returns an error indicating that sampling is disabled. |
| `1` / `true` / `yes` | Sampling capability metadata is advertised. The server still expects the client to provide an LLM and will return guidance if the client attempts to delegate sampling back to the server. |

## Client responsibilities

When sampling is enabled the client **must**:

1. **Provide an LLM.** The client is responsible for executing sampling
   requests and returning the generated message payloads to the server.
2. **Surface capability flags.** Tool UIs should hide sampling-dependent
   interactions when the capability metadata indicates that sampling is
   disabled.
3. **Propagate the `samplingEnabled` metadata.** When calling
   `sampling/createMessage`, the client should include any context or
   metadata that the downstream LLM requires to fulfil the request.

## Server responses

* If sampling is **disabled**, the server returns the following error:
  ```json
  {
    "code": -32001,
    "message": "Sampling disabled by server configuration",
    "data": {
      "featureFlag": "MOP_ENABLE_SAMPLING",
      "enabled": false
    }
  }
  ```
* If sampling is **enabled**, the server returns an error that explicitly
  notes the client must execute the request. The `data` payload includes
  the messages and optional system prompt so the client can hand them to
  its LLM.

## Advertising capabilities

The server exposes sampling state in multiple places:

* `initialize` response: `capabilities.sampling` contains either `{ "enabled": false, ... }`
  or metadata describing that sampling is client-side.
* `/.well-known/mcp-config`: includes `featureFlags.samplingEnabled` so that
  configuration tooling can show or hide sampling related UI.

Clients should consult these fields during startup and adjust their UI and
request routing accordingly.
