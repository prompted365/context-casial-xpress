# Casial Server

High-performance WebSocket MCP (Model Context Protocol) server with consciousness-aware context coordination.

## Overview

Casial Server provides a WebSocket-based MCP server implementation that enables AI agents and tools to coordinate context with consciousness-aware intelligence. It's built on top of [casial-core](../casial-core) and provides real-time context sharing capabilities.

## Features

- **WebSocket MCP Server**: Full MCP compliance with WebSocket transport
- **Real-time Context Coordination**: Live context sharing between connected clients
- **Consciousness-aware Processing**: Intelligent context prioritization and management
- **High Performance**: Built with Tokio for async performance
- **Observability**: Built-in metrics and tracing support
- **File System Monitoring**: Automatic context updates based on file changes

## Installation & Usage

### As a Binary

Install and run the server:

```bash
cargo install casial-server
casial-server --help
```

### Configuration

The server can be configured via YAML, JSON, or environment variables:

```yaml
# casial-server.yaml
host: "127.0.0.1"
port: 8080
max_connections: 1000
enable_metrics: true
```

Run with configuration:

```bash
casial-server --config casial-server.yaml
```

### Development

```bash
# Clone the repository
git clone https://github.com/prompted-llc/context-casial-xpress
cd context-casial-xpress

# Run the server in development mode
cargo run -p casial-server -- --help
```

## API

The server implements the Model Context Protocol (MCP) over WebSocket. Connected clients can:

- Register context sources
- Subscribe to context updates
- Send context queries
- Receive real-time context notifications

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Links

- [Repository](https://github.com/prompted-llc/context-casial-xpress)
- [Homepage](https://promptedllc.com)
- [Documentation](https://docs.rs/casial-server)
- [casial-core](https://crates.io/crates/casial-core) - Core coordination engine