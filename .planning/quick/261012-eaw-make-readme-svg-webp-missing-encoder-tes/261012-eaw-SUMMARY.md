---
phase: quick-261012-eaw
plan: "01"
subsystem: web-scripts
tags: [vitest, flaky-test, child-process, readme-webp]
generated_by: gsd-quick-orchestrator
lifecycle_mode: quick
phase_lifecycle_id: quick-261012-eaw
generated_at: "2026-10-12T15:25:00Z"
requires: []
provides:
  - startAnimatedWebpEncoder resolves only after the encoder process has spawned
affects:
  - web/scripts/readme-svg/encode.ts
  - web/tests/readme-svg-webp.test.ts
key-files:
  created: []
  modified:
    - web/scripts/readme-svg/encode.ts
    - web/tests/readme-svg-webp.test.ts
decisions:
  - "Fixed the production race instead of retiming the test: start awaits the child's spawn or error event"
  - "A missing encoder now rejects the start call with the same 'ffmpeg is required' message, so no frame write is ever attempted"
  - "Kept the existing spawnFailure race in writeFrame and finish for process errors after a successful spawn"
metrics:
  completed: 2026-10-12
  tasks: 2
  files: 2
---

# Quick 261012-eaw: Make the missing-encoder WebP test deterministic

`startAnimatedWebpEncoder` returned as soon as `spawn()` returned. Node reports
a missing executable later, through the child's `error` event. The test's first
frame write could therefore settle before that error arrived: it could resolve,
or reject with a stream error instead of the mapped message. That made the test
flaky under load. The Phase 38 `just web-build` run failed once, then passed
8 of 8 times on rerun.

The encoder now waits for the child's `spawn` or `error` event before it
returns. A missing command rejects `startAnimatedWebpEncoder` itself with
"ffmpeg is required to record README WebP previews." and removes the temporary
directory. The test asserts that contract, so it no longer races a write
against ENOENT. The caller in `capture.ts` already awaits start before its
`try` block, so it surfaces the error unchanged.

## Tasks

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1 | Wait for the encoder process before accepting frames | 7702cc6b3 | web/scripts/readme-svg/encode.ts |
| 2 | Assert that a missing encoder rejects start | 7702cc6b3 | web/tests/readme-svg-webp.test.ts |

## Verification

| Check | Result |
| --- | --- |
| Baseline, old code: `bunx vitest run tests/readme-svg-webp.test.ts` x20 | 20/20 pass; the race does not reproduce in isolation |
| Ordering probe (Node 24.13.0, scratchpad): spawn a missing command, write 512 bytes | the write returns before `error:ENOENT` fires, confirming the asynchronous error |
| New code: `bunx vitest run tests/readme-svg-webp.test.ts` x20, sequential | 20/20 pass, 8 tests each, including the real-ffmpeg recording case |
| New code: 4 rounds of 10 parallel runs of the same file | 40/40 pass |
| `cd web && bun run typecheck` | pass |
| `cd web && bun run test:unit` | 789/789 pass |
| `bun scripts/bright-builds-check.ts all` | findings=0 |

## Deviations from Plan

The orchestrator ran the plan inline instead of spawning a planner and an
executor. The change touches two files and was fully diagnosed before planning.

## Residual Risk

The old code did not fail locally, so there is no local before-and-after
failure. The fix removes the race by construction: start cannot resolve
without the `spawn` event, and Node never emits `spawn` for ENOENT.

## Self-Check: PASSED
