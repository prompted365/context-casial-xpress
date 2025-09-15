# Action Items for Publishing Casial Crates

## Pre-Publishing Checklist

### 1. Manifest Metadata Validation
- [ ] **name**: Verify unique names on crates.io
- [ ] **version**: Use `version.workspace = true` for inheritance  
- [ ] **authors**: Set to workspace default
- [ ] **license**: Confirm matches existing LICENSE files (`MIT OR Apache-2.0`)
- [ ] **description**: Write concise, descriptive summaries
- [ ] **readme**: Point to `README.md` in each crate directory
- [ ] **repository**: Set to GitHub repo URL
- [ ] **homepage**: Set to project homepage
- [ ] **keywords**: Add 3-5 relevant keywords per crate
- [ ] **categories**: Choose appropriate categories from crates.io list

### 2. Per-Crate Categories and Keywords
- [ ] **casial-core**: `["development-tools", "api-bindings"]` + AI/context keywords
- [ ] **casial-server**: `["web-programming", "development-tools"]` + server/API keywords  
- [ ] **casial-wasm**: `["wasm", "web-programming"]` + WASM/browser keywords
- [ ] **casial-cli**: `["command-line-utilities"]` + CLI/tools keywords

### 3. README.md Files
- [ ] Create `crates/casial-core/README.md` with:
  - [ ] Library overview and core concepts
  - [ ] Usage examples and API documentation
  - [ ] Installation via `cargo add casial-core`
- [ ] Create `crates/casial-server/README.md` with:
  - [ ] Server component overview
  - [ ] Configuration and deployment examples  
  - [ ] Installation and setup instructions
- [ ] Create `crates/casial-wasm/README.md` with:
  - [ ] WASM bindings overview
  - [ ] Browser/Node.js usage examples
  - [ ] Build and integration instructions
- [ ] Create `crates/casial-cli/README.md` with:
  - [ ] CLI tool overview and features
  - [ ] Command examples and usage
  - [ ] Installation via `cargo install casial-cli`

### 4. License Verification
- [ ] Confirm LICENSE-APACHE and LICENSE-MIT files exist
- [ ] Verify license field matches file contents
- [ ] Check if mixed licensing is properly documented

### 5. Workspace Dependencies
- [ ] Ensure path dependencies between workspace members have versions
- [ ] Verify `[workspace.dependencies]` is properly configured  
- [ ] Test that `cargo publish --dry-run` resolves dependencies correctly

### 6. Code Quality
- [ ] Run `cargo fmt --all` to format code
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Run `cargo test --workspace --all-features` 
- [ ] Run `cargo build --workspace --release`

### 7. Publishing Order
- [ ] Determine dependency graph (likely: core → server/wasm → cli)
- [ ] Write final publish order to `publish/publish_order.txt`
- [ ] Verify no circular dependencies

## Publishing Process

### 8. Name Availability Check
- [ ] Check `casial-core` availability: `curl -s https://crates.io/api/v1/crates/casial-core`
- [ ] Check `casial-server` availability: `curl -s https://crates.io/api/v1/crates/casial-server`
- [ ] Check `casial-wasm` availability: `curl -s https://crates.io/api/v1/crates/casial-wasm`
- [ ] Check `casial-cli` availability: `curl -s https://crates.io/api/v1/crates/casial-cli`

### 9. Dry Run Testing
- [ ] `cargo publish -p casial-core --dry-run`
- [ ] `cargo publish -p casial-server --dry-run`  
- [ ] `cargo publish -p casial-wasm --dry-run`
- [ ] `cargo publish -p casial-cli --dry-run`
- [ ] Fix any errors before proceeding

### 10. Actual Publishing
- [ ] `cargo publish -p casial-core` 
- [ ] Wait 60-120 seconds for index update
- [ ] `cargo publish -p casial-server`
- [ ] Wait 60-120 seconds for index update  
- [ ] `cargo publish -p casial-wasm`
- [ ] Wait 60-120 seconds for index update
- [ ] `cargo publish -p casial-cli`

### 11. Post-Publishing Verification
- [ ] Verify https://crates.io/crates/casial-core is live
- [ ] Verify https://crates.io/crates/casial-server is live
- [ ] Verify https://crates.io/crates/casial-wasm is live  
- [ ] Verify https://crates.io/crates/casial-cli is live
- [ ] Test installation: `cargo install casial-cli`
- [ ] Check docs.rs builds are successful

### 12. Release Management
- [ ] Create git tag for release version (e.g., `v0.1.0`)
- [ ] Push tag to GitHub: `git push --tags`
- [ ] Create GitHub release with links to published crates
- [ ] Update repository README.md with installation instructions

## Future Enhancements

### 13. CLI Improvements (Optional)
- [ ] Add shell completions using `clap_complete`
- [ ] Generate man pages using `clap_mangen`
- [ ] Implement `build.rs` for compile-time doc generation
- [ ] Consider adding progress bars for long operations
- [ ] Add colored output for better UX

### 14. Binary Distribution (Optional)
- [ ] Set up GitHub Actions for cross-platform builds
- [ ] Create releases for major platforms (Linux, macOS, Windows)
- [ ] Consider package manager submissions (Homebrew, etc.)

## Notes
- All crates should use the same version for consistency
- Monitor first few downloads to catch any immediate issues
- Consider setting up docs.rs documentation links
- Keep `CHANGELOG.md` updated for future releases