# Casial WASM

WASM bindings for universal consciousness-aware context coordination, bringing Casial's context management capabilities to web browsers and Node.js environments.

## Overview

Casial WASM provides WebAssembly bindings for [casial-core](../casial-core), enabling browser-based and Node.js applications to participate in consciousness-aware context coordination. This allows web applications to intelligently manage and coordinate context across different environments.

## Features

- **Browser & Node.js Compatible**: Runs in any modern JavaScript environment
- **Context Coordination**: Full access to Casial's context management capabilities
- **Memory Efficient**: Optimized WASM binary with `wee_alloc`
- **JavaScript Integration**: Seamless interop with JavaScript objects and APIs
- **Real-time Processing**: Async-capable context processing
- **TypeScript Support**: Generated TypeScript definitions for better DX

## Installation

### npm/yarn

```bash
npm install casial-wasm
# or
yarn add casial-wasm
```

### Cargo (for Rust projects)

```toml
[dependencies]
casial-wasm = "0.1.0"
```

## Usage

### Browser

```html
<script type="module">
  import init, { ContextCoordinator } from './pkg/casial_wasm.js';
  
  async function run() {
    await init();
    
    const coordinator = new ContextCoordinator();
    
    // Register context sources
    coordinator.register_source("user-input", {
      type: "interactive",
      priority: "high"
    });
    
    // Process context
    const result = await coordinator.process_context({
      content: "User wants to create a new document",
      source: "user-input",
      timestamp: Date.now()
    });
    
    console.log("Context processed:", result);
  }
  
  run();
</script>
```

### Node.js

```javascript
const { ContextCoordinator } = require('casial-wasm');

// Initialize the coordinator
const coordinator = new ContextCoordinator();

// Use with async/await
async function processUserAction(action) {
  const context = {
    content: action.description,
    source: "user-interface",
    metadata: action.metadata
  };
  
  return await coordinator.process_context(context);
}

// Use with promises
coordinator.query_context("recent interactions")
  .then(results => console.log("Query results:", results))
  .catch(err => console.error("Query failed:", err));
```

### TypeScript

```typescript
import init, { ContextCoordinator, ContextEntry } from 'casial-wasm';

interface UserAction {
  type: string;
  data: any;
  timestamp: number;
}

class ContextManager {
  private coordinator: ContextCoordinator;
  
  async initialize() {
    await init();
    this.coordinator = new ContextCoordinator();
  }
  
  async processAction(action: UserAction): Promise<ContextEntry[]> {
    return this.coordinator.process_context({
      content: JSON.stringify(action.data),
      source: `user-${action.type}`,
      timestamp: action.timestamp
    });
  }
}
```

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)

### Build

```bash
# Clone the repository
git clone https://github.com/prompted-llc/context-casial-xpress
cd context-casial-xpress/crates/casial-wasm

# Build for web
wasm-pack build --target web

# Build for Node.js
wasm-pack build --target nodejs

# Build for bundler (webpack, etc.)
wasm-pack build --target bundler
```

## API Reference

The WASM bindings expose the following key classes and methods:

- `ContextCoordinator`: Main coordination engine
- `ContextEntry`: Individual context items
- `ContextQuery`: Query interface for retrieving context
- `ContextSource`: Context source registration and management

See the [TypeScript definitions](./pkg/casial_wasm.d.ts) for complete API documentation.

## Performance

The WASM module is optimized for size and performance:

- Compiled with `-Os` optimizations
- Uses `wee_alloc` for minimal memory footprint
- Async-capable for non-blocking operations
- Efficient serialization with `serde-wasm-bindgen`

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Links

- [Repository](https://github.com/prompted-llc/context-casial-xpress)
- [Homepage](https://promptedllc.com)
- [Documentation](https://docs.rs/casial-wasm)
- [casial-core](https://crates.io/crates/casial-core) - Core coordination engine
- [WebAssembly](https://webassembly.org/) - Learn more about WASM