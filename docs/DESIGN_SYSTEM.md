# FileMind — Design System & Visual Specification

**Document Version:** 1.0.0  
**Design Standard:** Impeccable (Operate Mode)  
**Surface Mode:** Desktop Utility & Data Density  
**Status:** Approved Specification  

---

## 1. Design Philosophy & Impeccable Integration

FileMind is designed as **premium desktop utility software**. It intentionally departs from prevailing web and AI software tropes: it is not a dark navy dashboard, does not use purple-to-cyan gradient glows, rejects floating holographic orbs, has no glassmorphism, and does not simulate terminal windows.

Instead, FileMind draws inspiration from high-craft physical instruments, technical drafting tools, and paper ephemera:
- **Calm, High-Contrast Ground:** Warm ivory and paper-neutral surfaces that reduce visual fatigue during prolonged storage audits.
- **Graphic Precision:** High-contrast graphite and warm charcoal typography with strict tabular numerals for alignment.
- **Restrained Accents:** Muted sage green as an earthy, disciplined primary accent; muted terracotta for alerts, destructive warnings, and deletion gates.
- **Desktop Density:** Compact tables, purposeful whitespace, visible borders, and immediate discoverability of information over decorative padding.

### Impeccable Skill Activation & Quality Floor
The design system was verified against the **Impeccable** engine located at `C:\Users\batma\.gemini\config\skills\impeccable\`. Operating in **Operate Mode** (governing software where users are inside a focused task), the interface adheres to the following strict quality invariants:
- **No Kickers or Eyebrows:** Headers stand on their own without diminutive labels above them.
- **No Nested Cards:** Information grouping relies on subtle hairline dividers, borders, and spatial rhythm rather than multiple stacked card containers.
- **No Gradient Text or Fake Neobrutalism:** Emphasis is achieved through weight and scale, not multi-color text gradients or heavy 4px block shadows.
- **The Parts You Did Not Draw:** System scrollbars, text selection colors, focus rings, and caret colors are themed explicitly to match the palette.
- **Tabular Figures:** All measurements, byte sizes, file counts, and timestamps enforce `font-variant-numeric: tabular-nums` to maintain vertical alignment in tables.

---

## 2. Color System & Semantic Tokens

```
Light Mode (Default: Warm Ivory & Paper Neutrals)
┌─────────────────────────────────────────────────────────────┐
│ Background Ground:  #FBF9F5 (Warm Ivory Paper)              │
│ Surface Layer 1:    #F3EFEA (Subtle Container Ground)       │
│ Surface Layer 2:    #EAE4DC (Elevated Panels & Toolbars)    │
│ Hairline Borders:   #DCD5CA (1px Subtle Boundary)           │
│ Primary Text:       #1F2022 (Graphite Ink)                  │
│ Accent Primary:     #4A6B5B (Muted Sage Green)              │
│ Accent Destructive: #B4533C (Muted Terracotta)              │
└─────────────────────────────────────────────────────────────┘

Dark Mode (Complementary Warm Charcoal)
┌─────────────────────────────────────────────────────────────┐
│ Background Ground:  #18191A (Deep Warm Charcoal)            │
│ Surface Layer 1:    #222325 (Panel Ground)                  │
│ Surface Layer 2:    #2B2D30 (Elevated Dropdowns & Toolbars) │
│ Hairline Borders:   #383B3E (Subtle Dark Boundary)          │
│ Primary Text:       #EAE7E1 (Warm Ivory Off-White)          │
│ Accent Primary:     #628975 (Soft Sage Green)               │
│ Accent Destructive: #C86953 (Soft Terracotta)               │
└─────────────────────────────────────────────────────────────┘
```

### 2.1 CSS Variables & Palette Tokens

```css
:root {
  /* Light Theme (Warm Ivory Paper) */
  --bg-ground: #FBF9F5;
  --bg-surface-1: #F3EFEA;
  --bg-surface-2: #EAE4DC;
  --bg-surface-hover: #E3DDD4;
  --bg-surface-active: #D9D2C8;

  --border-subtle: #E5DFD4;
  --border-strong: #D1C9BC;
  --border-focus: #4A6B5B;

  --text-primary: #1F2022;
  --text-secondary: #5A5C61;
  --text-muted: #82858B;
  --text-inverse: #FBF9F5;

  /* Primary Accent: Muted Sage Green */
  --accent-base: #4A6B5B;
  --accent-hover: #3D584B;
  --accent-active: #31463C;
  --accent-subtle: #EBF0EC;
  --accent-border: #9CB2A6;

  /* Destructive & Warning Accent: Muted Terracotta */
  --danger-base: #B4533C;
  --danger-hover: #98432E;
  --danger-active: #7E3523;
  --danger-subtle: #F8EDE9;
  --danger-border: #D8A598;

  /* Safety Tier Indicators */
  --tier-safe-bg: #EBF0EC;
  --tier-safe-text: #2D4C3E;
  --tier-safe-border: #B2C7BC;

  --tier-caution-bg: #FBF4E6;
  --tier-caution-text: #6B4E1A;
  --tier-caution-border: #E0CFA6;

  --tier-high-bg: #F8EDE9;
  --tier-high-text: #8E3520;
  --tier-high-border: #D8A598;

  --tier-danger-bg: #FCE8E6;
  --tier-danger-text: #961D18;
  --tier-danger-border: #E59B96;

  /* The Parts You Did Not Draw */
  --selection-bg: #DCE5E0;
  --selection-text: #1F2022;
  --scrollbar-thumb: #D5CEBF;
  --scrollbar-thumb-hover: #BFB7A6;
  --focus-ring: 0 0 0 2px #FBF9F5, 0 0 0 4px #4A6B5B;
}

