# Feature: Version Protection

Status: Implemented
Owner: Zendevve
Created: 2024-12-19

---

## Purpose

Protect a user's preferred CapCut version from being overwritten by automatic updates.

---

## Scope

### In scope
- Detect installed CapCut versions
- Allow user to select which version to keep
- Delete unwanted version folders
- Lock configuration files to read-only
- Create blocker files/folders to prevent updates

### Out of scope
- Downloading CapCut installers directly (links to ProjectBukkit/CapcutVersions instead)
- macOS support

---

## Business Rules

- Only one version can be kept; others are deleted
- User must confirm before deletion occurs
- CapCut must not be running during protection

---

## User Flows

### Primary Flow: Protect Existing Installation

1. User launches app
2. Clicks "Protect Existing Installation"
3. App scans for CapCut → shows PreCheck screen
4. If CapCut found and not running → shows VersionSelect
5. User selects version to keep
6. Clicks "Apply Protection"
7. App deletes other versions, locks config, creates blockers
8. Shows Complete screen

### Alternative Flow: Download Legacy Version

1. User clicks "Download Legacy Version"
2. App shows Download Manager with persona cards
3. User selects a persona (Offline Purist / Audio Engineer / Creator)
4. Clicks "Download from Uptodown" → opens browser
5. After installing, user returns and uses "Protect Existing Installation" flow

---

## System Behaviour

- Entry points: Desktop GUI (eframe/egui)
- Reads from: LOCALAPPDATA\CapCut directory
- Writes to: Same directory (deletes folders, modifies file permissions)
- Side effects: Creates updater.exe.bak, CapCutUpdater.bak folders
- Error handling: Shows Error screen with message

---

## Verification

### Test environment
- Windows 10/11 with CapCut installed
- Multiple versions in Apps/ folder for testing

### Test commands
- Build and run validation is handled through internal QA workflows.

### Test flows

**Positive scenarios**
| ID | Description | Expected result |
| --- | --- | --- |
| POS-001 | Happy path with multiple versions | User selects one, others deleted, config locked |
| POS-002 | Single version installed | Version selected by default, protection applies |

**Negative scenarios**
| ID | Description | Expected result |
| --- | --- | --- |
| NEG-001 | CapCut not installed | PreCheck shows "not found" with Re-Check button |
| NEG-002 | CapCut running | PreCheck shows warning, blocks Apply until closed |

---

## Definition of Done

- [x] Wizard flow implemented (Welcome → PreCheck → VersionSelect → Running → Complete)
- [x] Version scanning works
- [x] Version deletion works
- [x] Config locking works
- [x] Blocker creation works
- [x] Error handling displays errors
- [x] UI is responsive and professional
