# Phase 34: Scene Timing Survey - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-07
**Phase:** 34-Scene Timing Survey
**Mode:** Yolo (recommended answers auto-selected)
**Areas discussed:** Coverage check, Timing method, Idle scenes, Committed table

## Coverage check

| Option | Description | Selected |
|--------|-------------|----------|
| Test parses `SCENE_IDS` from scenes.ts | Fails when the catalog gains a scene | ✓ |
| Exhaustive Rust match only | Catches `SceneId` additions, not catalog-only drift | |

## Timing method

| Option | Description | Selected |
|--------|-------------|----------|
| `--runs 3`, fresh session each run, median/min/max | Matches the roadmap rule of a median of at least three | ✓ |
| Single long run | No noise signal | |

## Idle scenes

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse README preview cues before warmup | Already reviewed cue positions | ✓ |
| Synthetic multi-stroke drawing script | More code, more arbitrary | |

## Committed table

| Option | Description | Selected |
|--------|-------------|----------|
| xtask prints the ranked Markdown table; it is committed to `docs/benchmarks/scene-survey.md` | Simple and reproducible | ✓ |
| Generated doc writer in xtask | More tooling than needed | |

## Claude's Discretion

- Column names, the even-run median, and helper layout.

## Deferred Ideas

- Browser frame-drop measurement (Phase 35), before/after (Phase 36).
