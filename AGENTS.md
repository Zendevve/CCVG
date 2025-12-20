# AGENTS.md

CC Version Guard — Tauri v2 Desktop Application

Follows [MCAF](https://mcaf.managed-code.com/)

---

## Conversations (Self-Learning)

Learn the user's habits, preferences, and working style. Extract rules from conversations, save to "## Rules to follow", and generate code according to the user's personal rules.

**Update requirement (core mechanism):**

Before doing ANY task, evaluate the latest user message.
If you detect a new rule, correction, preference, or change → update `AGENTS.md` first.
Only after updating the file you may produce the task output.
If no new rule is detected → do not update the file.

---

## Rules to follow (Mandatory, no exceptions)

### Commands

- build: `npm run tauri build`
- dev: `npm run tauri dev`
- cargo build: `cargo build --release` (in src-tauri/)
- cargo test: `cargo test` (in src-tauri/)
- format: `cargo fmt` (in src-tauri/)
- lint: `cargo clippy` (in src-tauri/)

### Task Delivery (ALL TASKS)

- Read AGENTS.md and docs before planning
- Write multi-step plan before implementation
- Implement code and tests together
- Run builds yourself — do NOT ask user to execute them
- Summarize changes before marking complete

### Documentation (ALL TASKS)

- All docs live in `docs/`
- Update feature docs when behaviour changes
- Update ADRs when architecture changes
- Templates: `docs/templates/ADR-Template.md`, `docs/templates/Feature-Template.md`

### Testing

- Desktop GUI app — manual testing via `npm run tauri dev`
- Verify all wizard screens work before committing
- Test file system operations on real CapCut installations

### Project Structure

```
capcut_guard_tauri/
├── src/                         # Frontend (Vanilla JS)
│   ├── index.html              # Wizard screens
│   ├── styles.css              # Midnight Obsidian theme
│   ├── main.js                 # Wizard logic & Tauri IPC
│   └── assets/                 # Static assets
├── src-tauri/                  # Backend (Rust)
│   ├── src/
│   │   ├── main.rs             # Entry point
│   │   ├── lib.rs              # Tauri builder & command registration
│   │   └── commands/           # Tauri commands (modular)
│   │       ├── mod.rs
│   │       ├── scanner.rs      # Version scanning
│   │       ├── process.rs      # Process detection
│   │       ├── cleaner.rs      # Cache cleaning
│   │       ├── protector.rs    # File locking
│   │       └── switcher.rs     # Version switching
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── README.md
```

### Code Style

- Rust 2021 edition for backend
- Vanilla JavaScript (no framework) for frontend
- Use Phosphor Icons (web CDN) — never ASCII/Unicode symbols
- Follow 60-30-10 color rule for UI theming
- CSS variables for all design tokens

### External Resources (Mandatory)

- **Download Source**: ALWAYS use official ByteDance CDN URLs for legacy downloads
- **Icons**: Use Phosphor Icons from `unpkg.com/@phosphor-icons/web`

### UI/UX Rules (CRITICAL)

- **Wizard-only pattern** — step-by-step guided flow (NO dashboard for now)
- **Feature parity with legacy** — match original eframe/egui app exactly
- **Hick's Law** — ONE action per screen, minimize choices
- **Jakob's Law** — Follow familiar setup wizard patterns
- **Large touch targets** — Buttons min 44px height
- **Responsive layouts** — max content width 480px
- **Visual feedback** — hover states, transitions, loading spinners
- **Progress tracking** — clear indication of current step

### Tauri-Specific Rules

- **Commands in modules** — each feature in its own `commands/*.rs` file
- **Serialize all data** — use serde for IPC between frontend and backend
- **Error handling** — return Result types, handle in frontend
- **No Node.js** — frontend is pure browser JavaScript

### Critical (NEVER violate)

- Never commit secrets or API keys
- Never ship without testing the built exe
- Never force push to main
- Never approve or merge (human decision)
- **Never block core application functionality** (e.g., effects, asset downloads). Blocking must be scoped strictly to auto-update mechanisms.
- **ALWAYS strictly follow MCAF phases** (Planning -> Execution -> Verification). Never skip directly to coding.
- Never ignore user frustration signals — add emphatic rules immediately

### Boundaries

**Always:**
- Read AGENTS.md before editing code
- Build and run before committing

**Ask first:**
- Adding new npm dependencies
- Adding new Rust crates
- Changing wizard flow structure
- Deleting features

---

## Preferences

### Likes
- Professional, sleek, corporate UI aesthetic
- Phosphor icons (web version)
- Wizard-style guided flows
- Responsive layouts
- Modular Rust code structure

### Dislikes
- Bland, basic, amateurish UI
- Unicode symbols that render incorrectly
- Layouts requiring scroll for basic content
- Monolithic single-file Rust code
- npm dependencies when vanilla JS works
