# Crate Publish Verification Report v0.1.3

**Generated**: 2025-09-15 02:07:30 UTC
**Workspace Version**: 0.1.3
**Repository**: https://github.com/prompted365/context-casial-xpress

## Summary

All three crates have been successfully published to crates.io with version 0.1.3. This version includes **corrected licensing** - casial-core uses dual MIT/Apache-2.0 licensing for maximum adoptability, while casial-server and casial-wasm use the Fair Use license.

## Published Crates

### casial-core v0.1.3

- **Registry**: crates.io
- **Crate ID**: 1729885
- **Published**: 2025-09-15 02:06:18 UTC
- **Size**: 27,841 bytes
- **License**: MIT OR Apache-2.0 ✅
- **Repository**: https://github.com/prompted365/context-casial-xpress
- **Dependencies**: 15 direct dependencies
- **Features**: default (std), std, wasm (js-sys, web-sys)
- **Download URL**: https://crates.io/api/v1/crates/casial-core/0.1.3/download

**Description**: Consciousness-aware context coordination engine - The Casial substrate for paradox-resilient context management

### casial-server v0.1.3

- **Registry**: crates.io
- **Crate ID**: 1729886
- **Published**: 2025-09-15 02:06:53 UTC
- **Size**: 37,997 bytes
- **License**: non-standard (Fair Use) ✅
- **Repository**: https://github.com/prompted365/context-casial-xpress
- **Dependencies**: 26 direct dependencies
- **Binary**: casial-server
- **Download URL**: https://crates.io/api/v1/crates/casial-server/0.1.3/download

**Description**: High-performance WebSocket MCP server with consciousness-aware context coordination

### casial-wasm v0.1.3

- **Registry**: crates.io
- **Crate ID**: 1729887
- **Published**: 2025-09-15 02:07:12 UTC
- **Size**: 14,364 bytes
- **License**: non-standard (Fair Use) ✅
- **Repository**: https://github.com/prompted365/context-casial-xpress
- **Dependencies**: 10 direct dependencies
- **Crate Type**: cdylib
- **Features**: default (console_error_panic_hook)
- **Download URL**: https://crates.io/api/v1/crates/casial-wasm/0.1.3/download

**Description**: WASM bindings for universal consciousness-aware context coordination

## Verification Steps Completed

1. ✅ Workspace version bumped to 0.1.3
2. ✅ Repository URLs corrected to `prompted365` organization
3. ✅ **Licensing corrected**:
   - casial-core: MIT OR Apache-2.0 (for maximum adoptability)
   - casial-server & casial-wasm: Fair Use license via `license-file = "../../LICENSE.md"`
4. ✅ All crates compile successfully
5. ✅ casial-core published successfully
6. ✅ casial-server published successfully (depends on casial-core 0.1.3)
7. ✅ casial-wasm published successfully (depends on casial-core 0.1.3)
8. ✅ All crates available on crates.io registry
9. ✅ Git repository tagged with v0.1.3
10. ✅ Changes pushed to GitHub repository

## Licensing Model

### Mixed Licensing Strategy
- **casial-core**: Dual MIT/Apache-2.0 license for maximum ecosystem adoptability
- **casial-server & casial-wasm**: Fair Use license allowing research/evaluation but requiring commercial licensing

### License Files Present
- `LICENSE-MIT`: MIT License for casial-core
- `LICENSE-APACHE`: Apache 2.0 License for casial-core  
- `LICENSE.md`: Fair Use License for casial-server and casial-wasm
- `NOTICE`: Attribution notice for Apache-licensed components

## Installation & Usage

### For Research/Educational Use (Free)
```bash
# Core library (MIT/Apache-2.0)
cargo add casial-core

# Server and WASM bindings (Fair Use - free for research/education)
cargo add casial-wasm
cargo install casial-server
```

### For Commercial Use
Contact Prompted LLC at breyden@prompted.community for commercial licensing of casial-server and casial-wasm.

## Notes

- **🔧 Licensing Fix**: Previous versions incorrectly had all crates under MIT/Apache-2.0. v0.1.3 correctly implements the intended mixed licensing model.
- **📁 Repository URL**: Correctly points to `prompted365` organization
- **🔗 Dependency Chain**: casial-server and casial-wasm both depend on casial-core, requiring publication in the correct order.
- **⚙️ Workspace Management**: All crates inherit version and metadata from the workspace configuration.
- **📜 Fair Use License**: casial-server and casial-wasm now properly show as "non-standard" license on crates.io, referencing the Fair Use license file.
- **🏢 Ubiquity OS**: Part of the broader Ubiquity OS ecosystem with board-governed licensing framework

## Status: ✅ COMPLETE

All crates successfully published with **correct licensing** and repository information.

---

**Contact**: breyden@prompted.community  
**Website**: https://promptedllc.com  
**Part of**: Ubiquity OS ecosystem