---
phase: 17-shared-player-and-early-pages-delivery
plan: "07"
subsystem: infra
tags: [github-pages, oidc, wasm, host-01, host-02]

requires:
  - phase: 17-shared-player-and-early-pages-delivery
    provides: Production Vite base /liquidfun-rs/ and same-checkout just web-build dist gate
  - phase: 17-shared-player-and-early-pages-delivery
    provides: One-session Dam Break playground shell with Play/Pause/Reset/Retry
provides:
  - SHA-pinned Pages build-site job that runs bun scripts/web-build.ts build
  - Main-only OIDC deploy-pages job using github-pages and no PAT
  - Main-safe concurrency that does not cancel an in-flight production deploy
affects: [17-08, pages-deploy, host-03]

tech-stack:
  added: []
  patterns: [same-checkout-pages-workflow, oidc-pages-deploy, main-safe-concurrency]

key-files:
  created:
    - .github/workflows/pages.yml
  modified: []

key-decisions:
  - "Every main and pull_request run builds WASM plus web/dist from the same checkout with no path filters."
  - "Deploy uses job-scoped pages: write plus id-token: write and the github-pages environment; no PAT."
  - "cancel-in-progress is false on main so an in-flight deploy finishes while queued revisions may coalesce."

patterns-established:
  - "Pages workflow top-level permissions stay contents: read; write tokens live only on deploy-pages."
  - "build-site runs bun scripts/web-build.ts build and uploads web/dist; deploy-pages needs that artifact and does not rebuild WASM."
  - "Concurrency group is workflow+ref; cancel-in-progress is github.ref != refs/heads/main."

requirements-completed: [HOST-01, HOST-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 17-2026-09-17T11-24-10
generated_at: 2026-09-17T12:19:10Z

duration: 2min
completed: 2026-09-17
---

# Phase 17 Plan 07: Pages Workflow And OIDC Deploy Summary

**SHA-pinned two-job Pages workflow that builds WASM and `web/dist` from one checkout, then deploys that artifact to GitHub Pages with OIDC and main-safe concurrency.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-17T12:17:34Z
- **Completed:** 2026-09-17T12:19:10Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added `.github/workflows/pages.yml` named `Pages` for `main` pushes, pull requests, and `workflow_dispatch`, with no path filters.
- `build-site` installs Rust 1.97.0, `wasm32-unknown-unknown`, wasm-pack 0.15.0, and Bun 1.4.2, then runs `bun scripts/web-build.ts build` and uploads `web/dist`.
- `deploy-pages` runs only after `build-site` succeeds on a `main` push, using job-scoped `pages: write` plus `id-token: write`, the `github-pages` environment, and official SHA-pinned Pages actions.

## Task Commits

Each task was committed atomically:

1. **Task 1: Write the two-job Pages workflow** - `e744f45` (feat)
1. **Task 2: Statically prove HOST-01/02 shape** - `7dfafdf` (docs)

**Plan metadata:** recorded in the plan-completion docs commit

## Files Created/Modified

- `.github/workflows/pages.yml` - Same-checkout WASM+site build and OIDC Pages deploy

## Decisions Made

- Every `main` and pull-request run builds WASM plus `web/dist` from the same checkout with no path filters; deploy does not rebuild WASM.
- Deploy uses job-scoped `pages: write` plus `id-token: write` and the `github-pages` environment. No PAT, `peaceiris/actions-gh-pages`, or workflow-level write tokens.
- `cancel-in-progress` is `${{ github.ref != 'refs/heads/main' }}` so production deploys finish; queued main revisions may coalesce.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Avoided a literal `paths:` token in the D-14 comment**
- **Found during:** Task 2 (Statically prove HOST-01/02 shape)
- **Issue:** The plan asked comments to mention D-14 (no path filters) while the Task 2 python assert requires `'paths:' not in text`.
- **Fix:** Worded the comment as "no path filters" so D-14 is cited and the assert still passes.
- **Files modified:** `.github/workflows/pages.yml`
- **Verification:** python assert plus `rg -n "D-14|D-15|D-16"`
- **Committed in:** `7dfafdf` (Task 2 commit)

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Comment wording only. HOST-01/02 shape is unchanged.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 17-08. The workflow is committed and statically matches HOST-01/02. Plan 17-08 records the live Pages URL and source revision after an ordinary `main` push. This plan did not dispatch a deploy.

---
*Phase: 17-shared-player-and-early-pages-delivery*
*Completed: 2026-09-17*

## Self-Check: PASSED
