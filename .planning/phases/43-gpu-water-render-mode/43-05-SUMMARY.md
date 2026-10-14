---
phase: 43-gpu-water-render-mode
plan: "05"
subsystem: web-physics
tags: [typescript, worker, water, speed-lane, frame-contract, render-mode]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 43-2026-10-13T22-55-49
generated_at: "2026-10-14T00:19:36.489Z"
requires:
  - "43-01 ProofSession.captureFrame(particleSpeeds?) and RenderFrame.maybeParticleSpeeds"
  - "43-04 mode.ts wantsParticleSpeeds(mode)"
provides:
  - "GeneratedProofSession.captureFrame(maybeParticleSpeeds?: boolean)"
  - "createSceneSession(generated, maybeWantsParticleSpeeds?) and createPuzzleSession(generated, id, maybeWantsParticleSpeeds?), accessor read at every capture"
  - "PuzzleHost picks appearance; adoptPuzzleSession and scene-lifecycle pass wantsParticleSpeeds(renderMode())"
  - "Worker advance/capture requests carry a validated boolean speeds flag; transferred frames keep and transfer the lane"
  - "createWorkerSession(generation, maybeFactory?, maybeAbortSignal?, maybeWantsParticleSpeeds?)"
affects:
  - "43-06 browser proof: data-speed-lane present in Water on direct scenes, puzzles and the Tesla Valve worker"
tech-stack:
  added: []
  patterns:
    - "Stateless per-capture flag: owners hold a () => boolean accessor and never store a mode-derived flag"
key-files:
  created: []
  modified:
    - web/src/physics/session.ts
    - web/src/physics/puzzle-session.ts
    - web/src/player/puzzle-host.ts
    - web/src/player/scene-lifecycle.ts
    - web/src/physics/worker-messages.ts
    - web/src/physics/simulation-worker.ts
    - web/src/physics/worker-session.ts
    - web/tests/session.test.ts
    - web/tests/puzzle-session.test.ts
    - web/tests/puzzle-host-fixture.ts
    - web/tests/worker-messages.test.ts
    - web/tests/worker-session.test.ts
decisions:
  - "D-11 discretion (how the capture option is sent): a stateless per-capture flag. All three owners (direct scene, puzzle, Tesla Valve worker) read wantsParticleSpeeds(appearance.renderMode()) at capture time; no owner stores the flag, so Play/Edit/Retry, level, scene and recreate changes cannot leave it stale"
  - "Worker frame requests read the accessor when the request is built (call time), not when the queued request is posted"
  - "The lane follows the requested mode, not the drawn surface; export and replay callers keep zero-argument captureFrame()"
metrics:
  duration: 5min
  completed: 2026-10-14
  tasks: 2
  files: 12
---

# Phase 43 Plan 05: Speed lane wiring for scene, puzzle and worker owners Summary

Captures now request the per-particle speed lane only while Water is the requested mode. This applies to all three frame owners: direct scene sessions, puzzle sessions and the Tesla Valve live worker. Each owner holds a `() => boolean` accessor and reads it on every capture, so turning Water off drops the lane on the next frame. The worker protocol carries a validated `speeds` flag and transfers the lane buffer.

## What Was Built

- **Task 1 (948a5bcc0):** `GeneratedProofSession.captureFrame(maybeParticleSpeeds?: boolean)`. `createSceneSession` and `createPuzzleSession` take an optional `maybeWantsParticleSpeeds` accessor and call `captureFrame(maybeWantsParticleSpeeds?.() ?? false)`; the puzzle path covers both `captureFrame()` and `nextFrame()` through `captureParsed`. `PuzzleHost` now picks `"appearance"`. `adoptPuzzleSession` passes `() => wantsParticleSpeeds(host.appearance.renderMode())`. `scene-lifecycle.ts` defines `wantsSpeeds` once and passes it to `createSceneSession`. The puzzle host fixture gained a minimal `appearance` stub (`"shaded-blob"`). There are 8 new tests: scene `false` without an accessor, the flag following the accessor frame by frame, the lane parsed when requested, and no `maybeParticleSpeeds` key when declined. The puzzle tests cover `captureFrame` and `nextFrame`, with and without an accessor.
- **Task 2 (dc81856c6):** `advance` and `capture` operations carry `readonly speeds: boolean`. `parseWorkerRequest` moves `capture` out of the no-payload group. Both ops validate the flag through `speedFlag`, which throws `Invalid speed lane flag`. `validateTransferredFrame` forwards `particleSpeeds: () => frame.maybeParticleSpeeds`, so the shared parser checks the lane fail closed. `frameTransferables` appends the lane before the tight-buffer check. `simulation-worker.ts` `sendFrame` calls `session.captureFrame(speeds)` with `request.speeds` for advance and capture. `createWorkerSession` takes a fourth `maybeWantsParticleSpeeds` argument and sets `speeds` on `nextFrame`, `advanceBudgeted` and `captureFrame`. `scene-lifecycle.ts` passes `wantsSpeeds` to it. The tests cover acceptance, rejection of missing, string, number or null flags on both ops, lane values kept, the key omitted when absent, a wrong-length lane rejected, 6 buffers with the lane and 5 without, offset and oversized lane views rejected, and worker requests that follow the accessor at call time for all three frame ops.

## Verification

- `cd web && bun run typecheck`: clean (tsconfig includes `tests`).
- `bun run test:unit`: 119 files, 1836 tests passed (1815 after Task 1, which added 8; Task 2 added 21).
- `bun run build:app`: ok. `bun scripts/bright-builds-check.ts all`: findings=0.
- `CI=true bunx playwright test e2e/live-worker.spec.ts`: 8 passed (worker vs direct state, gravity recovery, warmup checkpoints).
- `git diff --quiet a490fc708 -- web/src/export/svg-export-worker.ts web/scripts/readme-svg.ts web/scripts/puzzle-replay.ts web/src/export/record.ts`: unchanged (D-17).
- Acceptance greps: `readonly speeds: boolean` ×2, `Invalid speed lane flag`, `particleSpeeds: () =>`, `captureFrame(speeds)`, `maybeWantsParticleSpeeds?.() ?? false` in both owner files, `wantsSpeeds` ×3 in `scene-lifecycle.ts`, `wantsParticleSpeeds(` in `puzzle-host.ts` and `scene-lifecycle.ts`.
- Only `worker-session.ts` builds `advance`/`capture` requests (grep across `src`, `e2e`, `benchmarks`, `scripts`).

## Deviations from Plan

None. The plan was executed as written. The worker owner reads the accessor through a small local `speeds()` helper that returns `maybeWantsParticleSpeeds?.() ?? false`. The flag validation is the shared `speedFlag` helper, which keeps the planned error text.

## Known Stubs

None.

## Next Phase Readiness

Plan 06 can assert in the browser that `data-speed-lane="present"` appears in Water on direct scenes, on puzzles and on `#/scene/tesla-valve` (worker backend), and that it is absent in other modes. It can also run the puzzle on/off RunFingerprint check and the 25/25 gate (T-43-14).

## Self-Check: PASSED

- FOUND: .planning/phases/43-gpu-water-render-mode/43-05-SUMMARY.md
- FOUND: 948a5bcc0
- FOUND: dc81856c6
