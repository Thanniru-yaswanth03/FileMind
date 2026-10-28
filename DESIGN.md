---
name: FileMind Design System
description: Premium desktop utility design system for storage intelligence and safety planning
colors:
  bg-ground: "#FBF9F5"
  bg-surface-1: "#F3EFEA"
  bg-surface-2: "#EAE4DC"
  bg-surface-hover: "#E3DDD4"
  bg-surface-active: "#D9D2C8"
  border-subtle: "#E5DFD4"
  border-strong: "#D1C9BC"
  border-focus: "#4A6B5B"
  text-primary: "#1F2022"
  text-secondary: "#5A5C61"
  text-muted: "#82858B"
  text-inverse: "#FBF9F5"
  accent-base: "#4A6B5B"
  accent-hover: "#3D584B"
  accent-active: "#31463C"
  accent-subtle: "#EBF0EC"
  accent-border: "#9CB2A6"
  danger-base: "#B4533C"
  danger-hover: "#98432E"
  danger-active: "#7E3523"
  danger-subtle: "#F8EDE9"
  danger-border: "#D8A598"
  tier-safe: "#4A6B5B"
  tier-caution: "#C08A3E"
  tier-high: "#B4533C"
  tier-danger: "#9E2A2B"
typography:
  display:
    fontFamily: "Segoe UI Variable Display, Segoe UI, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "-0.025em"
  headline:
    fontFamily: "Segoe UI Variable Text, Segoe UI, sans-serif"
    fontSize: "1rem"
    fontWeight: 600
    lineHeight: 1.375
    letterSpacing: "-0.02em"
  body:
    fontFamily: "Segoe UI Variable Text, Segoe UI, sans-serif"
    fontSize: "0.8125rem"
    fontWeight: 400
    lineHeight: 1.385
    letterSpacing: "0em"
  caption:
    fontFamily: "Segoe UI Variable Text, Segoe UI, sans-serif"
    fontSize: "0.75rem"
    fontWeight: 400
    lineHeight: 1.33
    letterSpacing: "0em"
  mono:
    fontFamily: "Cascadia Code, Consolas, monospace"
    fontSize: "0.75rem"
    fontWeight: 400
    lineHeight: 1.33
    letterSpacing: "0em"
rounded:
  sm: "3px"
  md: "4px"
  lg: "6px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
components:
  button-primary:
    backgroundColor: "{colors.accent-base}"
    textColor: "{colors.text-inverse}"
    rounded: "{rounded.md}"
    padding: "6px 14px"
  button-primary-hover:
    backgroundColor: "{colors.accent-hover}"
    textColor: "{colors.text-inverse}"
    rounded: "{rounded.md}"
    padding: "6px 14px"
  button-secondary:
    backgroundColor: "{colors.bg-surface-1}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.md}"
    padding: "6px 12px"
  button-danger:
    backgroundColor: "{colors.danger-base}"
    textColor: "{colors.text-inverse}"
    rounded: "{rounded.md}"
    padding: "6px 14px"
  badge-safe:
    backgroundColor: "{colors.accent-subtle}"
    textColor: "{colors.accent-active}"
    rounded: "{rounded.sm}"
    padding: "2px 6px"
  badge-danger:
    backgroundColor: "{colors.danger-subtle}"
    textColor: "{colors.danger-active}"
    rounded: "{rounded.sm}"
    padding: "2px 6px"
---

# FileMind Design System

## Overview
FileMind is designed as premium desktop utility software that prioritizes information density, precision, and earned trust. Rejecting generic AI dashboard clichés (neon glows, purple gradients, glassmorphism, floating AI orbs), FileMind embraces a calm, paper-neutral visual world inspired by technical drafting tools, high-craft instruments, and archival documentation.

## Colors
The color palette relies on warm ivory paper neutrals as the canvas, high-contrast graphite for typography, muted sage green as the restrained primary accent, and muted terracotta for alerts and destructive operations.

