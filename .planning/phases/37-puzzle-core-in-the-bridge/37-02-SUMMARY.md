---
phase: 37-puzzle-core-in-the-bridge
plan: "02"
subsystem: web-frame-contract
tags: [typescript, frame, puzzle, D-17, GOAL-02, vitest]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 37-2026-10-11T21-49-11
generated_at: "2026-10-11T23:30:08.960Z"
requires:
  - parseRenderFrame / RawProofFrame / RenderFrame (web/src/physics/frame.ts, unchanged scene behavior)
provides:
  - PuzzleOutcome and PuzzleFrameStatus exports (web/src/physics/frame.ts)
  - six optional RawProofFrame puzzle getters (goalCount, goalTarget, goalFull, outcome, outcomeStep, stepLimit)
  - RenderFrame.maybePuzzle, parsed fail-closed by parsePuzzleStatus
affects:
  - 37-07 (Rust ProofFrame Option getters must match these names and the 0/1/2 outcome codes)
  - 37-08 and Phase 38 HUD (consume RenderFrame.maybePuzzle)
tech-stack:
  added: []
  patterns:
    - optional generated getters called with `?.()` so existing fakes keep compiling
    - conditional spread for optional fields under exactOptionalPropertyTypes
key-files:
  created:
    - web/tests/frame-puzzle.test.ts
  modified:
    - web/src/physics/frame.ts
key-decisions:
  - "TS only validates and carries the Rust-decided puzzle outcome: all-or-nothing scalars, outcome codes 0 running / 1 won / 2 lost, outcomeStep present iff latched, and a loss must latch exactly at stepLimit"
  - "The worker path (validateTransferredFrame) still returns scene-shaped frames; puzzles use the direct backend until Phase 38"
requirements: [GOAL-02]
duration: 2min
completed: 2026-10-11
---

# Phase 37 Plan 02: Optional puzzle frame scalars in the TS parser Summary

**`parseRenderFrame` now reads the six optional D-17 puzzle getters into a fail-closed `maybePuzzle: PuzzleFrameStatus`. Scene frames, the two `FakeRawProofFrame` fakes and the worker literal are unchanged.**

## Performance

- **Duration:** about 2 min (23:28:52Z to 23:30:08Z)
- **Tasks:** 2 of 2
- **Files:** 2 (1 created, 1 modified)

## Exported TS Contract

```typescript
export type PuzzleOutcome = "running" | "won" | "lost";

export interface PuzzleFrameStatus {
  readonly goalCount: number;
  readonly goalTarget: number;
  readonly goalFull: number;
  readonly outcome: PuzzleOutcome;
  readonly maybeOutcomeStep?: number;
  readonly stepLimit: number;
}

// RawProofFrame (all optional): goalCount?(), goalTarget?(), goalFull?(),
// outcome?(), outcomeStep?(), stepLimit?(): number | undefined
// RenderFrame: readonly maybePuzzle?: PuzzleFrameStatus
```

Parse rules in `parsePuzzleStatus`:

- If all six getters are absent or return `undefined`, the frame is a scene frame and has no `maybePuzzle` key.
- If any of goalCount, goalTarget, goalFull, outcome or stepLimit is missing, the parse throws.
- The values must be safe integers, with `goalTarget >= 1`, `goalFull >= goalTarget`, `stepLimit >= 1` and `goalCount <= particleCount`.
- The outcome code must be 0, 1 or 2.
- While running, outcomeStep must be absent. After a win or loss it must be present, with `outcomeStep <= stepIndex` and `outcomeStep <= stepLimit`. A loss also needs `outcomeStep === stepLimit`.
- Every failure throws the existing `Invalid Rust/WASM frame` message, so no frame values appear in the error.

`frame.ts` went from 221 to 346 lines, under the 628-line limit.

## Task Commits

1. **Task 1: Add the optional puzzle scalars to the frame parser.** TDD: RED gave 14 failures and 2 passes (the two scene cases), then GREEN passed 16 of 16.
1. **Task 2: Run the web checks and commit.** `2fedfc54b` (`feat(37-02): parse optional puzzle frame scalars into maybePuzzle`). It contains only `web/src/physics/frame.ts` and `web/tests/frame-puzzle.test.ts`.

The plan asks for one commit, so both tasks share `2fedfc54b`.

## Check Results (`target/phase37/checks-02.results`)

| Check | Result |
| --- | --- |
| `cd web && bun run typecheck` | exit=0 |
| `cd web && bun run test:unit` | exit=0 (62 files, 548 tests) |
| `bun scripts/bright-builds-check.ts all` | exit=0 (file-lengths findings=0) |

Task 1 verification: `bunx vitest run tests/frame-puzzle.test.ts tests/frame.test.ts tests/session.test.ts tests/worker-messages.test.ts` passed 64 tests. `git diff --quiet -- web/tests/frame.test.ts web/tests/session.test.ts web/src/physics/worker-messages.ts` succeeded.

## Deviations from Plan

- The TDD RED step was not committed separately, because the plan asks for one commit that holds only the two owned files.
- I added a small `parsePositiveInteger` helper and a `parseOutcomeStep` helper to keep `parsePuzzleStatus` short. The rules are the same as the plan's.
- The orchestrator asked for `generated_by: gsd-execute-phase`, and this file uses it. The plan's output section says `gsd-executor`.
- GOAL-02 was not marked complete in REQUIREMENTS.md. Phase verification owns that, and the Rust settle-window latch is still to come.

Otherwise the plan was executed as written.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: web/src/physics/frame.ts, web/tests/frame-puzzle.test.ts
- FOUND: commit 2fedfc54b
- FOUND: target/phase37/checks-02.results (3 lines, all exit=0)
