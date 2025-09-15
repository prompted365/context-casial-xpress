# Trusted Publishing Setup Guide 🔐

✅ **STATUS: READY FOR TRUSTED PUBLISHING SETUP**

All infrastructure is complete! Just need to configure crates.io settings.

## ✅ What's Been Completed

### 🚀 GitHub Actions Workflow
- **File**: `.github/workflows/release.yml`
- **Trigger**: Push tags starting with `v*` (e.g., `v0.1.4`)
- **Features**: 
  - Automated publishing in correct dependency order
  - Built-in testing before publishing
  - Automatic GitHub releases with detailed notes
  - Uses OIDC for secure authentication

### 📦 Enhanced Crate Metadata
- **Better discoverability**: Optimized keywords and categories
- **SEO improvements**: Enhanced descriptions and documentation links
- **Badges**: Added crates.io, docs.rs, and license badges
- **Rust version**: Specified minimum supported version (1.75.0)

## 🔧 What You Need to Do

### 1. Set Up Trusted Publishing on crates.io

For each crate, go to crates.io settings and configure:

#### casial-core
- **URL**: https://crates.io/crates/casial-core/settings
- **Repository owner**: `prompted365`
- **Repository name**: `context-casial-xpress`
- **Workflow filename**: `release.yml`
- **Environment**: `release` (recommended for security)

#### casial-server  
- **URL**: https://crates.io/crates/casial-server/settings
- **Repository owner**: `prompted365` 
- **Repository name**: `context-casial-xpress`
- **Workflow filename**: `release.yml`
- **Environment**: `release`

#### casial-wasm
- **URL**: https://crates.io/crates/casial-wasm/settings  
- **Repository owner**: `prompted365`
- **Repository name**: `context-casial-xpress`
- **Workflow filename**: `release.yml`
- **Environment**: `release`

### 2. Set Up GitHub Environment (Recommended)

1. Go to your repo: https://github.com/prompted365/context-casial-xpress/settings/environments
2. Create environment named `release`
3. Add protection rules:
   - **Required reviewers**: Add yourself or trusted collaborators
   - **Deployment branches**: Only `main` branch
   - **Wait timer**: Optional 5-10 minute delay

### 3. Test the Workflow

Once trusted publishing is configured:

```bash
# Bump version to 0.1.4
cargo set-version --workspace --bump patch

# Commit the version bump
git add .
git commit -m "Bump version to v0.1.4"
git push origin main

# Create and push a tag to trigger automated publishing
git tag v0.1.4 -m "v0.1.4: Test trusted publishing workflow"
git push origin v0.1.4
```

The workflow will:
1. ✅ Check and test the workspace
2. 🔐 Authenticate via OIDC (no API tokens needed!)
3. 📦 Publish crates in correct order
4. 🚀 Create GitHub release with detailed notes

## 🎯 Benefits of This Setup

### Security
- ✅ **No long-lived API tokens** in repo secrets
- ✅ **Cryptographic verification** via OIDC
- ✅ **Time-limited tokens** (30 minutes max)
- ✅ **Repository verification** prevents unauthorized publishing

### Automation  
- ✅ **Consistent publishing** process
- ✅ **Automatic testing** before publish
- ✅ **GitHub releases** with changelogs
- ✅ **Dependency order** handling

### Discoverability
- ✅ **Enhanced metadata** for better search results
- ✅ **SEO-optimized** descriptions and keywords
- ✅ **Professional badges** and documentation
- ✅ **Clear licensing** information

## 🔄 Migration from API Tokens

If you currently have API tokens:

1. ✅ Set up trusted publishing (above steps)
2. ✅ Test with a new release
3. ✅ Remove API tokens from repo secrets once confirmed working

Both methods can run in parallel during transition.

## 📈 Enhanced Discoverability

### New Keywords
- **casial-core**: `["ai", "agents", "consciousness", "coordination", "substrate"]`
- **casial-server**: `["server", "websocket", "mcp", "ai", "agents"]` 
- **casial-wasm**: `["wasm", "webassembly", "browser", "javascript", "ai"]`

### Improved Categories
- **algorithms**, **data-structures**, **concurrency** for better algorithm discovery
- **network-programming**, **wasm** for platform-specific searches
- **api-bindings** for integration use cases

### Documentation Links
- All crates now point to **docs.rs** for API documentation
- Enhanced README with examples and architecture diagrams
- Clear installation instructions and usage examples

## 🎉 Next Steps

After setting up trusted publishing:

1. **Test the workflow** with a patch version bump
2. **Monitor the automation** in GitHub Actions
3. **Update your release process** to use git tags instead of manual publishing
4. **Enjoy secure, automated publishing**! 🚀

---

**Questions?** Contact breyden@prompted.community or open an issue!