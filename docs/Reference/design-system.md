# Design System Reference

> **Source of Truth:** `design.json` (macOS 26 Tahoe Design Tokens)

## Overview

Version Guard uses the macOS 26 Tahoe design system with Laws of UX as the psychological framework. All visual tokens are defined in `design.json` at the project root.

## Color System

### Backgrounds
```css
--bg-window: rgba(30, 30, 32, 0.78);
--bg-titlebar: rgba(38, 38, 40, 0.95);
--bg-content: #232325;
--bg-elevated: #2C2C2E;
```

### Accents
```css
--accent-blue: #007AFF;    /* Primary interactive */
--accent-green: #30D158;   /* Success, toggle on */
--accent-red: #FF453A;     /* Error, destructive */
--accent-orange: #FF9F0A;  /* Warning */
```

### Labels
```css
--label-primary: #FFFFFF;              /* 100% */
--label-secondary: rgba(255,255,255,0.55);
--label-tertiary: rgba(255,255,255,0.25);
```

## Typography

```css
--font-system: -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui;
--font-mono: 'SF Mono', Menlo, Monaco, monospace;

/* Scale */
Body: 13px / 18px, weight 400
Title: 17px / 22px, weight 600
Large Title: 26px / 32px, weight 700
```

## Spacing (4px unit)

```css
--space-1: 4px;
--space-2: 8px;
--space-3: 12px;
--space-4: 16px;
--space-5: 20px;
--space-6: 24px;
--space-8: 32px;
```

## Components

### Buttons
| Variant | Height | Radius | Background |
|---------|--------|--------|------------|
| Primary | 44px | 8px | `--accent-blue` |
| Primary Large | 56px | 8px | `--accent-blue` |
| Secondary | 32px | 8px | `--fill-primary` |
| Plain | auto | - | transparent |

### Toggle Switch
- Width: 38px
- Height: 22px
- Knob: 18px white circle
- Off: `rgba(120,120,128,0.32)`
- On: `--accent-green`

### Glass Panel
```css
background: rgba(60, 60, 65, 0.50);
backdrop-filter: blur(40px) saturate(150%);
border-radius: 10px;
border: 0.5px solid rgba(255,255,255,0.12);
```

## Motion

```css
--ease-out: cubic-bezier(0.0, 0.0, 0.2, 1);
--ease-spring: cubic-bezier(0.175, 0.885, 0.32, 1.275);

--duration-fast: 100ms;
--duration-normal: 200ms;
--duration-slow: 300ms;
```

## Critical Rules

### DO NOT:
- Apply gradients to buttons
- Put backdrop-filter on individual list rows
- Use bright colors for window backgrounds
- Use more than 3 font weights per view
- Exceed 55% opacity for secondary text

### ALWAYS:
- Use rgba for backgrounds
- Apply blur AND saturate together
- Use 0.5px borders (hairline)
- Maintain 44px minimum touch targets
- Apply inner shadow to glass containers
