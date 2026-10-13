---
phase: 40-timed-valve-and-teaching-levels
fixed_at: 2026-10-13T09:47:30Z
review_path: .planning/phases/40-timed-valve-and-teaching-levels/40-REVIEW.md
iteration: 1
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 40: Code Review Fix Report

**Fixed at:** 2026-10-13T09:47:30Z
**Source review:** .planning/phases/40-timed-valve-and-teaching-levels/40-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 1
- Fixed: 1
- Skipped: 0

Scope was `critical_warning`, so the six Info findings (IN-01 to IN-06) were not attempted.

## Fixed Issues

### WR-01: `[` / `]` valve shortcut does not work on layouts that type brackets with a modifier

**Files modified:** `web/src/puzzle/keyboard.ts`, `web/src/components/PuzzlePanel.tsx`, `web/tests/puzzle-keyboard.test.ts`
**Commit:** 686dbd808
**Applied fix:** Added a required `altGraph` field to `PuzzleKeyInput`. `PuzzlePanel` fills it from `event.getModifierState("AltGraph")`. The new pure helper `isModifiedShortcut` lets `[` and `]` through with Alt/Option or AltGr (Ctrl+Alt plus AltGraph). It still blocks them under Meta, or under Ctrl when AltGraph is not active. Every other key keeps the old rule: any Ctrl, Meta or Alt blocks it. In the unit tests, the old case that expected `{ altKey: true }` to block `]` now uses `{ altKey: true, metaKey: true }`. New Arrange/Act/Assert tests cover macOS Option+`[`, Windows AltGr+`]`, and Ctrl+Alt without AltGraph (still ignored).
**Verification:** `tsc --noEmit` passed. `vitest run tests/puzzle-keyboard.test.ts tests/puzzle-shell-wiring.test.ts` passed 60 of 60. `vite build` then `playwright test e2e/puzzle-valve.spec.ts` passed 1 of 1 (it presses `]` and `[` on the stepper).

---

_Fixed: 2026-10-13T09:47:30Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
