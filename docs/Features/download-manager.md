# Feature: Download Manager

Status: Implemented
Owner: Zendevve
Created: 2024-12-19

---

## Purpose

Guide users to download legacy CapCut versions from Uptodown by presenting curated "persona" options.

---

## Scope

### In scope
- Display 3 persona cards with recommended versions
- Link to Uptodown versions page
- Show version features and risk levels

### Out of scope
- Direct in-app downloads
- Version verification/hashing

---

## Business Rules

- Personas represent use-case archetypes:
  - **Offline Purist** (v1.5.0): Clean UI, 4K export, zero cloud
  - **Audio Engineer** (v2.5.4): Multi-track audio, stable mixer
  - **Creator** (v3.9.0): Last free Auto-Captions (API risk)
- All download buttons link to: `https://capcut.en.uptodown.com/windows/versions`
- High-risk versions show warning icon

---

## User Flows

### Primary Flow

1. User clicks "Download Legacy Version" on Welcome screen
2. Download Manager shows 3 persona cards
3. User clicks a card to select it
4. User clicks "Download from Uptodown"
5. Browser opens to Uptodown versions page
6. User downloads and installs desired version
7. User returns to app and uses "Skip - Protect existing installation"

---

## UI Requirements

- Cards have dark background with accent border when selected
- Icon color changes to green when selected
- Feature badges show with checkmarks
- Risk warnings show amber warning icon
- Responsive layout with dynamic spacing

---

## Definition of Done

- [x] 3 persona cards displayed
- [x] Cards are clickable and show selection state
- [x] Download button opens Uptodown in browser
- [x] Skip link navigates to PreCheck flow
- [x] Back link returns to Welcome
- [x] Proper contrast on selected cards
- [x] Responsive layout
