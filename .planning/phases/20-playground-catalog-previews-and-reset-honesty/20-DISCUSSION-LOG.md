# Phase 20: Playground catalog previews and Reset honesty - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-20
**Phase:** 20-playground-catalog-previews-and-reset-honesty
**Mode:** Yolo
**Areas discussed:** Catalog preview medium, Catalog layout inside the Kobalte shell, Reset live-control honesty, Focused Chromium smoke

[auto-select] Selected all gray areas: Catalog preview medium, Catalog layout inside the Kobalte shell, Reset live-control honesty, Focused Chromium smoke.

---

## Catalog preview medium

| Option | Description | Selected |
|--------|-------------|----------|
| Compact static inline SVG/CSS in DemoNavigation | Restore WEB-01 illustrations without a WASM world per entry; matches Phase 18 D-02 | ✓ |
| Reuse README WebP gallery stills as in-app images | ROADMAP forbids treating documentation WebPs as the in-app substitute | |
| Live WASM or Canvas thumbnails | Phase 18 D-02 banned; would multiply sessions | |

**User's choice:** Compact static inline SVG/CSS in DemoNavigation (recommended default)
**Notes:** [auto] Caption remains `Static preview`. Do not imply live simulation.

---

## Catalog layout inside the Kobalte shell

| Option | Description | Selected |
|--------|-------------|----------|
| Keep PlaygroundShell list; add compact preview per link | Preserves owner-approved sidebar + Kobalte drawer; `shell.spec.ts` keeps `.catalog-card` count 0 | ✓ |
| Restore Phase 18 three-column `.catalog-card` grid | Success criterion 2 forbids this; cards broke narrow widths | |
| Separate catalog page or extra design system | New capability / out of phase | |

**User's choice:** Keep PlaygroundShell list; add compact preview per link (recommended default)
**Notes:** [auto] Shared DemoNavigation for sidebar and mobile drawer. Keep 44px hit targets and dark tokens. Kobalte 0.13.12 drawer-only.

---

## Reset live-control honesty

| Option | Description | Selected |
|--------|-------------|----------|
| After Reset, every preset select shows documented initial value | Covers live runtime presets and construction pending values | ✓ |
| Resync live presets only | Construction Apply-setting selects could still lie after Reset | |
| Leave pendingValue; treat native reset as enough | Audit gap; chrome would still disagree with the rebuilt world | |

**User's choice:** After Reset, every preset select shows documented initial value (recommended default)
**Notes:** [auto] Representative live labels: Fountain emission-rate, Float or Sink body, Color Mixer stir-speed, Water Wheel jet-strength. Implementation (remount vs resync) is Claude's discretion.

---

## Focused Chromium smoke

| Option | Description | Selected |
|--------|-------------|----------|
| Extend `just web-player-smoke` for six previews + representative Reset labels | Matches ROADMAP success criterion 4 and Phase 19 focused-smoke pattern | ✓ |
| Combinatorial matrix of every live preset on every scene | Overkill versus representative coverage | |
| Screenshot-hash or multi-browser matrix | Phase 19 D-13 kept hashes optional and Chromium-only | |

**User's choice:** Extend `just web-player-smoke` for six previews + representative Reset labels (recommended default)
**Notes:** [auto] Desktop sidebar all six; one mobile-drawer preview check; no Firefox/Safari/Linux qualification.

---

## Claude's Discretion

- Exact SVG/CSS artwork and list-item packing
- Remount vs resync for `pendingValue`
- Playwright file split within the existing Chromium suite

## Deferred Ideas

- Phase 21 leftover cleanup (`loadProofSession`, FallbackPanel, scene file-lengths)
- Dam Break headless speed versus C++
- Design-system migration
