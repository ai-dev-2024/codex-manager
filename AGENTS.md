# AGENTS.md - Codex Manager

## Project Overview

**Codex Manager** is a professional desktop application for managing multiple OpenAI API accounts with intelligent routing, usage monitoring, and automatic failover. Built with Tauri v2 + React + TypeScript + Rust.

**Version**: 0.3.0  
**License**: MIT  
**Repository**: https://github.com/ai-dev-2024/codex-manager  
**Support**: https://ko-fi.com/ai_dev_2024

---

## Architecture

```
Codex Manager
├── Frontend (React 19 + TypeScript + Vite)
│   ├── Dashboard (usage stats, best account)
│   ├── Accounts (CRUD, import/export)
│   ├── Settings (proxy, routing, appearance)
│   └── About (version, credits)
│
├── Backend (Rust + Tauri v2)
│   ├── Commands (account, config, proxy)
│   ├── Routing Engine (4 strategies)
│   ├── Storage (Encrypted SQLite)
│   ├── Usage Poller (OpenAI API)
│   └── Proxy Server (Axum)
│
└── Distribution
    ├── GitHub Releases (automatic)
    ├── Windows: MSI, NSIS (.exe)
    ├── macOS: DMG (Universal)
    └── Linux: AppImage, DEB, RPM
```

---

## Tech Stack

### Frontend
- **React 19** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **Tailwind CSS** - Styling
- **DaisyUI** - Component library
- **Zustand** - State management
- **Recharts** - Charts/visualization
- **Lucide React** - Icons
- **i18next** - Internationalization

### Backend
- **Rust** - Systems language
- **Tauri v2** - Desktop framework
- **Tokio** - Async runtime
- **Axum** - Web framework (proxy)
- **Rusqlite** - SQLite database
- **AES-GCM** - Encryption
- **Argon2** - Password hashing

### Tools
- **GitHub Actions** - CI/CD
- **Tauri CLI** - Building/bundling

---

## File Structure

```
codex-manager/
├── .github/
│   ├── FUNDING.yml              # Ko-fi support link
│   └── workflows/
│       ├── ci.yml               # Test/build workflow
│       └── release.yml          # Release automation
│
├── docs/
│   ├── ARCHITECTURE.md          # System design
│   ├── API.md                   # API documentation
│   ├── BUILD.md                 # Build instructions
│   └── OPENCODE_INTEGRATION.md  # OpenCode setup
│
├── public/
│   ├── icon.svg                 # App icon (512x512)
│   └── banner.svg               # Banner image (1280x640)
│
├── src/                         # React Frontend
│   ├── components/
│   │   ├── ui/                  # shadcn/ui components
│   │   ├── layout/              # Layout components
│   │   ├── modals/              # Dialog components
│   │   └── ...
│   ├── pages/
│   │   ├── Dashboard.tsx        # Dashboard page
│   │   ├── Accounts.tsx         # Accounts page
│   │   ├── Settings.tsx         # Settings page
│   │   └── About.tsx            # About page
│   ├── stores/
│   │   ├── useAccountStore.ts   # Account state
│   │   └── useConfigStore.ts    # Config state
│   ├── hooks/
│   │   └── useTauri.ts          # Tauri bridge
│   ├── lib/
│   │   └── utils.ts             # Utilities
│   ├── types/
│   │   └── account.ts           # TypeScript types
│   ├── main.tsx                 # Entry point
│   ├── App.tsx                  # Main app
│   └── index.css                # Global styles
│
├── src-tauri/                   # Rust Backend
│   ├── src/
│   │   ├── main.rs              # App entry
│   │   ├── lib.rs               # Library exports
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── account.rs       # Account commands
│   │   │   ├── config.rs        # Config commands
│   │   │   └── proxy.rs         # Proxy commands
│   │   ├── modules/
│   │   │   ├── mod.rs
│   │   │   ├── tray.rs          # System tray
│   │   │   └── updater.rs       # Auto-updater
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── account.rs
│   │   │   └── config.rs
│   │   └── state/
│   │       └── mod.rs
│   ├── icons/                   # App icons
│   │   ├── 32x32.png
│   │   ├── 128x128.png
│   │   ├── 128x128@2x.png
│   │   ├── icon.icns            # macOS
│   │   └── icon.ico             # Windows
│   ├── capabilities/
│   │   └── default.json         # Tauri permissions
│   ├── Cargo.toml               # Rust dependencies
│   ├── tauri.conf.json          # Tauri config
│   └── build.rs                 # Build script
│
├── CHANGELOG.md                 # Version history
├── LICENSE                      # MIT License
├── README.md                    # Main documentation
├── AGENTS.md                    # This file
├── RELEASE_CHECKLIST.md         # Release checklist
├── package.json                 # Node.js dependencies
├── Cargo.toml                   # Workspace config
├── vite.config.ts               # Vite config
├── tsconfig.json                # TypeScript config
├── tailwind.config.js           # Tailwind config
├── components.json              # shadcn/ui config
└── index.html                   # HTML entry
```

