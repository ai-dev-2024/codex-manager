# Architecture Documentation

## System Overview

Codex Manager is a local proxy service that manages multiple OpenAI API accounts, providing intelligent routing, usage monitoring, and failover capabilities.

## Design Principles

1. **Local-First**: All data stays on your machine
2. **Security**: Strong encryption for credentials at rest
3. **Transparency**: OpenAI-compatible API, no vendor lock-in
4. **Reliability**: Automatic failover and circuit breaker patterns
5. **Simplicity**: Minimal configuration, sensible defaults

## Core Components

### 1. Account Model (`src/models/`)

**Purpose**: Define the data structures for accounts and usage tracking.

**Key Structures**:
- `Account`: Represents a single OpenAI API tenant
  - Unique ID (UUID v4)
  - Encrypted API key
  - Optional organization ID
  - Usage limits (daily/monthly)
  - Priority level for routing
  - Model scope restrictions
  
- `UsageSnapshot`: Captures usage metrics at a point in time
  - Token usage counts
  - Cost estimates
  - Budget remaining
  - Utilization ratios

- `RequestContext`: Routing context for each request
  - Target model
  - Session ID (for sticky routing)
  - Estimated token count

### 2. Storage Layer (`src/storage/`)

**Purpose**: Secure persistence of account data and usage history.

**Technology**: SQLite with AES-256-GCM encryption

**Security Features**:
- Master key derivation via Argon2id
- Unique nonce per encryption operation
- Combined nonce+ciphertext storage (Base64)
- Atomic write operations (temp file + rename)

**Schema**:
- `accounts`: Main account table with encrypted API keys
- `usage_snapshots`: Time-series usage data
- `metadata`: Application metadata

**Indexes**:
- `idx_accounts_enabled`: Fast filtering of enabled accounts
- `idx_accounts_priority`: Priority-based sorting
- `idx_usage_account`: Usage lookup by account
- `idx_usage_timestamp`: Time-based queries

### 3. Routing Engine (`src/routing/`)

**Purpose**: Determine which account to use for each request.

**Strategies**:

#### Least Utilized (Default)
```rust
// Select account with lowest utilization_ratio()
// utilization = usage / hard_limit
```
- Minimizes risk of hitting limits
- Balances load across accounts
- Best for general use

#### Round Robin
```rust
// Cycle through available accounts
// index = (index + 1) % count
```
- Even distribution
- Simple and predictable
- Good for similar-priority accounts

#### Priority
```rust
// Always select highest priority enabled account
// Falls back to next priority on failure
```
- Use primary account first
- Backup accounts as fallback
- Good for tiered setups

#### Sticky
```rust
// Route same content to same account
// session_id = hash(first_user_message)
```
- Maximizes prompt caching
- Consistent context
- Best for chat applications

**Circuit Breaker Pattern**:
```
Normal: Closed (requests allowed)
  ↓ 3 consecutive errors
Open: Block requests for 60 seconds
  ↓ After timeout
Half-Open: Allow test request
  ↓ Success
Closed: Resume normal operation
```

### 4. Usage Poller (`src/usage/`)

**Purpose**: Fetch and track usage data from OpenAI APIs.

**Endpoints**:
- `/v1/dashboard/billing/usage`: Monthly usage in cents
- `/v1/dashboard/billing/subscription`: Account limits
- `/v1/usage`: Token-level usage (if available)

**Polling Strategy**:
- Application startup: Full refresh
- Manual refresh: On-demand
- Background: Exponential backoff on errors
- Minimum interval: 60 seconds
- Maximum interval: 3600 seconds (1 hour)

**Backoff Algorithm**:
```rust
interval = min_interval + (2^errors)
capped_at = max_interval
```

### 5. Proxy Server (`src/proxy/`)

**Purpose**: Accept OpenAI-compatible requests and forward to appropriate account.

**Technology Stack**:
- Axum: Web framework
- Tower: Middleware layer
- Hyper: HTTP implementation
- Reqwest: Upstream client

**Endpoints**:
- `GET /health`: Health check
- `GET /v1/models`: List available models
- `POST /v1/chat/completions`: Chat completions (with streaming)
- `POST /v1/completions`: Text completions
- `POST /v1/embeddings`: Embeddings
- `POST /v1/images/generations`: DALL-E

**Request Flow**:
```
1. Receive request
2. Authenticate (check CAM_API_KEY)
3. Parse request body
4. Extract model and session ID
5. Query routing engine
6. Inject account API key
7. Forward to OpenAI
8. Stream response
9. Track success/failure
```

**Streaming Support**:
- Server-sent events (SSE) format
- Transparent proxying
- No buffering of large responses
- Proper error propagation

### 6. TUI Interface (`src/ui/`)

**Purpose**: Interactive terminal UI for management.

