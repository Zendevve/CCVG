<div align="center">
    <img src="docs/images/banner_large.png" alt="CC Version Guard Banner" width="100%" />
</div>

<br>

<div align="center">
    <a href="https://prgportfolio.com" target="_blank">
        <img src="https://img.shields.io/badge/PRG-Gold Project-FFD700?style=for-the-badge&logo=data:image/svg%2bxml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/Pgo8IURPQ1RZUEUgc3ZnIFBVQkxJQyAiLS8vVzNDLy9EVEQgU1ZHIDIwMDEwOTA0Ly9FTiIKICJodHRwOi8vd3d3LnczLm9yZy9UUi8yMDAxL1JFQy1TVkctMjAwMTA5MDQvRFREL3N2ZzEwLmR0ZCI+CjxzdmcgdmVyc2lvbj0iMS4wIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciCiB3aWR0aD0iMjYuMDAwMDAwcHQiIGhlaWdodD0iMzQuMDAwMDAwcHQiIHZpZXdCb3g9IjAgMCAyNi4wMDAwMDAgMzQuMDAwMDAwIgogcHJlc2VydmVBc3BlY3RSYXRpbz0ieE1pZFlNaWQgbWVldCI+Cgo8ZyB0cmFuc2Zvcm09InRyYW5zbGF0ZSgwLjAwMDAwMCwzNC4wMDAwMDApIHNjYWxlKDAuMTAwMDAwLC0wLjEwMDAwMCkiCmZpbGw9IiNGRkQ3MDAiIHN0cm9rZT0ibm9uZSI+CjxwYXRoIGQ9Ik0xMiAzMjggYy04IC04IC0xMiAtNTEgLTEyIC0xMzUgMCAtMTA5IDIgLTEyNSAxOSAtMTQwIDQyIC0zOCA0OAotNDIgNTkgLTMxIDcgNyAxNyA2IDMxIC0xIDEzIC03IDIxIC04IDIxIC0yIDAgNiAyOCAxMSA2MyAxMyBsNjIgMyAwIDE1MCAwCjE1MCAtMTE1IDMgYy04MSAyIC0xMTkgLTEgLTEyOCAtMTB6IG0xMDIgLTc0IGMtNiAtMzMgLTUgLTM2IDE3IC0zMiAxOCAyIDIzCjggMjEgMjUgLTMgMjQgMTUgNDAgMzAgMjUgMTQgLTE0IC0xNyAtNTkgLTQ4IC02NiAtMjAgLTUgLTIzIC0xMSAtMTggLTMyIDYKLTIxIDMgLTI1IC0xMSAtMjIgLTE2IDIgLTE4IDEzIC0xOCA2NiAxIDc3IDAgNzIgMTggNzIgMTMgMCAxNSAtNyA5IC0zNnoKbTExNiAtMTY5IGMwIC0yMyAtMyAtMjUgLTQ5IC0yNSAtNDAgMCAtNTAgMyAtNTQgMjAgLTMgMTQgLTE0IDIwIC0zMiAyMCAtMTgKMCAtMjkgLTYgLTMyIC0yMCAtNyAtMjUgLTIzIC0yNiAtMjMgLTIgMCAyOSA4IDMyIDEwMiAzMiA4NyAwIDg4IDAgODggLTI1eiIvPgo8L2c+Cjwvc3ZnPgo=" alt="Gold" />
    </a>
    <br>
    <a href="LICENSE">
        <img src="https://img.shields.io/badge/License-Proprietary-blue.svg?style=for-the-badge" alt="Proprietary License" />
    </a>
    <a href="https://tauri.app">
        <img src="https://img.shields.io/badge/Tauri-2.0-FFC131?style=for-the-badge&logo=tauri&logoColor=black" alt="Tauri" />
    </a>
    <a href="https://www.rust-lang.org">
        <img src="https://img.shields.io/badge/Rust-1.70+-000000?style=for-the-badge&logo=rust" alt="Rust" />
    </a>
    <a href="https://developer.mozilla.org/en-US/docs/Web/JavaScript">
        <img src="https://img.shields.io/badge/JavaScript-ES6+-F7DF1E?style=for-the-badge&logo=javascript&logoColor=black" alt="JavaScript" />
    </a>
</div>

<br>

<div align="center">
  <b>CC Version Guard</b> is a powerful, secure version manager for CapCut desktop, giving creators full control over their editing environment.
</div>

> [!IMPORTANT]
> **Unofficial Tool:** This application is not affiliated with, endorsed by, or connected to ByteDance or CapCut. It is a local file manager designed to help users manage their own legally installed software versions.
>
> All "Legacy Versions" downloads are sourced **directly** from official CapCut servers (CDNs). We do not host or modify any CapCut binaries.

---------------

## Table of Contents

- [Features](#features)
- [Background Story](#background-story)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [What's Inside?](#whats-inside)
- [Contributing](#contributing)
- [License](#license)
- [Footer](#footer)

## Features

- 🛡️ **Auto-Update Blocker:** Prevent CapCut from updating automatically, keeping your preferred version stable.
- 💾 **Version Switching:** Instantly switch between installed versions without re-downloading.
- ⬇️ **Legacy Downloader:** Access and install previous official versions of CapCut directly from their servers.
- 📦 **Backup Manager:** Create and restore backups of your specific version installations.
- 🚀 **Performance Optimized:** Built with Rust and Tauri for a lightweight, blazing-fast experience.
- 🎨 **Tahoe Design:** A beautiful, modern interface following Apple's macOS Tahoe design system.

## Background Story

As a video editor, I was frustrated by mandatory updates that would often break my workflow or change features I relied on. I needed a way to stick to a stable version of CapCut while still having the flexibility to test new features when I chose to—not when the software decided.

I built **CC Version Guard** to give power back to the creators. It started as a simple script and evolved into a full-fledged application, prioritizing security, legality, and user experience.

## Getting Started

### Prerequisites

- Windows 10 or 11
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually installed by default)

### Installation

1. Download the latest installer from the [Releases](https://github.com/Zendevve/capcut-version-guard/releases) page.
2. Run `CC-Version-Guard-Setup.exe`.
3. Launch the application and select your CapCut installation.

## What's Inside?

```bash
├── .github/              # Community health files (PRG Gold Standard)
│   ├── CODE_OF_CONDUCT.md
│   ├── CONTRIBUTING.md
│   ├── CREDITS.md
│   └── SECURITY.md
├── src-tauri/            # Rust Backend
│   ├── src/
│   │   ├── commands/     # App logic (autostart, backup, protection)
│   │   └── main.rs       # Entry point
│   ├── icons/            # App icons
│   └── tauri.conf.json   # Configuration
├── src/                  # Frontend
│   ├── assets/           # UI Assets
│   ├── index.html        # Main View
│   ├── input.css         # Tailwind & Custom Styles
│   └── main.js           # UI Logic
└── README.md             # This file
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](.github/CONTRIBUTING.md) for details on how to submit pull requests, report issues, and improve the project.

## License

Distributed under the **CC Version Guard License**. See `LICENSE` for more information.

This is an **Open Core** project:
- The source code is proprietary for personal use.
- Binaries are provided for convenience.

## Footer

### Credits

- **Author:** [Zendevve](https://github.com/Zendevve)
- **Design:** Based on macOS Tahoe System

---------------

<div align="center">
    <a href="https://github.com/Zendevve/capcut-version-guard">
        <img src="src-tauri/icons/128x128.png" alt="Footer Icon" width="100" height="100"/>
    </a>
</div>