[data-theme="dark"] {
  /* Dark Theme (Warm Charcoal) */
  --bg-ground: #18191A;
  --bg-surface-1: #222325;
  --bg-surface-2: #2B2D30;
  --bg-surface-hover: #34363A;
  --bg-surface-active: #3E4145;

  --border-subtle: #2E3034;
  --border-strong: #42454B;
  --border-focus: #628975;

  --text-primary: #EAE7E1;
  --text-secondary: #B0AEA8;
  --text-muted: #7E7C77;
  --text-inverse: #18191A;

  /* Primary Accent: Soft Sage Green */
  --accent-base: #628975;
  --accent-hover: #759F8A;
  --accent-active: #88B49F;
  --accent-subtle: #242E28;
  --accent-border: #4A6355;

  /* Destructive & Warning: Soft Terracotta */
  --danger-base: #C86953;
  --danger-hover: #D77A64;
  --danger-active: #E48D78;
  --danger-subtle: #332320;
  --danger-border: #6B3C32;

  /* Safety Tier Indicators (Dark Mode) */
  --tier-safe-bg: #202D26;
  --tier-safe-text: #9CCEB6;
  --tier-safe-border: #3D5A4B;

  --tier-caution-bg: #302A1F;
  --tier-caution-text: #E5C384;
  --tier-caution-border: #615235;

  --tier-high-bg: #332320;
  --tier-high-text: #E89582;
  --tier-high-border: #6B3C32;

  --tier-danger-bg: #381D1C;
  --tier-danger-text: #F08985;
  --tier-danger-border: #7B3330;

  --selection-bg: #2C3E34;
  --selection-text: #EAE7E1;
  --scrollbar-thumb: #3E4145;
  --scrollbar-thumb-hover: #50545A;
  --focus-ring: 0 0 0 2px #18191A, 0 0 0 4px #628975;
}
```

---

## 3. Typography System

Product UI prioritizes clarity, legibility at small sizes, and alignment over expressive display lettering.

### 3.1 Font Families
- **UI & Body Sans:** System Workhorse Stack:
  `"Segoe UI Variable Text", "Segoe UI", -apple-system, BlinkMacSystemFont, "Inter", sans-serif`
- **Monospace (Data, Hashes, Paths):**
  `"Cascadia Code", "Consolas", "Courier New", monospace`

### 3.2 Type Scale
| Token | Size | Line Height | Weight | Letter Spacing | Usage |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `text-micro` | 11px | 14px | 500 | +0.02em | Badges, status tags, column indicators |
| `text-caption`| 12px | 16px | 400 | normal | Table cells, secondary path breadcrumbs |
| `text-body` | 13px | 18px | 400 | normal | Default interface text, list items |
| `text-body-strong` | 13px | 18px | 600 | normal | Active selections, table headers, labels |
| `text-ui-label` | 14px | 20px | 500 | -0.01em | Button labels, input values, card titles |
| `text-subhead` | 16px | 22px | 600 | -0.02em | Section headers, panel titles |
| `text-headline`| 20px | 26px | 600 | -0.025em | Main view titles, dialog headlines |

*Rule:* All numerical quantities (file sizes, item counts, durations, percentages, dates) MUST specify:
```css
font-variant-numeric: tabular-nums;
```

---

## 4. Layout Architecture & Component Hierarchy

### 4.1 Application Shell Anatomy
```
┌────────────────────────────────────────────────────────────────────────┐
│ App Titlebar & Breadcrumbs (Height: 40px)                              │
├──────────────┬──────────────────────────────────────────┬──────────────┤
│ Navigation   │ Main Workspace View                     │ Property     │
│ Rail         │ (Treemap / Sunburst / Virtualized Table) │ Inspector    │
│              │                                          │ Panel        │
│ • Overview   │                                          │              │
│ • Visualizer │                                          │ • Metadata   │
│ • Index      │                                          │ • Safety     │
│ • Duplicates │                                          │ • Hashes     │
│ • Workspace  │                                          │ • History    │
│ • Plans      │                                          │              │
│ • Quarantine │                                          │              │
│ • Copilot    │                                          │              │
│ (Width: 200) │ (Flexible Main Canvas)                   │ (Width: 320) │
├──────────────┴──────────────────────────────────────────┴──────────────┤
│ Status Bar: Disk Usage | Current Scan Rate | Active Worker (Height: 28) │
└────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Component Design Specifications

