# CapCut Version Guard

A desktop utility to lock your CapCut version and prevent unwanted auto-updates.

## Features

- **Version Detection** — Scans for installed CapCut versions
- **Version Selection** — Choose which version to keep
- **Download Manager** — Links to legacy versions on Uptodown
- **Update Blocking** — Locks configuration files and creates blockers

## Quick Start

```bash
# Build
cargo build --release

# Run
cargo run --release

# The exe is at:
# target/release/capcut_guard_rust.exe
```

## Documentation

See `docs/` for:
- Feature specifications
- Architecture decisions (ADRs)
- Development setup

## Tech Stack

- **Rust** 2021 edition
- **eframe/egui** for GUI
- **egui-phosphor** for icons

## License

MIT
