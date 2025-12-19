# AGENTS.md

CC Version Guard — Rust/eframe Desktop Application

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

- build: `cargo build --release`
- test: `cargo test`
- run: `cargo run --release`
- format: `cargo fmt`
- lint: `cargo clippy`

### Task Delivery (ALL TASKS)

- Read AGENTS.md and docs before planning
- Write multi-step plan before implementation
- Implement code and tests together
- Run tests, format, and build yourself — do NOT ask user to execute them
- Summarize changes before marking complete

### Documentation (ALL TASKS)

- All docs live in `docs/`
- Update feature docs when behaviour changes
- Update ADRs when architecture changes
- Templates: `docs/templates/ADR-Template.md`, `docs/templates/Feature-Template.md`

### Testing

- Desktop GUI app — manual testing via `cargo run --release`
- Verify all UI flows work before committing
- Test file system operations on real CapCut installations

### Code Style

- Rust 2021 edition
- No magic literals — extract to constants
- Use egui-phosphor icons, never ASCII/Unicode symbols
- Follow 60-30-10 color rule for UI theming

### External Resources (Mandatory)

- **Download Source**: ALWAYS use `https://github.com/ProjectBukkit/CapcutVersions` for legacy downloads. Do NOT use Uptodown.

### UI/UX Rules (CRITICAL)

- **Responsive layouts** — use dynamic spacing based on available_height/available_width
- **Max content width** — constrain to 500px for readability on large screens
- **Never require scrolling** on primary screens unless content is truly long
- **Proper contrast** — selected items use accent BORDER, not accent FILL (keeps text readable)
- **Click detection** — use frame_response.response.rect for card interactions

### Critical (NEVER violate)

- Never commit secrets or API keys
- Never ship without testing the built exe
- Never ignore user frustration signals — add emphatic rules immediately

### Boundaries

**Always:**
- Read AGENTS.md before editing code
- Build and run before committing

**Ask first:**
- Adding new dependencies
- Changing wizard flow structure
- Deleting features

---

## Preferences

### Likes
- Professional, sleek, corporate UI aesthetic
- Phosphor icons
- Wizard-style guided flows
- Responsive layouts that adapt to window size

### Dislikes
- Bland, basic, amateurish UI
- Unicode symbols that render as squares
- Layouts requiring scroll for basic content
- Poor contrast on selected items
