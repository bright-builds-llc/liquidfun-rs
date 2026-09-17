---
phase: 16-rust-wasm-browser-bridge
plan: "01"
subsystem: wasm
tags: [rust, wasm-bindgen, wasm-pack, particles, copied-frames]

requires:
  - phase: v1.0-experimental-foundation
    provides: Native Rust world, rigid-body, particle, and fixed-step APIs
provides:
  - Unpublished non-default liquidfun-wasm workspace crate
  - Opaque persistent proof session with bounded fixed stepping
  - Validated copied typed-array frame ABI
  - Exact Rust 1.97.0 and wasm-pack 0.15.0 generation proof
affects: [16-02, 16-03, 16-04, web-playground]

tech-stack:
  added: [wasm-bindgen 0.2.128, wasm-pack 0.15.0, wasm32-unknown-unknown]
  patterns: [native-testable session core, thin wasm shell, owned bulk frame copies]

key-files:
  created:
    - crates/liquidfun-wasm/Cargo.toml
    - crates/liquidfun-wasm/src/frame.rs
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/liquidfun-wasm/src/lib.rs

key-decisions:
  - "Keep liquidfun as the sole default workspace member and depend outward from the private wrapper."
  - "Copy five validated bounded numeric lanes into JavaScript-owned typed arrays instead of exposing WASM memory."
  - "Reject invalid advance counts and step-index overflow before invoking the engine."
  - "Record compilation and ES-module import only; real Chromium execution remains Plan 16-04."

patterns-established:
  - "Native core, thin shell: SessionCore owns simulation state while ProofSession only maps bounded errors."
  - "Coherent frame copy: borrow particle lanes once, validate alignment, then export immutable owned arrays."

requirements-completed: [WASM-01, WASM-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 16-2026-09-17T01-48-41
generated_at: 2026-09-17T02:41:30Z

duration: 13min
completed: 2026-09-17
---

# Phase 16 Plan 01: Native-Testable WASM Session and Frame Summary

**A private Rust/WASM owner now advances 192 water particles and rigid basin geometry through bounded native engine steps, returning five validated copied frame lanes and generating browser-targeted bindings with the exact pinned tools.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-09-17T02:28:39Z
- **Completed:** 2026-09-17T02:41:30Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Added an unpublished `cdylib`/`rlib` wrapper without changing ordinary default Cargo builds.
- Constructed a persistent checked basin with three static fixtures, one dynamic circle, and exactly 192 explicitly colored water particles.
- Added bounded fixed stepping, coherent frame capture, copied typed-array exports, and 14 focused native tests.
- Generated the exact `--target web` package with Rust 1.97.0 and wasm-pack 0.15.0, confirming typed arrays and generated `free()` declarations.

## Task Commits

Each task was committed atomically:

1. **Task 1: Establish the private wrapper and validated copied-frame ABI**
   - `e28cf10` (`test`) — failing copied-frame contract tests
   - `66292cf` (`feat`) — validated copied-frame implementation
1. **Task 2: Build the persistent proof scene and opaque session**
   - `e48b64e` (`test`) — failing persistent-session tests
   - `dc50752` (`feat`) — bounded scene, session, stepping, and capture
1. **Task 3: Prove exact-tool real WASM generation**
   - `00f70bd` (`chore`) — exact target/tool generation gate

## Files Created/Modified

- `Cargo.toml` — Adds the wrapper as a non-default workspace member.
- `Cargo.lock` — Pins wasm-bindgen 0.2.128 and its matching generated-glue dependencies.
- `crates/liquidfun-wasm/Cargo.toml` — Defines the private WASM wrapper package.
- `crates/liquidfun-wasm/src/lib.rs` — Exposes the opaque wasm-bindgen session and frame shell.
- `crates/liquidfun-wasm/src/frame.rs` — Validates bounded semantic lanes and returns fresh boxed numeric slices.
- `crates/liquidfun-wasm/src/scene.rs` — Builds the fixed checked basin, rigid circle, and particle grid.
- `crates/liquidfun-wasm/src/session.rs` — Owns persistent world state, fixed stepping, and coherent frame capture.

## Decisions Made

- Kept all browser binding types outside the publishable engine and preserved `default-members = ["crates/liquidfun"]`.
- Used fixed bounded error strings at the wasm shell while retaining typed private construction, stepping, and capture errors.
- Treated Node ES-module import as a static generated-package probe only; no browser-runtime success is claimed by this plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Restored the GSD metrics table expected by state tooling**

- **Found during:** Plan completion metadata update
- **Issue:** The new-milestone `STATE.md` used prose bullets under Performance Metrics, so `state record-metric` could not record Plan 16-01.
- **Fix:** Replaced the placeholder bullets with the expected table schema and reran the required gsd-tools command successfully.
- **Files modified:** `.planning/STATE.md`
- **Verification:** `state record-metric` returned `"recorded": true`.
- **Committed in:** Plan metadata commit

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Metadata-only compatibility repair; implementation scope and runtime behavior are unchanged.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The generated declarations now provide the exact `ProofSession` and `ProofFrame` types needed by Plan 16-02's typed frontend boundary.
- Browser initialization, visible Canvas execution, and runtime disposal evidence remain explicitly unclaimed until later Phase 16 plans.

## Self-Check: PASSED

All listed source files, the summary, and all five task commits were verified.

---

*Phase: 16-rust-wasm-browser-bridge*
*Completed: 2026-09-17*
