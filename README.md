<div align="center">

<img src="public/banner.svg" alt="Codex Manager Banner" width="100%">

# Codex Manager

### Professional OpenAI Account Management & Switching Tool

[![Version](https://img.shields.io/badge/Version-0.3.0-blue.svg?style=flat-square)](https://github.com/ai-dev-2024/codex-manager/releases)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg?style=flat-square)]()
[![Tauri](https://img.shields.io/badge/Tauri-v2-orange.svg?style=flat-square)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB.svg?style=flat-square&logo=react)](https://react.dev)
[![Rust](https://img.shields.io/badge/Rust-000000.svg?style=flat-square&logo=rust)](https://rust-lang.org)

**[Download](https://github.com/ai-dev-2024/codex-manager/releases)** • **[Documentation](docs/)** • **[Support](https://ko-fi.com/ai_dev_2024)** • **[Issues](https://github.com/ai-dev-2024/codex-manager/issues)**

</div>

---

## 📖 Overview

**Codex Manager** is a professional desktop application for managing multiple OpenAI API accounts with intelligent routing, usage monitoring, and automatic failover. Built with Tauri v2 + React + Rust.

### The Problem

Managing multiple OpenAI API accounts is painful:
- ❌ Manual account switching
- ❌ No usage visibility across accounts
- ❌ Hitting rate limits unexpectedly
- ❌ No failover when accounts fail
- ❌ Credentials stored insecurely

### The Solution

Codex Manager solves all these:
- ✅ **One-click account switching**
- ✅ **Real-time usage monitoring**
- ✅ **Smart routing to avoid limits**
- ✅ **Automatic failover**
- ✅ **Encrypted credential storage**

---

## ✨ Features

### Account Management
- **Multi-Account Support**: Store unlimited OpenAI API accounts
- **Organization Support**: Handle org IDs and team accounts
- **Model Scoping**: Restrict accounts to specific models
- **Usage Limits**: Set daily/monthly spending caps
- **Priority System**: Tier your accounts (primary, backup, etc.)
- **Import/Export**: JSON backup and restore

### Smart Routing
Four routing strategies to fit your workflow:

| Strategy | Best For | Description |
|----------|----------|-------------|
| **Least Utilized** | Balanced Load | Uses account with lowest usage ratio |
| **Round Robin** | Even Distribution | Cycles through accounts evenly |
| **Priority** | Tiered Setup | Uses highest priority available account |
| **Sticky** | Chat Apps | Routes same session to same account |

### Usage Monitoring
- **Real-time Tracking**: Live usage and cost data
- **Billing Integration**: Connects to OpenAI billing API
- **Limit Alerts**: Visual warnings when approaching limits
- **Historical Data**: Track usage over time
- **Cost Estimation**: Project monthly spending

### Security
- 🔐 **AES-256-GCM Encryption**: All credentials encrypted at rest
- 🔐 **Argon2id Key Derivation**: Memory-hard password hashing
- 🔐 **Zero Cloud**: 100% local, no data leaves your machine
- 🔐 **No Telemetry**: No tracking or analytics
- 🔐 **Open Source**: Full transparency, audit the code

### Proxy Server
Built-in OpenAI-compatible proxy:
- 🌐 **OpenAI API Compatible**: Drop-in replacement
- 🌐 **Streaming Support**: Server-sent events (SSE)
- 🌐 **All Endpoints**: /v1/chat/completions, /v1/embeddings, etc.
- 🌐 **Circuit Breaker**: Automatic failover on errors
- 🌐 **Rate Limit Handling**: Smart retry with backoff

### User Interface
- 🎨 **Modern UI**: React 19 + Tailwind CSS + DaisyUI
- 🎨 **Dark/Light Mode**: Automatic system preference detection
- 🎨 **Dashboard**: Usage charts and account overview
- 🎨 **System Tray**: Quick access and minimize to tray
- 🎨 **Auto-start**: Optional system startup
- 🎨 **Keyboard Shortcuts**: Power user friendly

---

## 🚀 Quick Start

### Installation

#### Option 1: Download Pre-built Binaries

1. Go to [Releases](https://github.com/ai-dev-2024/codex-manager/releases)
2. Download for your platform:
   - **Windows**: `Codex.Manager_0.3.0_x64-setup.exe`
   - **macOS**: `Codex.Manager_0.3.0_universal.dmg`
   - **Linux**: `Codex.Manager_0.3.0_amd64.AppImage`
3. Install and run

#### Option 2: Homebrew (macOS/Linux)

```bash
# Coming soon
brew tap ai-dev-2024/codex-manager
brew install --cask codex-manager
```

#### Option 3: Build from Source

```bash
# Clone repository
git clone https://github.com/ai-dev-2024/codex-manager.git
cd codex-manager

# Install dependencies
npm install

# Build for production
npm run tauri build

# Or run in development
npm run tauri dev
```

### First Run

1. **Launch Codex Manager**
2. **Set Master Password**: This encrypts your account database
3. **Add Your First Account**:
   - Click "+ Add Account"
   - Enter account label (e.g., "Personal")
   - Paste your OpenAI API key
   - Set optional limits and priority
4. **Start Using**:
   - Configure your apps to use the proxy (see below)
   - Or use the built-in features directly

### Configure Your Apps

#### OpenCode

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
export OPENAI_API_KEY="sk-codex-manager"
export OPENAI_BASE_URL="http://127.0.0.1:8080/v1"

# Or set in OpenCode settings
```

#### Claude Code

```bash
export ANTHROPIC_API_KEY="sk-codex-manager"
export ANTHROPIC_BASE_URL="http://127.0.0.1:8080"
claude
```

#### Any OpenAI-Compatible Client

```python
import openai

client = openai.OpenAI(
    api_key="sk-codex-manager",
    base_url="http://127.0.0.1:8080/v1"
)
```

---

## 📚 Documentation

- **[Architecture](docs/ARCHITECTURE.md)** - System design and technical details
- **[API Reference](docs/API.md)** - Proxy API endpoints and usage
- **[Build Guide](docs/BUILD.md)** - Building from source for all platforms
- **[Changelog](CHANGELOG.md)** - Version history and release notes

---

## 🖼️ Screenshots

*Coming soon - Screenshots of the app in action*

---

## 🛠️ Tech Stack

### Frontend
- **React 19** - UI library
- **TypeScript** - Type safety
- **Vite** - Build tool
- **Tailwind CSS** - Styling
- **DaisyUI** - Component library
- **Zustand** - State management
- **Recharts** - Data visualization
- **Lucide React** - Icons

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
- **Tauri CLI** - Building and bundling

---

## 📋 System Requirements

### Minimum
- **OS**: Windows 10, macOS 10.15, Ubuntu 20.04
- **RAM**: 4 GB
- **Storage**: 200 MB
- **Network**: Internet connection for OpenAI API

### Recommended
- **OS**: Windows 11, macOS 14, Ubuntu 22.04
- **RAM**: 8 GB
- **Storage**: 500 MB

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Ways to Contribute
- 🐛 Report bugs
- 💡 Suggest features
- 📝 Improve documentation
- 🔧 Submit pull requests
- 🌍 Translate to other languages

---

## 💖 Support

If you find Codex Manager useful, please consider supporting its development:

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/ai_dev_2024)

Your support helps keep the project alive and enables new features!

---

## 📄 License

MIT License - See [LICENSE](LICENSE) file for details.

```
Copyright (c) 2025 ai-dev-2024

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
```

---

## 🙏 Acknowledgments

- Inspired by [Antigravity Manager](https://github.com/lbjlaq/Antigravity-Manager) by lbjlaq
- Built with [Tauri](https://tauri.app)
- Icons by [Lucide](https://lucide.dev)
- UI components by [DaisyUI](https://daisyui.com)

---

## 🔗 Links

- **Repository**: https://github.com/ai-dev-2024/codex-manager
- **Releases**: https://github.com/ai-dev-2024/codex-manager/releases
- **Issues**: https://github.com/ai-dev-2024/codex-manager/issues
- **Discussions**: https://github.com/ai-dev-2024/codex-manager/discussions
- **Support**: https://ko-fi.com/ai_dev_2024

---

<div align="center">

**Made with ❤️ by ai-dev-2024**

[⬆ Back to Top](#codex-manager)

</div>
