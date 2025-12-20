# ADR-001: GUI Framework Choice

Status: Accepted
Date: 2024-12-19

## Context

Need a cross-platform desktop GUI framework for a Windows utility app that:
- Has a modern, professional appearance
- Compiles to a single executable
- Doesn't require runtime dependencies

## Decision

Use **eframe/egui** (Rust) with **egui-phosphor** for icons.

## Consequences

### Positive
- Single executable, no installer needed
- Immediate mode GUI is simple to implement
- Professional icon font included
- Easy theming with Color32

### Negative
- Immediate mode has limitations for complex UIs
- No native Windows widgets
- Learning curve for Rust