- **Background Ground (`#FBF9F5`):** Warm ivory paper ground reducing visual fatigue.
- **Surface Layer 1 (`#F3EFEA`):** Slightly elevated container ground for navigation rails and sidebars.
- **Surface Layer 2 (`#EAE4DC`):** Toolbars, property panels, and table headers.
- **Hairline Borders (`#E5DFD4` / `#D1C9BC`):** Crisp structural boundaries providing visual stability without heavy shadows.
- **Primary Text (`#1F2022`):** Deep graphite ink ensuring WCAG AAA contrast ratio (>13:1).
- **Muted Sage Green (`#4A6B5B`):** Earthy, restrained accent for primary actions, selected rows, and verified safe indicators.
- **Muted Terracotta (`#B4533C`):** Disciplined warm rust accent for warnings, high-risk warnings, and destructive gates.

Dark mode mirrors this exact semantic structure using a warm charcoal ground (`#18191A`), elevated panels (`#222325`), warm ivory text (`#EAE7E1`), and soft sage (`#628975`).

## Typography
FileMind uses the Windows system workhorse sans stack (`"Segoe UI Variable Text", "Segoe UI", "Inter", sans-serif`) for all interface text, and `"Cascadia Code", "Consolas", monospace` strictly for paths, hashes, and code.

- **Tabular Numerals Invariant:** Every numeric quantity (byte sizes, file counts, percentages, dates) MUST specify `font-variant-numeric: tabular-nums` to guarantee vertical column alignment in data tables.
- **Tight Type Scale:** Steps scale by a restrained 1.125–1.2 ratio (11px, 12px, 13px, 14px, 16px, 20px). Exaggerated display sizes are prohibited.
- **No Kickers:** Section titles and cards stand on their own without diminutive eyebrow labels.

## Layout
The application layout follows classic desktop utility conventions:
- **Navigation Rail (Left, 200px):** Overview, Storage Visualizer, File Index, Duplicates, Workspaces, Plans, Quarantine Vault, Settings.
- **Main Canvas (Center, Flexible):** High-density virtualized data tables, interactive treemaps, or sunburst charts.
- **Property Inspector (Right, 320px Collapsible):** Selected file details, ACL security descriptors, BLAKE3 hashes, and safety score rationale.
- **Status Bar (Bottom, 28px):** Disk capacity meters, active scan throughput, worker thread indicators.

## Elevation & Depth
- Elevation is communicated primarily through subtle 1px hairline borders (`#E5DFD4`) and surface color shifts rather than heavy drop shadows.
- When floating menus or tooltips appear, they use soft, directional shadows with ambient blur: `box-shadow: 0 4px 12px rgba(31, 32, 34, 0.08)`.
- Hard offset block shadows (`4px 4px 0`) and zero-offset glowing halos are strictly banned.

## Shapes
- Border radii are disciplined and restrained: `3px` for badges, `4px` for buttons, inputs, and list items, `6px` for floating tooltips.
- Oversized pill shapes, floating circular bubbles, and heavily rounded cards (`16px+`) are prohibited.

## Components
- **High-Density Virtualized Table:** 28px row height, sortable headers, tabular numbers, subtle hover states, zero alternating row noise.
- **Treemap & Sunburst Visualizer:** Categorical fills mapped to sage green, slate, warm kraft, and terracotta; 1px cell dividers.
- **Safety Tier Badges:** Compact micro-labels (`11px`) indicating Safe (Sage), Caution (Amber), High Risk (Terracotta), and Danger (Crimson).
- **The Parts You Did Not Draw:** System scrollbars, text selection (`#DCE5E0`), and keyboard focus rings (`0 0 0 2px #FBF9F5, 0 0 0 4px #4A6B5B`) are explicitly themed.

## Do's and Don'ts

### Do
- Use tabular numerals (`font-variant-numeric: tabular-nums`) for all numbers and sizes.
- Rely on inline progressive disclosure and property drawers instead of full-screen modals.
- Provide keyboard shortcuts (`Ctrl+F`, `Arrows`, `Space`, `Enter`, `Esc`) for all core workflows.
- Maintain minimum 4.5:1 text contrast on all surfaces.
- Use monospace fonts strictly for file paths, hashes, and code snippets.

### Don't
- Do not use generic dark-navy backgrounds with purple/cyan gradients.
- Do not use glowing orbs, pulse animations, or floating AI avatars.
- Do not use kicker/eyebrow labels above headings.
- Do not use nested card containers (cards inside cards).
- Do not use gradient text or glassmorphic blur effects.
- Do not use Unicode glyphs or emoji as icons; use consistent SVG line icons.

