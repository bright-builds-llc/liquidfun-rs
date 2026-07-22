---
phase: 11-examples-headless-tooling-and-testbed
fixed_at: 2026-07-22T16:44:48Z
review_path: /Users/peterryszkiewicz/Repos/liquidfun-rs/.planning/phases/11-examples-headless-tooling-and-testbed/11-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 11: Code Review Fix Report

**Fixed at:** 2026-07-22T16:44:48Z
**Source review:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/.planning/phases/11-examples-headless-tooling-and-testbed/11-REVIEW.md`
**Iteration:** 1

**Summary:**

- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: Renderer capability claims are disconnected from rendered output

**Files modified:** `crates/liquidfun-testbed/src/capability/render.rs`, `crates/liquidfun-testbed/src/capability/report.rs`
**Commit:** c5fcd25
**Applied fix:** Replaced fixed post-render capability values with per-frame semantic emission evidence collected beside the typed draw operations, conservatively aggregated that evidence across every required frame, removed the two unconditional passing dispositions, and added a suppression regression proving that omitted contact normals fail the real capability matrix.

### WR-02: Package extraction does not consume the archive instance that was validated

**Files modified:** `tools/xtask/src/package.rs`, `tools/xtask/src/package/tests.rs`
**Commit:** d979c14
**Applied fix:** Read the compressed package archive once into a bounded immutable byte buffer and made both inspection and extraction consume those exact bytes. Added a regression that overwrites the source archive after inspection and proves extraction still uses the validated original manifest.

***

_Fixed: 2026-07-22T16:44:48Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
