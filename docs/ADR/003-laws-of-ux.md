# ADR-003: Laws of UX Design Framework

## Status
**Accepted** — 2025-12-20

## Context
The application needed a complete UI/UX overhaul to achieve a "first-party app" feel and focus on conversion. The previous design followed Apple HIG / Liquid Glass concepts, but lacked a cohesive psychological framework for user experience decisions.

## Decision
Adopt **Laws of UX** as the psychological framework and **macOS 26 Tahoe Design System** (`design.json`) as the visual token source.

### Laws of UX Applied

| Category | Laws |
|----------|------|
| Perception | Aesthetic-Usability Effect, Law of Prägnanz |
| Decisions | Hick's Law, Choice Overload |
| Interaction | Fitts's Law, Doherty Threshold |
| Grouping | Chunking, Proximity, Common Region, Similarity |
| Memory | Miller's Law, Von Restorff, Serial Position, Selective Attention |
| Progress | Zeigarnik Effect, Goal-Gradient, Peak-End Rule |
| Familiarity | Jakob's Law, Mental Models |
| Efficiency | Tesler's Law, Occam's Razor, Pareto Principle |

### design.json Token Categories

- Colors (backgrounds, fills, labels, accents, borders)
- Typography (SF Pro system stack, 13px body, 22px title)
- Spacing (4px unit system)
- Radii (12px window, 10px panel, 8px button)
- Shadows (window, elevated, card, button)
- Animations (spring physics, 100-300ms durations)

## Consequences

**Positive:**
- Consistent, predictable user experience
- Psychologically optimized for conversion
- Clear documentation for future development
- Accessibility improvements (keyboard support, ARIA attributes)

**Negative:**
- Requires adherence to strict design tokens
- Less flexibility for ad-hoc styling decisions

## References
- [Laws of UX](https://lawsofux.com/)
- [design.json](../Reference/design-system.md)
- [AGENTS.md](../../AGENTS.md)
