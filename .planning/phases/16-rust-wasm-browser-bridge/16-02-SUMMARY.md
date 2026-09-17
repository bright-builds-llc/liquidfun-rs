---
phase: 16-rust-wasm-browser-bridge
plan: "02"
subsystem: web
tags: [bun, solidjs, vite, vitest, wasm-pack, typescript]

requires:
  - phase: 16-rust-wasm-browser-bridge
    provides: Private WASM wrapper and generated declaration contract from Plan 16-01
provides:
  - Exact-pinned Bun frontend dependency graph and committed lockfile
  - Safe rerunnable current-checkout WASM generation command
  - Bounded copied typed-array parser and exactly-once session owner
affects: [16-03, 16-04, web-playground]

tech-stack:
  added: [Bun 1.4.2, SolidJS 1.9.15, Vite 8.3.0, TypeScript 7.0.2, Vitest 5.0.1, Playwright 1.63.0]
  patterns: [generated-package build output, parse-at-boundary frames, idempotent explicit disposal]

key-files:
  created:
    - scripts/web-build.ts
    - web/package.json
    - web/bun.lock
    - web/tsconfig.json
    - web/vite.config.ts
    - web/vitest.config.ts
    - web/index.html
    - web/src/physics/frame.ts
    - web/src/physics/loader.ts
    - web/src/physics/session.ts
    - web/tests/frame.test.ts
    - web/tests/session.test.ts
  modified:
    - .gitignore
    - justfile

key-decisions:
  - "Regenerate ignored wasm-pack output from the current checkout before any frontend verification or build."
  - "Defer the complete Vite build until Plan 16-03 supplies the renderer and application entrypoints."
  - "Poison the TypeScript session owner after any advance, capture, parse, or frame-cleanup failure."

patterns-established:
  - "Build output boundary: generated WASM glue is disposable, ignored, and recreated through one exact-tool script."
  - "Frame boundary: validate counts, constructors, strides, finite values, and radii once before rendering."
  - "Ownership boundary: every temporary frame is freed in finally and each session is freed at most once."

requirements-completed: [WASM-02, WASM-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 16-2026-09-17T01-48-41
generated_at: 2026-09-17T02:51:30Z

duration: 7min
completed: 2026-09-17
---

# Phase 16 Plan 02: Reproducible Frontend Boundary Summary

**An exact-pinned Bun/SolidJS toolchain now regenerates ignored current-checkout WASM output, while a bounded TypeScript parser and explicit owner expose only copied renderer-ready arrays with exactly-once cleanup.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-09-17T02:44:49Z
- **Completed:** 2026-09-17T02:51:30Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments

- Locked all seven reviewed frontend packages under Bun 1.4.2 and proved frozen installation without lockfile drift.
- Added safe rerunnable `wasm` and deferred `build` orchestration with exact Bun, Rust, and wasm-pack checks plus ignored summaries.
- Validated all five copied frame lanes against hard bounds, constructors, strides, finite values, positive radii, and rigid-count consistency.
- Covered one-step capture, temporary-frame cleanup, failure poisoning, idempotent disposal, and use-after-dispose with 29 focused Vitest cases.

## Task Commits

Each task was committed atomically:

1. **Task 1: Pin the Bun frontend and make generation reproducible**
   - `5777496` (`chore`) — exact frontend pins, lockfile, configuration, build script, and Just recipes
   - `2da56a6` (`fix`) — ignored Bun-installed dependency output
1. **Task 2: Parse copied frames and enforce exactly-once ownership**
   - `1b33b47` (`test`) — failing frame validation and ownership contract
   - `0670265` (`feat`) — bounded parser, generated loader, and idempotent owner

## Files Created/Modified

- `.gitignore` — Ignores dependencies, generated bindings, bundles, browser reports, and web-build summaries.
- `justfile` — Adds transparent `web-wasm` and deferred `web-build` recipes.
- `scripts/web-build.ts` — Checks exact tools, safely regenerates bindings, and defines ordered frontend verification.
- `web/package.json` and `web/bun.lock` — Pin and lock the reviewed Bun frontend graph.
- `web/tsconfig.json`, `web/vite.config.ts`, and `web/vitest.config.ts` — Configure strict SolidJS typing, local-root Vite output, and Node Vitest tests.
- `web/index.html` — Provides only the semantic root required by the later app entrypoint.
- `web/src/physics/frame.ts` — Parses generated frame methods into bounded renderer-ready typed arrays.
- `web/src/physics/loader.ts` — Initializes generated web output through Vite's WASM asset URL.
- `web/src/physics/session.ts` — Owns one generated session and frees frames/sessions exactly once.
- `web/tests/frame.test.ts` and `web/tests/session.test.ts` — Verify frame invariants and lifecycle behavior.

## Decisions Made

- Kept generated JavaScript, declarations, package metadata, and WASM output untracked and reproducible.
- Returned wasm-bindgen's JavaScript-owned typed arrays directly after validation instead of cloning or exposing raw memory.
- Converted generated operation failures into one bounded fatal session error and prohibited catch-and-continue reuse.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Ignored Bun-installed dependencies**

- **Found during:** Task 1 commit status review
- **Issue:** The first frozen install left `web/node_modules/` as an untracked generated dependency tree.
- **Fix:** Added the exact dependency directory to `.gitignore`.
- **Files modified:** `.gitignore`
- **Verification:** `git status --short -- web/node_modules` returned no output and managed checks passed.
- **Committed in:** `2da56a6`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The fix preserves clean, rerunnable dependency installation without changing frontend or runtime scope.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 16-03 can add pure Canvas projection, rendering, and the approved SolidJS entrypoints against the validated `RenderFrame` boundary.
- The first complete `just web-build` remains intentionally deferred until those entrypoints and renderer tests exist.

## Self-Check: PASSED

All listed source files, the summary, and all four task commits were verified.

---

*Phase: 16-rust-wasm-browser-bridge*
*Completed: 2026-09-17*
