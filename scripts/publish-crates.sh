#!/bin/bash
set -e

# Context-Casial-Xpress Crates.io Publishing Script
# Publishes all crates in dependency order with proper versioning

VERSION="1.0.0"
DRY_RUN=false
SKIP_TESTS=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Help function
show_help() {
    echo "Context-Casial-Xpress Crates.io Publishing Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -v, --version VERSION    Set version number (default: 1.0.0)"
    echo "  -d, --dry-run           Perform dry run without actual publishing"
    echo "  -s, --skip-tests        Skip running tests before publishing"
    echo "  -h, --help              Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  CARGO_REGISTRY_TOKEN    Crates.io API token (required)"
    echo ""
    echo "Examples:"
    echo "  $0 --version 1.0.1"
    echo "  $0 --dry-run"
    echo "  CARGO_REGISTRY_TOKEN=<token> $0 --version 1.0.0"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -s|--skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "crates" ]]; then
    print_error "This script must be run from the context-casial-xpress root directory"
    exit 1
fi

# Check for cargo registry token
if [[ -z "$CARGO_REGISTRY_TOKEN" ]] && [[ "$DRY_RUN" == "false" ]]; then
    print_error "CARGO_REGISTRY_TOKEN environment variable is required for publishing"
    print_error "You can get a token from https://crates.io/me"
    print_error "Set it with: export CARGO_REGISTRY_TOKEN=<your-token>"
    exit 1
fi

print_status "Publishing Context-Casial-Xpress crates to crates.io"
print_status "Version: $VERSION"
print_status "Dry run: $DRY_RUN"
print_status "Skip tests: $SKIP_TESTS"
echo ""

# Define crates in dependency order (dependencies first)
CRATES=(
    "casial-core"
    "casial-wasm" 
    "casial-server"
)

# Update version in all Cargo.toml files
update_version() {
    local crate_name="$1"
    local crate_dir="crates/$crate_name"
    
    print_status "Updating version in $crate_dir/Cargo.toml to $VERSION"
    
    # Update version using sed (cross-platform compatible)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" "$crate_dir/Cargo.toml"
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$VERSION\"/" "$crate_dir/Cargo.toml"
    fi
    
    # Update internal dependencies
    for dep_crate in "${CRATES[@]}"; do
        if [[ "$dep_crate" != "$crate_name" ]]; then
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/^$dep_crate = { version = \".*\", path = /casial-$dep_crate = { version = \"$VERSION\", path = /" "$crate_dir/Cargo.toml"
            else
                sed -i "s/^$dep_crate = { version = \".*\", path = /casial-$dep_crate = { version = \"$VERSION\", path = /" "$crate_dir/Cargo.toml"
            fi
        fi
    done
}

# Update root Cargo.toml workspace version
print_status "Updating workspace version in Cargo.toml"
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
else
    sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
fi

# Update versions in all crates
for crate in "${CRATES[@]}"; do
    update_version "$crate"
done

# Run tests if not skipped
if [[ "$SKIP_TESTS" == "false" ]]; then
    print_status "Running tests before publishing..."
    if ! cargo test --all; then
        print_error "Tests failed! Aborting publication."
        exit 1
    fi
    print_success "All tests passed!"
else
    print_warning "Skipping tests as requested"
fi

# Build all crates to ensure they compile
print_status "Building all crates to ensure compilation..."
if ! cargo build --release --all; then
    print_error "Build failed! Aborting publication."
    exit 1
fi
print_success "All crates built successfully!"

# Function to publish a single crate
publish_crate() {
    local crate_name="$1"
    local crate_dir="crates/$crate_name"
    
    print_status "Publishing casial-$crate_name..."
    
    cd "$crate_dir"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        print_warning "DRY RUN: Would publish casial-$crate_name version $VERSION"
        cargo publish --dry-run
    else
        # Check if this version already exists
        if cargo search "casial-$crate_name" | grep -q "casial-$crate_name = \"$VERSION\""; then
            print_warning "casial-$crate_name version $VERSION already exists on crates.io, skipping..."
        else
            cargo publish --token "$CARGO_REGISTRY_TOKEN"
            print_success "Published casial-$crate_name version $VERSION"
            
            # Wait a bit for crates.io to propagate
            print_status "Waiting 30 seconds for crates.io propagation..."
            sleep 30
        fi
    fi
    
    cd - > /dev/null
}

# Publish crates in dependency order
for crate in "${CRATES[@]}"; do
    publish_crate "$crate"
done

# Create git tag for the release
if [[ "$DRY_RUN" == "false" ]]; then
    print_status "Creating git tag v$VERSION..."
    if git tag -a "v$VERSION" -m "Release version $VERSION - Context-Casial-Xpress consciousness-aware context coordination server"; then
        print_success "Created git tag v$VERSION"
        print_status "Push the tag with: git push origin v$VERSION"
    else
        print_warning "Failed to create git tag (may already exist)"
    fi
fi

print_success "🎉 Context-Casial-Xpress publication complete!"
echo ""
echo "Published crates with mixed licensing:"
echo "  📦 casial-core @ $VERSION (MIT OR Apache-2.0)"
echo "  📦 casial-wasm @ $VERSION (Fair Use License)"
echo "  📦 casial-server @ $VERSION (Fair Use License)"
echo ""
echo "🔗 Crates.io links:"
for crate in "${CRATES[@]}"; do
    echo "  https://crates.io/crates/casial-$crate"
done
echo ""
if [[ "$DRY_RUN" == "false" ]]; then
    echo "📋 Next steps:"
    echo "  1. Push git tag: git push origin v$VERSION"
    echo "  2. Create GitHub release from tag"
    echo "  3. Update documentation with new version"
    echo "  4. Update Smithery registry"
    echo ""
    echo "🌟 Context-Casial-Xpress is now available on crates.io!"
    echo "   Built stronger under pressure, like hydraulic lime 🏗️"
fi