---

## Key Commands

### Development
```bash
npm install                    # Install dependencies
npm run tauri dev             # Run in development mode
npm run tauri build           # Build production
```

### Git Operations
```bash
git init
git add .
git commit -m "Release v0.3.0"
git remote add origin https://github.com/ai-dev-2024/codex-manager.git
git push -u origin main
```

### Release
```bash
git tag -a v0.3.0 -m "Release v0.3.0"
git push origin v0.3.0
```

---

## Tauri Commands

### Account Commands
```rust
#[tauri::command]
async fn list_accounts() -> Result<Vec<Account>, String>

#[tauri::command]
async fn get_account(id: String) -> Result<Account, String>

#[tauri::command]
async fn add_account(account: AccountInput) -> Result<Account, String>

#[tauri::command]
async fn update_account(account: Account) -> Result<Account, String>

#[tauri::command]
async fn delete_account(id: String) -> Result<bool, String>

#[tauri::command]
async fn import_accounts(json: String) -> Result<Vec<Account>, String>

#[tauri::command]
async fn export_accounts() -> Result<String, String>

#[tauri::command]
async fn refresh_usage() -> Result<(), String>

#[tauri::command]
async fn switch_account(id: String) -> Result<Account, String>
```

### Config Commands
```rust
#[tauri::command]
async fn get_config() -> Result<Config, String>

#[tauri::command]
async fn set_config(config: Config) -> Result<Config, String>

#[tauri::command]
async fn reset_config() -> Result<Config, String>
```

### Proxy Commands
```rust
#[tauri::command]
async fn start_proxy(config: ProxyConfig) -> Result<(), String>

#[tauri::command]
async fn stop_proxy() -> Result<(), String>

#[tauri::command]
async fn get_proxy_status() -> Result<ProxyStatus, String>
```

---

## Routing Strategies

### 1. Least Utilized (Default)
Selects account with lowest usage ratio.
```rust
// utilization = current_usage / limit
// Select min(utilization)
```

### 2. Round Robin
Cycles through accounts evenly.
```rust
// index = (index + 1) % count
```

### 3. Priority
Uses highest priority enabled account.
```rust
// max(priority) where enabled=true
```

### 4. Sticky
Routes same session to same account.
```rust
// session_hash = hash(first_message)
// account_map[session_hash] = account_id
```

---

## Security Model

### Encryption
- **Algorithm**: AES-256-GCM
- **Key Derivation**: Argon2id (memory-hard)
- **Nonce**: 12 bytes, unique per operation
- **Storage**: nonce + ciphertext (Base64)

### Access Control
- Master password required on first run
- No credential caching
- Automatic memory clearing
- Database file encrypted

### Network Security
- Localhost binding by default (127.0.0.1:8080)
- API key authentication
- HTTPS to OpenAI upstream
- No credential logging

---

## Configuration

### Default Config Location
- **Linux**: `~/.config/codex-manager/config.toml`
- **macOS**: `~/Library/Application Support/com.codex-manager/config.toml`
- **Windows**: `%APPDATA%\codex-manager\config\config.toml`

### Default Values
```toml
[proxy]
bind_addr = "127.0.0.1:8080"
api_key = "sk-codex-manager"
openai_base_url = "https://api.openai.com"

[routing]
strategy = "least_utilized"
min_request_interval_ms = 100

[polling]
enabled = true
interval_seconds = 300

[ui]
theme = "system"
language = "en"
```

---

## GitHub Actions

### Triggers
- Push to tags matching `v*`
- Manual workflow dispatch

### Jobs
1. **create-release**: Creates draft release
2. **build-tauri**: Builds for all platforms
3. **publish-release**: Publishes release

### Platforms
- Windows (x64): MSI, NSIS
- macOS (Universal): DMG (Intel + ARM)
- Linux (x64): AppImage, DEB, RPM

---

## External APIs

### OpenAI API
- Billing endpoint: `https://api.openai.com/v1/dashboard/billing/usage`
- Subscription: `https://api.openai.com/v1/dashboard/billing/subscription`
- Standard API: `https://api.openai.com/v1/...`

### Local Proxy
- Base URL: `http://127.0.0.1:8080/v1`
- Health: `http://127.0.0.1:8080/health`

---

## Dependencies

### Production (package.json)
- react ^19.1.0
- react-dom ^19.1.0
- react-router-dom ^7.10.1
- zustand ^5.0.9
- recharts ^3.5.1
- lucide-react ^0.561.0
- @tauri-apps/api ^2
- @tauri-apps/plugin-opener ^2
- @tauri-apps/plugin-autostart ^2.5.1
- @tauri-apps/plugin-updater ^2.9.0
- tailwindcss ^3.4.19
- daisyui ^5.5.13
- i18next ^25.7.2

