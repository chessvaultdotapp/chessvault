# List available tasks.
default:
    @just --list

# Verify the workspace in CI order, stopping on failure (requires cargo-nextest).
check:
    cargo build --workspace --all-targets --locked
    cargo nextest run --workspace --locked
    cargo test --workspace --doc --locked
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --locked --no-deps

# Verify one package; the desktop is binary-only and has no doctests.
check-package package:
    cargo build -p {{quote(package)}} --all-targets --locked
    cargo nextest run -p {{quote(package)}} --locked
    if [ {{quote(package)}} != chessvault ]; then cargo test -p {{quote(package)}} --doc --locked; fi
    cargo fmt --all -- --check
    cargo clippy -p {{quote(package)}} --all-targets --locked --no-deps

# Verify platform-dirs with native path resolution rather than development paths.
check-platform-native:
    cargo build -p platform-dirs --all-targets --locked --no-default-features
    cargo nextest run -p platform-dirs --locked --no-default-features
    cargo test -p platform-dirs --doc --locked --no-default-features
    cargo fmt --all -- --check
    cargo clippy -p platform-dirs --all-targets --locked --no-deps --no-default-features

# Launch the desktop application.
run:
    cargo run -p chessvault --locked

# Launch the desktop application with app debug logging.
run-debug:
    RUST_LOG=chessvault=debug cargo run -p chessvault --locked

# Optimize SVG artwork in place (requires project dependencies installed with pnpm).
optimize-assets:
    @command -v node >/dev/null 2>&1 || { echo "Node.js is required: https://nodejs.org/en/download" >&2; exit 1; }
    @command -v pnpm >/dev/null 2>&1 || { echo "pnpm is required: https://pnpm.io/installation" >&2; exit 1; }
    cd apps/chessvault && pnpm exec svgo --folder assets --recursive

# Apply workspace-wide Rust formatting.
fmt:
    cargo fmt --all
