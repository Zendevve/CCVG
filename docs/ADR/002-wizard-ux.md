# ADR-002: Wizard-Based UX Pattern

Status: Accepted
Date: 2024-12-19

## Context

The app performs a multi-step process:
1. Check system state
2. Select version
3. Apply protection
4. Show results

Need a UX pattern that guides users through this flow.

## Decision

Use a **wizard pattern** with distinct screens:
- Welcome
- DownloadManager (optional path)
- PreCheck
- VersionSelect
- Running
- Complete
- Error

## Consequences

### Positive
- Clear step-by-step guidance
- Users can't skip critical checks
- Each screen has focused purpose

### Negative
- More screens to maintain
- Navigation state must be managed
