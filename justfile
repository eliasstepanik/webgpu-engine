# justfile
set dotenv-load          # .env is loaded automatically

preflight:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo doc --workspace --no-deps --document-private-items

build: preflight           # run depends on a clean preflight
    cargo build --workspace

run:
    cargo run -p game

# Build with editor feature enabled
build-editor:
    cargo build -p game --features editor

# Run with editor feature enabled
run-editor:
    cargo run -p game --features editor

# Build without editor (production build)
build-prod:
    cargo build -p game --release --no-default-features

# Run without editor (production mode)
run-prod:
    cargo run -p game --release --no-default-features

# Roll back to a stable release tag and rebuild
rollback tag:
    git fetch --tags
    git checkout {{tag}}
    just build
    echo "Rolled back to {{tag}}"