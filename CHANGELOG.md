# Changelog

All notable changes to Codex Manager will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-02-03

### Added

#### Core Features
- **Multi-Account Management**: Store and manage multiple OpenAI API accounts with encrypted local storage
- **Usage Monitoring**: Real-time usage tracking with billing integration and limit enforcement
- **Smart Routing**: Intelligent account selection with multiple strategies (least-utilized, round-robin, priority, sticky)
- **Local Proxy Server**: OpenAI-compatible HTTP proxy with streaming support
- **Circuit Breaker**: Automatic failover when accounts hit rate limits or errors
- **Encrypted Storage**: AES-256-GCM encryption with Argon2id key derivation for all credentials
- **Cross-Platform Support**: Native builds for Windows, macOS, and Linux

#### Account Management
- UUID-based account identification
- Support for organization IDs
- Model scoping with wildcard support
- Daily and monthly usage limits
- Priority-based routing preferences
- Enable/disable account states
- Account labeling and metadata

#### Routing Engine
- **Least Utilized Strategy** (default): Selects account with lowest usage ratio
- **Round Robin Strategy**: Even distribution across accounts
- **Priority Strategy**: Tiered fallback system
- **Sticky Strategy**: Session-based routing for prompt caching
- Circuit breaker pattern with automatic recovery
- Real-time account health monitoring

#### Proxy Server
- Axum-based HTTP server
- OpenAI-compatible endpoints:
  - `GET /v1/models` - List available models
  - `POST /v1/chat/completions` - Chat completions with streaming
  - `POST /v1/completions` - Text completions
  - `POST /v1/embeddings` - Embeddings
  - `POST /v1/images/generations` - DALL-E image generation
- Server-sent events (SSE) streaming support
- Authentication middleware
- Request forwarding with account injection
- Error propagation and handling

#### Usage Monitoring
- OpenAI billing API integration
- Subscription and limit fetching
- Token usage tracking
- Exponential backoff polling
- Usage history storage
- Real-time utilization calculations

#### Storage Layer
- SQLite database with WAL mode
- AES-256-GCM encryption at rest
- Argon2id password hashing
- Atomic write operations
- Comprehensive indexing for performance
- In-memory database support for testing

#### User Interface
- Interactive TUI using Ratatui
- Tab-based navigation
- Account list and detail views
- Real-time status updates
- Keyboard shortcuts:
  - `Tab/Arrow Keys`: Navigate tabs
  - `↑/↓`: Navigate account list
  - `a`: Add new account
  - `d`: Delete selected account
  - `e`: Toggle account enabled/disabled
  - `r`: Refresh usage data
  - `q/ESC`: Quit

#### CLI Interface
- Comprehensive command-line interface using Clap
- Subcommands:
  - `proxy` - Start proxy server
  - `add` - Add new account
  - `list` - List all accounts
  - `remove` - Remove account
  - `show` - Show account details
  - `refresh` - Refresh usage data
  - `config` - Manage configuration
  - `tui` - Launch interactive TUI

#### Configuration
- TOML-based configuration files
- Environment variable support
- XDG directory specification compliance
- Hot-reload ready configuration
- Platform-specific default paths

#### Security
- Zero cloud dependency - 100% local processing
- No credential exfiltration
- No sensitive data in logs
- Secure defaults (localhost binding)
- Memory-safe Rust implementation

#### Documentation
- Comprehensive README with quick start
- Architecture documentation
- API documentation
- Build instructions
- OpenCode integration guide

### Technical Implementation

#### Dependencies
- Tokio async runtime
- Axum web framework
- Rusqlite with bundled SQLite
- AES-GCM encryption
- Argon2 password hashing
- Ratatui TUI framework
- Clap CLI parser

#### Performance
- Connection pooling ready
- Prepared statements
- In-memory caching
- Streaming responses
- Efficient header injection
- Low memory footprint (~50MB)

#### Error Handling
- Graceful degradation
- Circuit breaker pattern
- Exponential backoff
- Clear error messages
- Comprehensive logging

### Project Structure
```
codex-manager/
├── Cargo.toml              # Dependencies and build config
├── README.md               # User documentation
├── CHANGELOG.md            # This file
├── src/
│   ├── main.rs            # CLI entry point
│   ├── models/            # Account and usage models
│   ├── storage/           # Encrypted SQLite storage
│   ├── usage/             # OpenAI API client
│   ├── routing/           # Routing engine
│   ├── proxy/             # HTTP proxy server
│   ├── ui/                # TUI interface
│   └── config/            # Configuration management
└── docs/
    ├── ARCHITECTURE.md    # Architecture documentation
    ├── API.md             # API documentation
    └── BUILD.md           # Build instructions
```

### Known Limitations
- Usage API availability depends on OpenAI account type
- TUI input handling could be improved
- Primarily tested on Linux (Windows/macOS testing ongoing)
- Some dependency version conflicts may need resolution

### Future Roadmap
- Web-based GUI option
- Account import/export (encrypted)
- Usage alerts and notifications
- Cost forecasting
- Multi-provider support (Anthropic, Google)
- Docker container
- Prometheus metrics export

[0.3.0]: https://github.com/ai-dev-2024/codex-manager/releases/tag/v0.3.0
