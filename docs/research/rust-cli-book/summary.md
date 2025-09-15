# Rust CLI Book Research Summary

Based on crawling the Rust CLI book tutorial pages, here are the key findings for publishing CLI crates to crates.io:

## Publishing to crates.io

### Required Steps
1. Create account on crates.io (linked to GitHub)
2. Generate token from https://crates.io/me
3. Run `cargo login <token>` (once per machine)
4. Add required metadata to Cargo.toml
5. Run `cargo publish`

### Required Manifest Metadata

According to the tutorial, the essential fields for crates.io publishing are:

```toml
[package]
name = "grrs"
version = "0.1.0" 
authors = ["Your Name <email@example.com>"]
license = "MIT OR Apache-2.0"
description = "A tool to search files"
readme = "README.md"
homepage = "https://github.com/you/grrs"
repository = "https://github.com/you/grrs"
keywords = ["cli", "search", "demo"]
categories = ["command-line-utilities"]
```

### License Recommendations
- Use `"MIT OR Apache-2.0"` for maximum compatibility
- Must match actual LICENSE files in repository
- SPDX license expressions required

### Categories for Different Crate Types
- CLI tools: `["command-line-utilities"]`
- Libraries: choose from appropriate categories
- WASM: consider `["wasm"]` if applicable
- Web/server: `["web-programming"]`, `["development-tools"]`

## CLI-Specific Best Practices

### Command Line Argument Parsing
- Use `clap` with derive feature for structured argument parsing
- Document CLI args with doc comments (becomes help text)
- Example structure:
  ```rust
  use clap::Parser;
  
  /// Search for a pattern in a file and display the lines that contain it.
  #[derive(Parser)]
  struct Cli {
      /// The pattern to look for
      pattern: String,
      /// The path to the file to read
      path: std::path::PathBuf,
  }
  ```

### Documentation Generation
- Use `clap_mangen` to auto-generate man pages
- Implement in `build.rs` for compile-time generation
- Generates both `--help` and manual pages automatically

### Distribution Methods (in order of user convenience)

1. **cargo install** (easiest to set up)
   - Good for Rust developers
   - Requires Rust toolchain on user machine
   - Compiles from source (slow)

2. **Binary releases** (GitHub releases)
   - Pre-compiled binaries for different platforms
   - Use CI/CD (Travis CI, GitHub Actions) to build
   - Target `x86_64-unknown-linux-musl` for Linux compatibility
   - Set `MACOSX_DEPLOYMENT_TARGET=10.7` for macOS compatibility

3. **Package managers** (most user-friendly)
   - Homebrew (macOS): add Formula file
   - Various Linux package managers
   - Tools: `cargo-bundle`, `cargo-deb`, `cargo-aur`

## File Requirements

### README.md
- Must be present in each crate directory
- Include: overview, usage examples, installation instructions
- For CLI crates: show `cargo install <crate-name>` command

### LICENSE files
- Must have LICENSE files matching the license field
- Dual MIT/Apache-2.0 is standard for Rust ecosystem

### Binary metadata
- Binary crates automatically get `[[bin]]` section
- No special configuration needed for simple single-binary crates

## Key Recommendations

1. **Start simple**: Begin with `cargo publish`, then add binary releases, finally package managers
2. **Follow ripgrep model**: Support multiple installation methods
3. **Generate documentation**: Use clap's auto-generation for help and man pages
4. **Test thoroughly**: Use `cargo publish --dry-run` before actual publishing
5. **Consider ergonomics**: Shell completions, proper error messages, progress indicators

## Workspace-Specific Considerations

- Each crate in a workspace needs its own README.md within its directory
- Version inheritance: use `version.workspace = true` in member crates
- Dependencies between workspace members need special handling for publishing
- Publish in dependency order (leaf crates first)