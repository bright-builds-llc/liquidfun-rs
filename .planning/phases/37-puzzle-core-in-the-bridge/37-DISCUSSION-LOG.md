# Phase 37: Puzzle Core in the Bridge - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-11
**Phase:** 37-puzzle-core-in-the-bridge
**Mode:** Yolo (recommended answers auto-selected; advisor agents skipped because the v1.5 research synthesis already compares the options)
**Areas discussed:** Determinism spike, Level registry and spec, Placements, Goal counting and latch, Session and frame contract, Replay harness

---

## Determinism spike

| Option | Description | Selected |
| --- | --- | --- |
| Bun script on the generated wasm plus a native hash test over existing scenes | Reuses the readme-svg loading path; no new tools | ✓ |
| wasmtime or a wasm32 test runner | A new tool dependency | |
| Playwright in a real browser | Heavier; Phase 40 owns browser replay | |

**Choice:** Bun plus native FNV-1a frame hash, run before any puzzle code, with the policy branching on the result.

## Level registry and spec

| Option | Description | Selected |
| --- | --- | --- |
| Separate `LevelId` allowlist plus TS `PUZZLE_IDS`, with an include_str sync test | Matches the shared acceptance rule | ✓ |
| Extend `SceneId` | Breaks README, survey and portrait coverage | |

**Choice:** a separate registry, with `first-pour` as the first level. Zero tools loses and one plank wins.

## Placements

| Option | Description | Selected |
| --- | --- | --- |
| `kinds: u8[]` + integer `poses: i32[]` stride 3 (0.05 m grid, τ/24 index) | Exact snapping, serializable | ✓ |
| `poses: f32[]` in meters and radians | Needs snapping on both sides | |

**Choice:** integer poses. There is one minimal static plank kind, and its rotation is baked into the vertices from a literal table.

## Goal counting and latch

| Option | Description | Selected |
| --- | --- | --- |
| Strict-interior rectangle, counted groups, 30-step settle, latched outcome | Research synthesis | ✓ |
| Win on first reaching target | A splash could win | |

## Session and frame contract

| Option | Description | Selected |
| --- | --- | --- |
| `SessionSource` enum, `newPuzzle`, `levelInfo()`, optional frame scalars | Scenes unchanged | ✓ |
| A separate PuzzleSession type | Duplicates stepping and frame capture | |

## Replay harness

| Option | Description | Selected |
| --- | --- | --- |
| Line-based text fixtures shared by native unit tests and a Bun wasm replay | No serde; one format | ✓ |
| JSON fixtures | Needs serde or a hand parser in the bridge | |

## Claude's Discretion

- The `first-pour` geometry and tuning, the module split, the counted-mask mapping, the hash helper, and the shape of the `levelInfo` return value.

## Deferred Ideas

- Full validator (Phase 39), timed valve (Phase 40), Playwright replay (Phase 40), route and HUD (Phase 38), moving cup regions (Phase 40).
