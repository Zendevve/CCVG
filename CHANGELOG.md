# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.3.0] - 2025-12-20

### 🎨 Complete UI/UX Overhaul

This release introduces a complete redesign of the user interface using the **Laws of UX** psychological framework and **macOS 26 Tahoe design system**.

### Added
- **Laws of UX Framework** — Psychologically optimized user experience
  - Fitts's Law: 44px+ minimum touch targets
  - Von Restorff Effect: Distinctive primary CTAs with glow shadows
  - Hick's Law: One primary action per view
  - Peak-End Rule: Celebration animations on success
  - Zeigarnik Effect: Progress indicators and status cards
- **Keyboard Accessibility** — All toggles respond to Enter/Space keys
- **Actionable Empty States** — "No installations found" now includes a CTA button
- **Improved Error Recovery** — "Start Over" and "Change Options" buttons with clear actions
- **`design.json`** — Single source of truth for all design tokens

### Changed
- **Complete CSS Rewrite** — 729 lines using exact design.json tokens
- **"Done" Button** — Now returns to welcome screen instead of closing app
- **Error View** — Renamed buttons with icons for clarity
- **AGENTS.md** — Replaced Apple HIG with Laws of UX framework

### Fixed
- JavaScript syntax error that was breaking all button handlers
- Toggle switches now properly update `aria-checked` attribute
- Switch button resets to disabled state after successful switch
- `navigateTo` and `goBack` functions now accessible to inline handlers

### Documentation
- Added ADR-003: Laws of UX Design Framework
- Added `docs/Reference/design-system.md`
- Updated README.md with new design philosophy
- Created comprehensive CHANGELOG

---

## [2.2.0] - 2025-12-19

### Added
- Quick Switch feature for changing versions without re-protecting
- Version removal protection enhancements

### Fixed
- Various UI polish improvements

---

## [2.1.0] - 2025-12-18

### Added
- Legacy version download manager
- Curated list of 6 legacy versions with persona recommendations

---

## [2.0.0] - 2025-12-17

### Added
- Complete rewrite in Tauri v2
- Modern glass-morphism UI
- Step-by-step wizard flow

### Changed
- Migrated from pure Rust CLI to Tauri desktop app
