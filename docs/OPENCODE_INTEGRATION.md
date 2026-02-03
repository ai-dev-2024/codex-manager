# OpenCode Integration Guide

This guide shows how to configure OpenCode to use Codex Account Manager for multi-account OpenAI API management.

## Overview

Codex Account Manager provides a local proxy server that:
- Accepts standard OpenAI API requests
- Routes them to the best available account
- Handles rate limiting and failover automatically
- Tracks usage across all accounts

OpenCode sees only one endpoint, while Codex Account Manager handles the complexity of multiple accounts.

## Setup Steps

### 1. Install and Configure Codex Account Manager

```bash
# Install (from source)
cargo install --path .

# Or download pre-built binary
# TODO: Add release links when available
```

### 2. Add Your OpenAI Accounts

```bash
export CAM_MASTER_KEY="your-secure-master-key"

# Add multiple accounts
codex-account-manager add "Personal" "sk-personal-..."
codex-account-manager add "Work" "sk-work-..."
codex-account-manager add "Backup" "sk-backup-..."

# Verify accounts
codex-account-manager list
```

### 3. Start the Proxy Server

```bash
# Start proxy (runs in foreground)
codex-account-manager proxy

# Or with custom port
codex-account-manager proxy --bind 127.0.0.1:8080
```

The proxy is now running at `http://127.0.0.1:8080`

### 4. Configure OpenCode

#### Option A: Environment Variables (Recommended)

Add to your shell profile (`.bashrc`, `.zshrc`, etc.):

```bash
# OpenCode will use these settings
export OPENAI_API_KEY="sk-codex-account-manager"
export OPENAI_BASE_URL="http://127.0.0.1:8080/v1"
```

Then reload your shell:
```bash
source ~/.bashrc  # or ~/.zshrc
```

#### Option B: OpenCode Configuration File

If OpenCode supports a config file:

```yaml
# ~/.config/opencode/config.yaml
openai:
  api_key: "sk-codex-account-manager"
  base_url: "http://127.0.0.1:8080/v1"
```

#### Option C: Command Line Flags

When running OpenCode:

```bash
opencode --openai-api-key="sk-codex-account-manager" \
         --openai-base-url="http://127.0.0.1:8080/v1"
```

## Verification

### Test the Proxy

```bash
# Test with curl
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer sk-codex-account-manager"

# Should return list of models
```

### Test with OpenCode

Start a conversation with OpenCode. It should:
- Connect successfully to the proxy
- Route requests through available accounts
- Continue working even if one account fails

### Monitor in TUI

In another terminal:

```bash
codex-account-manager
```

Watch the TUI to see:
- Which account is being used
- Real-time usage updates
- Routing decisions

## Advanced Configuration

### Account Priorities

Set different priorities for accounts:

```bash
# In TUI: Select account, press 'e' to edit, set priority
# Or directly in code:
```

Higher priority accounts are preferred (when using priority routing strategy).

### Usage Limits

Set limits to prevent over-spending:

```bash
# Edit account to add limits
# Daily and monthly USD limits
```

### Model Scoping

Restrict accounts to specific models:

```bash
# Configure account to only use certain models
# Supports wildcards like "gpt-4*"
```

### Routing Strategy

Edit `~/.config/codex-account-manager/config.toml`:

```toml
[routing]
strategy = "least_utilized"  # Default - best balance
# strategy = "round_robin"    # Even distribution
# strategy = "priority"       # Prefer high-priority accounts
# strategy = "sticky"         # Cache-friendly routing
```

## Troubleshooting

### Connection Refused

```bash
# Check if proxy is running
curl http://127.0.0.1:8080/health

# Should return: {"status":"ok",...}
```

### Authentication Failed

- Verify `CAM_MASTER_KEY` is set
- Check that accounts were added successfully: `codex-account-manager list`
- Ensure proxy API key matches: should be `sk-codex-account-manager` (default)

### No Available Accounts

```bash
# Check account status
codex-account-manager list

# Refresh usage data
codex-account-manager refresh

# Check if accounts are over limits in TUI
```

### Rate Limiting

The proxy handles rate limits automatically by:
1. Detecting 429 responses
2. Opening circuit breaker for that account
3. Routing to next available account
4. Retrying original request

No manual intervention needed!

## Performance Tips

### Enable Session Stickiness

For chat applications, enable sticky routing:

```toml
[routing]
strategy = "sticky"
```

This routes the same conversation to the same account, maximizing:
- Prompt cache hits
- Context consistency
- Cost savings

### Background Usage Polling

Enable automatic usage updates:

```toml
[polling]
enabled = true
interval_seconds = 300  # Check every 5 minutes
```

### Multiple Proxy Instances

For high-throughput scenarios:

```bash
# Terminal 1: Primary proxy
codex-account-manager proxy --bind 127.0.0.1:8080

# Terminal 2: Secondary proxy
codex-account-manager proxy --bind 127.0.0.1:8081

# Configure OpenCode with load balancer or round-robin
```

## Security Considerations

### Master Key

- Never commit `CAM_MASTER_KEY` to version control
- Use a strong, unique password
- Consider using a password manager
- The key is only needed to decrypt the database on startup

### API Keys

- API keys are encrypted at rest with AES-256-GCM
- Keys are never logged or transmitted except to OpenAI
- In-memory keys are cleared on shutdown

### Network

- Proxy binds to localhost by default (127.0.0.1)
- Only change to 0.0.0.0 if you need external access
- Use a firewall if exposing the proxy externally

## Architecture Diagram

```
OpenCode
    │
    │ HTTP Request
    │ Authorization: Bearer sk-codex-account-manager
    ▼
┌─────────────────────────┐
│   Codex Account Manager │
│   Proxy (Port 8080)     │
└───────────┬─────────────┘
            │
            │ Route to best account
            │ (Least utilized / Priority / etc.)
            ▼
┌─────────────────────────┐
│   Routing Engine        │
│   - Filter disabled     │
│   - Check limits        │
│   - Circuit breaker     │
└───────────┬─────────────┘
            │
            │ Inject account API key
            ▼
┌─────────────────────────┐
│   OpenAI API            │
│   api.openai.com        │
└─────────────────────────┘
```

## Example Workflow

```bash
# 1. Start proxy in terminal 1
codex-account-manager proxy

# 2. In another terminal, configure OpenCode
export OPENAI_API_KEY="sk-codex-account-manager"
export OPENAI_BASE_URL="http://127.0.0.1:8080/v1"

# 3. Use OpenCode normally
opencode

# 4. In terminal 3, monitor with TUI
codex-account-manager

# 5. Watch routing decisions and usage in real-time!
```

## Questions?

- Check the [README](README.md) for general documentation
- Review routing strategies in the architecture docs
- Open an issue for bugs or feature requests
