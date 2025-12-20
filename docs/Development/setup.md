# Development Setup

## Prerequisites

- Rust 1.70+ (install via rustup)
- Windows 10/11

## Setup

```bash
# Clone
git clone https://github.com/Zendevve/capcut-version-guard.git
cd capcut-version-guard

# Build
cargo build --release

# Run
cargo run --release
```

## Commands

| Command | Description |
| --- | --- |
| `cargo build --release` | Build optimized binary |
| `cargo run --release` | Build and run |
| `cargo fmt` | Format code |
| `cargo clippy` | Run linter |
| `cargo test` | Run tests |

## Output

The release binary is at:
```
target/release/capcut_guard_rust.exe
```

## Project Structure

```
capcut_guard_rust/
├── src/
│   └── main.rs      # All app code
├── Cargo.toml       # Dependencies
└── target/
    └── release/
        └── capcut_guard_rust.exe
```