#### High-Density Data Table
- **Row Height:** 28px compact; 36px standard.
- **Header:** 32px height, subtle bottom border (`1px solid var(--border-subtle)`), uppercase 11px text with sorting glyphs.
- **Alternating Rows:** Disabled; visual hierarchy is preserved via subtle hover (`var(--bg-surface-hover)`) and active focus outline.
- **Virtualization:** Backed by `@tanstack/react-virtual` to ensure zero FPS drop with 100,000+ rows.

#### Treemap & Sunburst Visualization
- **Palette Mapping:** Categorical fills mapped to the neutral and sage palette:
  - Code & Dev: `#4A6B5B` (Sage)
  - Binaries & Executables: `#3D584B` (Dark Sage)
  - Media & Video: `#82998C` (Slate Sage)
  - Archives & Compressed: `#B6A68F` (Warm Kraft)
  - Temporary & Caches: `#D8A598` (Muted Terracotta)
- **Cell Padding:** 1px gap between rectangles; cells under 12px width hide text labels to avoid clutter.

#### Safety Tier Badges
```html
<span class="badge badge-safe">Safe to Clear</span>
<span class="badge badge-caution">Caution (Inactive Cache)</span>
<span class="badge badge-high">High Risk (Uncommitted Git)</span>
<span class="badge badge-danger">Blocked (System Path)</span>
```
- **Styling:** Micro-typography (11px), 2px vertical padding, 6px horizontal padding, 3px border-radius, hairline border. No glowing icons or pulse animations.

#### Action Ribbon & Button Hierarchy
- **Primary Button (Commit / Execute):**
  `background: var(--accent-base); color: #FFF; border-radius: 4px; padding: 6px 14px;`
- **Secondary Button (Cancel / Inspect):**
  `background: var(--bg-surface-1); border: 1px solid var(--border-strong); color: var(--text-primary);`
- **Destructive Button (Purge / Quarantine):**
  `background: var(--danger-base); color: #FFF; border-radius: 4px;`

#### Refusing Unnecessary Modals
Following Impeccable's *Operate Mode* guidance, FileMind rejects intrusive full-screen modal overlays for routine tasks. Instead:
- **Cleanup Plan Confirmations:** Render inline as an expandable review pane within the main workflow.
- **File Details:** Render inside the persistent, collapsible right Property Inspector.
- **Quarantine Rollback:** Executes via inline action rows with toast undo prompts.

---

## 5. Interaction & Motion Rules

1. **Duration Budget:** All UI state transitions (drawer toggles, hover states, filter changes) must complete within **150ms to 200ms**.
2. **Timing Function:** Exponential ease-out (`cubic-bezier(0.16, 1, 0.3, 1)`).
3. **No Decorative Entrances:** No staged staggered fading on initial app launch; the utility must render into a functional state immediately upon startup.
4. **Keyboard Invariant:**
   - `Ctrl + F`: Jump focus to global FTS5 search.
   - `Arrow Up / Down`: Navigate virtualized file rows.
   - `Space`: Toggle item selection for cleanup plan.
   - `Enter`: Open details inspector.
   - `Esc`: Clear filter or collapse active drawer.

