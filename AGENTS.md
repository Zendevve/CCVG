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


### UI/UX Rules (CRITICAL: macOS TAHOE LIQUID GLASS)

> Based on Apple WWDC 2025 Liquid Glass Design System specifications.

#### Philosophy (The Philosophical Triad)
1.  **Clarity**: Eliminate ambiguity. Legibility > Style. Negative space > Borders.
2.  **Deference**: Content is the protagonist. UI recedes (neutral chrome, translucency).
3.  **Depth**: Use Z-axis layering (background → content → controls → modals).

#### Liquid Glass Material System
The material is NOT a single blur — it's a multi-layer composite:

| Layer | Effect | Purpose |
|-------|--------|---------|
| Base | `rgba(255,255,255, 0.08-0.18)` | Frosted foundation |
| Backdrop | `blur(20-50px) saturate(180%)` | Obscures detail, adds vibrancy |
| Noise | 5-10% opacity overlay | Prevents banding |
| Refraction | Inner shadows (top light, bottom dark) | Simulates light bending |
| Specular Border | Gradient stroke (top-left white → transparent) | Rim light simulation |

**Material Variants:**
- `--glass-clear`: 8% white — Immersive overlays, video controls
- `--glass-regular`: 12% white — Default windows, sidebars
- `--glass-thick`: 18% white — High-contrast areas, modals
- `--glass-ultra-thick`: 85% dark gray — Maximum legibility

#### Spatial Architecture
- **8-Point Grid**: All spacing/sizing MUST be multiples of 8 (8, 16, 24, 32...).
- **Touch Targets**: MINIMUM 44×44pt for all interactive elements.
- **Safe Areas**: Use `env(safe-area-inset-*)` for all padding.
- **Squircle Geometry**: Use continuous curvature (G2 continuity).
  - Windows: `26-32px` radius
  - Cards: `20px` radius
  - Buttons: `12px` radius
  - Icon containers: `8px` radius

#### Typography (SF Pro / System Stack)
- **Font Stack**: `-apple-system, BlinkMacSystemFont, "SF Pro Display", "Segoe UI", Roboto, sans-serif`
- **Dynamic Type Scale** (non-linear scaling):
  - Large Title: 34px/41px, Semibold
  - Title 1: 28px/34px, Semibold
  - Body: 17px/22px, Regular
  - Caption: 12px/16px, Regular
- **Dark Mode Elevation**: Lighter backgrounds = closer to user (not shadows).
- **Semantic Colors**: Use CSS variables (`--label-primary`, `--tint-blue`) over hex.
- **Icon Weights**: Stroke weight MUST match adjacent text weight.

#### Motion (Spring Physics)
- **Prefer Springs**: Use `cubic-bezier(0.175, 0.885, 0.32, 1.275)` for bouncy, interruptible animations.
- **Duration Standards**:
  - Instant feedback: 0.1s
  - Fast transitions: 0.2s
  - Normal animations: 0.35s
  - Slow/dramatic: 0.5s
- **Reduced Motion**: Respect `prefers-reduced-motion` — replace springs with cross-fades.

#### Victor Ponamariov's 50 UI Tips (MANDATORY)

**Typography:**
- Put headlines closer to their respective content (proximity law)
- Avoid justifying text — align left to prevent "rivers"
- Use lists instead of long paragraphs
- Make links visually distinct (not just color)
- Line height: 1-1.4× for headlines, 1.3-1.6× for body
- Don't stretch text containers to full width

**Forms:**
- Never hide form tips — show hints below inputs
- Place labels near their inputs (not across columns)
- Use progress indicators for multi-step forms
- Show password rules immediately, highlight as satisfied
- Require fewer fields — delay non-essential validation
- Input width should match expected content length
- Use labels, never rely on placeholders alone
- Don't use dropdowns if options ≤ 5-7 (use radio/buttons)
- Replace default file inputs with custom styled ones
- Autofocus first input
- Autoscroll to first error on large forms

**Validation:**
- Help users fill forms correctly the first time
- Show positive feedback (green checks) not just errors
- Place validation errors next to the relevant input

**Focus & Contrast:**
- Only ONE primary button per logical section
- Overlay dark filter on images with text
- Check contrast: minimum AA level (4.5:1)

**Grouping:**
- Group form elements — labels closer to their inputs
- Keep dangerous actions (delete) visually apart
- Use whitespace to create logical groups

**Visuals:**
- Consider removing borders — use backgrounds/shadows
- Subtle table striping/borders (barely visible)
- Soft shadows only, avoid shadow + border combo
- Show partial content in dropdowns to hint scrollability

**Usability:**
- Put frequently used options on top
- Add "copy to clipboard" for links/tokens
- Avoid complex forms in modals
- Increase clickable areas (use padding)
- Don't rely on dots for gallery navigation
- Consider undo instead of confirmation modals
- Label all icons
- Always provide "what to do next" instructions
- Indicate current location (nav highlights, breadcrumbs)

**Empty/Loading States:**
- Use empty states for engagement, not just "no data"
- Use skeletons for layout, spinners for buttons
- Keep button width consistent during loading (transparent text)
- Don't show loader immediately — wait 0.5-1s
- Smart loaders: "Taking longer than usual..." after 5s

**Misc:**
- Icon consistency: same stroke width, style, fill
- Prefer horizontal nav for simple apps
- Auto-submit short verification codes
- Handle large/long content (truncate, limit, "show more")
- Give visual hints when more content exists (cut-off items)
- Always show post/update dates

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