**Technology**: Ratatui with Crossterm

**Views**:
- Accounts: List, add, delete, toggle
- Routing: View routing statistics
- Status: System status and logs

**Key Bindings**:
- Navigation: Tab, Arrow keys
- Actions: a (add), d (delete), e (toggle), r (refresh)
- Exit: q, Esc

## Data Flow

### Account Creation
```
User Input (label, api_key)
    ↓
Create Account struct
    ↓
Encrypt API key (AES-256-GCM)
    ↓
Insert into SQLite
    ↓
Return account ID
```

### Request Routing
```
HTTP Request
    ↓
Parse model from body
    ↓
Build RequestContext
    ↓
Filter accounts (enabled, under limit)
    ↓
Apply routing strategy
    ↓
Select best account
    ↓
Forward with account's API key
    ↓
Stream response
    ↓
Report success/failure
```

### Usage Update
```
Timer/Trigger
    ↓
Load all accounts
    ↓
For each account:
    ├── Fetch billing usage
    ├── Fetch subscription limits
    └── Fetch token usage
    ↓
Save UsageSnapshot
    ↓
Update routing engine
```

## Security Architecture

### Threat Model

**Assets to Protect**:
1. OpenAI API keys (high value)
2. Usage data (medium value)
3. Account metadata (low value)

**Threats**:
1. Unauthorized access to database
2. Memory dumps exposing keys
3. Network interception
4. Malicious proxy usage

### Defenses

#### Encryption at Rest
```
Master Key (user-provided)
    ↓
Argon2id (salt + key stretching)
    ↓
256-bit encryption key
    ↓
AES-256-GCM (random nonce per operation)
    ↓
Encrypted API key stored
```

#### Access Control
- Database file permissions (OS-level)
- Master key required to decrypt
- No key caching in memory longer than necessary
- Automatic key clearing on drop

#### Network Security
- Localhost binding by default
- API key authentication for proxy
- HTTPS to OpenAI (upstream)
- No credential logging

## Performance Considerations

### Database
- Connection pooling (via r2d2 in production)
- Prepared statements for repeated queries
- Indexes on frequently filtered columns
- WAL mode for better concurrency

### Routing
- In-memory account cache
- DashMap for concurrent session tracking
- RwLock for read-heavy operations
- Atomic counters for statistics

### Proxy
- Keep-alive connections to OpenAI
- Streaming responses (no buffering)
- Efficient header injection
- Async/await throughout

## Scalability

### Single Instance
- Handles hundreds of concurrent requests
- Suitable for individual developers
- Low memory footprint (~50MB)

### Multi-Instance (Future)
- Shared state via SQLite
- No locking conflicts
- Horizontal scaling possible
- Load balancer in front

## Error Handling

### Graceful Degradation
1. Try primary account
2. If fails, try next available
3. If all fail, return error to client
4. Never block indefinitely

### Retry Strategy
- No automatic retries (client responsibility)
- Circuit breaker prevents cascade failures
- Exponential backoff for usage polling
- Clear error messages for debugging

## Monitoring

### Metrics
- Request count per account
- Error rate per account
- Response latency
- Circuit breaker state
- Queue depth

### Logging
- Request/response (debug level)
- Routing decisions (info level)
- Errors and warnings (warn/error level)
- No sensitive data in logs

## Configuration

### Environment Variables
- `CAM_MASTER_KEY`: Database encryption key
- `RUST_LOG`: Logging level (debug, info, warn, error)

### Config File
- `proxy.bind_addr`: Listen address
- `proxy.api_key`: Proxy authentication key
- `routing.strategy`: Routing algorithm
- `polling.interval_seconds`: Usage check frequency

### Runtime
- All config hot-reloadable (except bind address)
- No restart required for most changes
- Account updates immediate

## Testing Strategy

### Unit Tests
- Storage encryption/decryption
- Routing algorithm correctness
- Usage calculation accuracy

### Integration Tests
- Full request flow
- Circuit breaker behavior
- Concurrent request handling

### Manual Tests
- TUI navigation
- Real OpenAI API calls
- Error scenarios

## Future Enhancements

### Planned
- [ ] Web-based GUI (alternative to TUI)
- [ ] Account import/export
- [ ] Usage alerts and notifications
- [ ] Cost forecasting
- [ ] Team sharing (encrypted sync)

### Under Consideration
- [ ] Support for other providers (Anthropic, Google)
- [ ] Rate limiting per client
- [ ] Request/response logging
- [ ] Metrics export (Prometheus)
- [ ] Docker container

## References

- Antigravity Manager: https://github.com/lbjlaq/Antigravity-Manager
- OpenAI API Docs: https://platform.openai.com/docs
- Axum Framework: https://github.com/tokio-rs/axum
- Ratatui: https://github.com/ratatui-org/ratatui
