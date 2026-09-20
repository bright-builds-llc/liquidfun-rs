# Phase 20 deferred items

Discovered during 20-02 and left out of scope (pre-existing, not caused by
this plan's files).

- Bright Builds `file-lengths` still fails for
  `tools/xtask/tests/upstream_cli.rs` (642 physical lines). Plan 20-02
  required the checker to exit 0 as a cap on `web/src/app.css` and new
  catalog files; those files are 628, 120, and 40 lines. Do not split the
  xtask test in this catalog-preview plan. Scene-module file-length debt
  remains Phase 21.

- Plan 20-03 re-ran `bun scripts/bright-builds-check.ts file-lengths` as a
  cap on `web/src/App.tsx` (617 lines). The same pre-existing
  `tools/xtask/tests/upstream_cli.rs` finding remains; do not split that
  test in this Reset-honesty plan.

- Plan 20-04 `just web-player-smoke` exited 1 with 27 passed / 3 failed.
  Both new preview tests passed. Out of scope for 20-04 (`shell.spec.ts`
  preview locators only):
  - `player.spec.ts` Reset `RESET_STEP_CEILING` 8 received 29 after 20-03
    keyed SceneControls remount. Plan 20-05 diagnosed an actually-reset
    world: `resetStep < seriesStep` passed while a later single-sample
    read saw 4-step catch-up frames. `resetNearZero` now observes the
    first restarted `data-step-index`.
  - `demo-media-clock.spec.ts` 240-frame capture exceeded the 30s
    Playwright timeout. Plan 20-05 raised that test to 120s so
    `just web-player-smoke` can finish the numbered-frame capture.