### Production (Cargo.toml)
- tauri = "2"
- tokio = { version = "1.43", features = ["full"] }
- axum = "0.7"
- rusqlite = { version = "0.32", features = ["bundled"] }
- aes-gcm = "0.10"
- argon2 = "0.5"
- chrono = "0.4"
- uuid = "1.12"

---

## Common Tasks

### Adding a New Page
1. Create `src/pages/NewPage.tsx`
2. Add route in `src/App.tsx`
3. Add navigation item in `src/components/layout/Layout.tsx`
4. Create any needed components

### Adding a Tauri Command
1. Create function in `src-tauri/src/commands/`
2. Export in `src-tauri/src/commands/mod.rs`
3. Register in `src-tauri/src/lib.rs`:
   ```rust
   .invoke_handler(tauri::generate_handler![new_command])
   ```
4. Call from frontend:
   ```typescript
   import { invoke } from '@tauri-apps/api/core';
   const result = await invoke('new_command', { arg: value });
   ```

### Adding a New Routing Strategy
1. Add variant to `RoutingStrategy` enum in `src-tauri/src/routing/mod.rs`
2. Implement selection logic in `select()` method
3. Update UI in Settings page
4. Add to config validation

### Creating a Release
1. Update version in:
   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`
   - `Cargo.toml` (root)
   - `CHANGELOG.md`

2. Commit and tag:
   ```bash
   git add .
   git commit -m "Release v0.3.1"
   git tag -a v0.3.1 -m "Release v0.3.1"
   git push origin main
   git push origin v0.3.1
   ```

3. GitHub Actions builds automatically

---

## Testing

### Manual Testing Checklist
- [ ] Add account
- [ ] Edit account
- [ ] Delete account
- [ ] Import accounts (JSON)
- [ ] Export accounts
- [ ] Switch routing strategy
- [ ] Start proxy server
- [ ] Make API request via proxy
- [ ] Check usage display
- [ ] Test dark/light mode
- [ ] Test auto-start
- [ ] Test system tray

### API Testing
```bash
# Health check
curl http://localhost:8080/health

# List models
curl http://localhost:8080/v1/models \
  -H "Authorization: Bearer sk-codex-manager"

# Chat completion
curl http://localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer sk-codex-manager" \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello"}]}'
```

---

## Troubleshooting

### Windows
- **Error**: "link.exe not found" → Install Visual Studio Build Tools
- **Error**: "Windows SDK not found" → Install Windows SDK

### macOS
- **Error**: "codesign failed" → Set `signingIdentity: null` in tauri.conf.json
- **Error**: "cc not found" → Run `xcode-select --install`

### Linux
- **Error**: "lssl not found" → Install `libssl-dev`
- **Error**: "webkit2gtk not found" → Install `libwebkit2gtk-4.1-dev`

### General
- **Error**: "tauri.conf.json not found" → Ensure you're in project root
- **Error**: "out of memory" → Reduce parallel jobs: `CARGO_BUILD_JOBS=2`

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CAM_MASTER_KEY` | Database encryption password | Required |
| `RUST_LOG` | Rust logging level | info |
| `RUST_BACKTRACE` | Enable backtraces | 0 |

---

## Resources

### Documentation
- [README.md](README.md) - Main documentation
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - System design
- [API.md](docs/API.md) - API reference
- [BUILD.md](docs/BUILD.md) - Build instructions
- [CHANGELOG.md](CHANGELOG.md) - Version history

### External Links
- [Tauri Docs](https://tauri.app/v1/guides/)
- [React Docs](https://react.dev)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tailwind CSS](https://tailwindcss.com)
- [shadcn/ui](https://ui.shadcn.com)

### Support
- Issues: https://github.com/ai-dev-2024/codex-manager/issues
- Discussions: https://github.com/ai-dev-2024/codex-manager/discussions
- Ko-fi: https://ko-fi.com/ai_dev_2024

---

## Version History

### v0.3.0 (Current)
- Initial release
- Multi-account management
- Smart routing (4 strategies)
- Encrypted storage
- Local proxy server
- Cross-platform builds

---

## License

MIT License - See [LICENSE](LICENSE) file

Copyright (c) 2025 ai-dev-2024

---

## Notes for Future Agents

1. **This is a Tauri v2 application** - Use Tauri v2 APIs, not v1
2. **React 19** - Use modern React patterns
3. **Rust backend** - All heavy operations in Rust
4. **Encrypted storage** - Never store credentials in plaintext
5. **GitHub Actions** - Builds are automated on tag push
6. **Version consistency** - Keep versions synced across all files
7. **Security first** - Review all code for security issues
8. **Cross-platform** - Test on Windows, macOS, Linux
9. **Documentation** - Update docs when adding features
10. **Support Ko-fi** - Include support link in user-facing docs

---

**Last Updated**: 2025-02-03  
**Version**: 0.3.0  
**Status**: Ready for release
