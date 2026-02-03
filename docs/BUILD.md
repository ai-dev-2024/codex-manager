# Build Instructions

## Prerequisites

### Required Tools

- **Rust** 1.75 or later
- **Cargo** (comes with Rust)
- **Git** for cloning the repository

### Platform-Specific Requirements

#### Linux
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev

# Fedora
sudo dnf install -y gcc openssl-devel

# Arch
sudo pacman -S base-devel openssl
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Or use Homebrew
brew install openssl
```

#### Windows
```powershell
# Install Visual Studio Build Tools
# Or use winget
winget install Microsoft.VisualStudio.2022.BuildTools

# Install Rust via rustup
# https://rustup.rs/
```

## Building from Source

### 1. Clone the Repository

```bash
git clone https://github.com/ai-dev-2024/codex-manager.git
cd codex-manager
```

### 2. Build Debug Version

```bash
cargo build
```

The debug binary will be at:
- Linux/macOS: `target/debug/codex-manager`
- Windows: `target/debug/codex-manager.exe`

### 3. Build Release Version

```bash
cargo build --release
```

The release binary will be at:
- Linux/macOS: `target/release/codex-manager`
- Windows: `target/release/codex-manager.exe`

Release builds are optimized with:
- `opt-level = 3` (maximum optimization)
- `lto = true` (link-time optimization)
- `codegen-units = 1` (better optimization)
- `strip = true` (remove debug symbols)
- `panic = "abort"` (smaller binary)

## Development Build

### With Debug Logging

```bash
RUST_LOG=debug cargo run
```

### With Specific Log Level

```bash
# Error only
RUST_LOG=error cargo run

# Warning and above
RUST_LOG=warn cargo run

# Info and above (default)
RUST_LOG=info cargo run

# All levels
RUST_LOG=trace cargo run
```

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Specific Test

```bash
cargo test test_name
```

## Cross-Compilation

### Install Cross-Compilation Tools

```bash
# Install cross tool
cargo install cross

# Or use rustup targets
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

### Build for Different Targets

```bash
# Windows (from Linux/macOS)
cross build --target x86_64-pc-windows-gnu --release

# macOS Intel (from Linux)
cross build --target x86_64-apple-darwin --release

# macOS ARM (from Linux)
cross build --target aarch64-apple-darwin --release

# Linux ARM64
cross build --target aarch64-unknown-linux-gnu --release
```

## Creating Release Packages

### Linux

```bash
# Create tarball
tar -czf codex-manager-linux-x64.tar.gz -C target/release codex-manager

# Create deb package (requires cargo-deb)
cargo install cargo-deb
cargo deb

# Create rpm package (requires cargo-rpm)
cargo install cargo-rpm
cargo rpm build
```

### macOS

```bash
# Create app bundle
cargo install cargo-bundle
cargo bundle --release

# Create dmg (requires create-dmg)
brew install create-dmg
create-dmg \
  --volname "Codex Manager" \
  --window-pos 200 120 \
  --window-size 600 400 \
  --icon-size 100 \
  --app-drop-link 450 185 \
  "Codex-Manager.dmg" \
  "target/release/bundle/osx/Codex Manager.app"
```

### Windows

```bash
# Create zip
7z a -tzip codex-manager-windows-x64.zip target/release/codex-manager.exe

# Create installer (requires WiX)
cargo install cargo-wix
cargo wix --no-build
```

## Docker Build

### Build Docker Image

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/codex-manager /usr/local/bin/
EXPOSE 8080
ENTRYPOINT ["codex-manager"]
CMD ["proxy"]
```

```bash
docker build -t codex-manager:latest .
```

### Run Docker Container

```bash
docker run -d \
  --name codex-manager \
  -p 8080:8080 \
  -e CAM_MASTER_KEY="your-master-key" \
  -v codex-data:/data \
  codex-manager:latest
```

## CI/CD Pipeline

### GitHub Actions

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: codex-manager
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: codex-manager.exe
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: codex-manager

    runs-on: ${{ matrix.os }}

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}

    - name: Build
      run: cargo build --release --target ${{ matrix.target }}

    - name: Upload Artifact
      uses: actions/upload-artifact@v3
      with:
        name: codex-manager-${{ matrix.target }}
        path: target/${{ matrix.target }}/release/${{ matrix.artifact }}
```

## Troubleshooting

### Common Issues

#### OpenSSL Not Found (Linux)
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# Fedora
sudo dnf install openssl-devel
```

#### SQLite Linking Issues
```bash
# Install SQLite development files
sudo apt-get install libsqlite3-dev
```

#### Windows MSVC Build Failures
```powershell
# Install Visual Studio Build Tools with C++ workload
# Or use GNU toolchain
rustup default stable-x86_64-pc-windows-gnu
```

#### macOS SDK Issues
```bash
# Update Xcode Command Line Tools
sudo rm -rf /Library/Developer/CommandLineTools
sudo xcode-select --install
```

### Build Performance

#### Faster Debug Builds
```bash
# Use mold linker (Linux)
cargo install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# Use lld linker (macOS)
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Use sccache for caching
cargo install sccache
export RUSTC_WRAPPER=sccache
```

#### Parallel Compilation
```bash
# Set number of parallel jobs
export CARGO_BUILD_JOBS=8
```

## Size Optimization

### Check Binary Size

```bash
cargo bloat --release
```

### Strip Symbols

```bash
# Linux/macOS
strip target/release/codex-manager

# Windows
strip target/release/codex-manager.exe
```

### Compress Binary

```bash
# Using UPX (Linux/macOS)
upx --best target/release/codex-manager
```

## Verification

### Check Binary

```bash
# Check binary info
file target/release/codex-manager

# Check dependencies
ldd target/release/codex-manager

# Run binary
target/release/codex-manager --version
```

### Test Installation

```bash
# Set master key
export CAM_MASTER_KEY="test-key"

# Run help
target/release/codex-manager --help

# Run proxy
target/release/codex-manager proxy &

# Test health endpoint
curl http://127.0.0.1:8080/health
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CAM_MASTER_KEY` | Database encryption key | Required |
| `CAM_CONFIG_DIR` | Configuration directory | Platform-specific |
| `CAM_DATA_DIR` | Data directory | Platform-specific |
| `RUST_LOG` | Log level | info |
| `RUST_BACKTRACE` | Enable backtraces | 0 |

## Next Steps

After building:

1. Set up your master key: `export CAM_MASTER_KEY="your-key"`
2. Add accounts: `codex-manager add "Personal" "sk-..."`
3. Start proxy: `codex-manager proxy`
4. Configure your OpenAI client to use `http://127.0.0.1:8080/v1`

See [README.md](../README.md) for full usage instructions.
