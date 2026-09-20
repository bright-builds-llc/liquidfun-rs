---
phase: 21-playground-leftover-cleanup
plan: "01"
subsystem: web-loader
tags: [loadSceneSession, ProofSession, just-web-smoke, phase16-forensics, solidjs]

requires:
  - phase: 18-six-native-physics-demos
    provides: loadSceneSession(sceneId) constructor over generated ProofSession
  - phase: 17-shared-player-and-early-pages-delivery
    provides: just web-player-smoke product gate vs opt-in just web-smoke
provides:
  - loadSceneSession-only loader export
  - historical just web-smoke documentation
affects: [21-02 FallbackPanel leftover, 21-04 player-smoke and independent review]

tech-stack:
  added: []
  patterns:
    - App constructs WASM worlds only through loadSceneSession(sceneId)
    - just web-smoke stays historical Phase 16 forensic chrome behind PHASE16_CLOSURE_ATTEMPT_DIR

key-files:
  created: []
  modified:
    - web/src/physics/loader.ts
    - README.md
    - TESTING.md

key-decisions:
  - "Delete unused loadProofSession with no compatibility alias; keep new ProofSession(sceneId)."
  - "Leave rust-wasm-proof.spec.ts out of test:player and do not fold Dispose-session or PNG-hash into just web-player-smoke."
  - "Document just web-smoke as historical Phase 16 forensic chrome, not the v1.1 product gate."

patterns-established:
  - "Product constructor is loadSceneSession(sceneId) returning new ProofSession(sceneId)."
  - "Ordinary playground proof is just web-player-smoke; forensic smoke remains opt-in and is not expected to pass against current Play/Pause/Reset chrome."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-09-20T19-45-21
generated_at: 2026-09-20T20:14:48Z

duration: 3min
completed: 2026-09-20
---

# Phase 21 Plan 01: Unused Proof Helper Cleanup Summary

**Deleted unused `loadProofSession`; `loadSceneSession(sceneId)` remains the only constructor, and `just web-smoke` is documented as historical Phase 16 forensic chrome, not the v1.1 product gate.**

## Performance

- **Duration:** 3 min
- **Started:** 2026-09-20T20:12:07Z
- **Completed:** 2026-09-20T20:14:48Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Removed the unused Dam Break `loadProofSession` wrapper from `web/src/physics/loader.ts`.
- Kept `loadSceneSession(sceneId)` returning `new ProofSession(sceneId)` after WASM `init`; `App.tsx` still awaits `loadSceneSession(id)`.
- Documented `just web-smoke` in README and TESTING as historical Phase 16 forensic chrome that allocates `target/phase16/closure-attempt-N`, sets `PHASE16_CLOSURE_ATTEMPT_DIR`, and still looks for Dispose-session and PNG-hash selectors.
- Left `web/package.json` `test:player` on the four-file Chromium allowlist and left the forensic spec skipped when `PHASE16_CLOSURE_ATTEMPT_DIR` is undefined.

## Task Commits

Each task was committed atomically:

1. **Task 1: Delete loadProofSession and keep loadSceneSession** - `71d2ef5` (refactor)
2. **Task 2: Document just web-smoke as historical Phase 16 chrome** - `be0dd6c` (docs)

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Files Created/Modified

- `web/src/physics/loader.ts` — exports only `loadSceneSession`
- `README.md` — names `just web-smoke` historical Phase 16 forensic chrome
- `TESTING.md` — same wording plus existing Phase 16 forensic section and closure-attempt rules

## Decisions Made

- Deleted `loadProofSession` instead of re-wiring it into App, tests, or the forensic spec (D-01).
- Kept `rust-wasm-proof.spec.ts` opt-in and out of `test:player` (D-02).
- Did not rewrite forensic selectors onto Play/Pause/Reset; documented expected failure against current chrome (D-03).
- Did not rename generated `ProofSession`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 21-02 FallbackPanel unknown-only chrome. Forensic smoke stays historical; product proof remains `just web-player-smoke` in a later plan. Do not run C++ timing.

## Self-Check: PASSED

---
*Phase: 21-playground-leftover-cleanup*
*Completed: 2026-09-20*
