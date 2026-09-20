# Phase 22: Observability shell - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-20
**Phase:** 22-observability-shell
**Mode:** Yolo
**Areas discussed:** Evidence stamp layout, Recipe surface, Profile capture, Timer diagnostic path, Isolation and proof

---

## Evidence stamp layout

| Option | Description | Selected |
|--------|-------------|----------|
| Dated `target/dam-break-perf/<utc-stamp>/` with `pair.json` + `pair.md` | Persist unprofiled pair; never overwrite an existing stamp | ✓ |
| Stdout-only Markdown like today | Leaves PERF-PAIR unsatisfied | |
| Reuse research name `target/v12-native-perf/` | Older research path; roadmap locked `dam-break-perf` | |

**User's choice:** Yolo recommended default — persist under `target/dam-break-perf/<utc-stamp>/` with filename-safe UTC `YYYY-MM-DDTHH-MM-SSZ`.
**Notes:** Keep current Markdown stdout. Profile artifacts are `rust.json.gz` + `profile-identity.json`. Timers get `timers.json` in a new stamp.

---

## Recipe surface

| Option | Description | Selected |
|--------|-------------|----------|
| Keep pair recipe; add thin profile and timer aliases | `just` prints; xtask owns flags and orchestration | ✓ |
| One just recipe that pairs, profiles, and times | Mixes profiled timings into the gate command | |
| Hide CMake/samply flags in just | Violates repo just/xtask split | |

**User's choice:** Yolo recommended default — extend `playground-dam-break-bench`; add `playground-dam-break-profile` and `playground-dam-break-timers`.
**Notes:** Optional C++ `-g` is a profile-command flag only, never a pair flag or new CMake preset.

---

## Profile capture

| Option | Description | Selected |
|--------|-------------|----------|
| `[profile.profiling]` + samply 0.13.1 of timed Rust loop; fail closed if missing | Matches PERF-PROFILE; profiled times are not the 3× number | ✓ |
| Profile the existing `--release` binary without debug info | Weaker symbolication | |
| Silently skip when samply is missing | Host-specific Mac signing would hide setup failure | |
| Call `step_profiled` inside `dam-break-bench` | Contaminates the gate binary | |

**User's choice:** Yolo recommended default — profiling profile + samply 0.13.1; missing tool fails with install text.
**Notes:** Construction, insertion, warm-up, capture, and rendering stay outside the sampled timed loop.

---

## Timer diagnostic path

| Option | Description | Selected |
|--------|-------------|----------|
| Separate `dam-break-timers` path using existing `step_profiled` parents | Optional cheap iteration; absent from the gate process | ✓ |
| Enable parent Instant spans in the unprofiled pair | Mixes diagnostic overhead into the 3× authority | |
| Invent a new parent vocabulary | Phase 12 parents already exist | |

**User's choice:** Yolo recommended default — separate diagnostic path; reuse `particle_prepare` / `particle_solve` / `rigid_solve` (and other existing parents).
**Notes:** Write `timers.json` into a new stamp.

---

## Isolation and proof

| Option | Description | Selected |
|--------|-------------|----------|
| Fake cmake/samply xtask tests + `package verify`; no physics edits | Roadmap: tooling can be proven without a live pair | ✓ |
| Require live Dam Break pair and real samply as phase DoD | Host-specific, slow, and not needed to prove the shell | |
| Add samply/serde/dhat to `liquidfun` | Breaks package isolation | |

**User's choice:** Yolo recommended default — fake tools for tests; live host run optional; `liquidfun` stays clean.
**Notes:** Audit notes, dhat, 3× gate, and WASM sanity are later phases.

## Claude's Discretion

- Exact JSON field names beyond required identity and timing fields
- Exact samply argv and timed-loop targeting
- Fake-tool fixture shape
- Whether profile/timer commands reuse `--warmup` / `--steps` defaults
- Exact fail-closed stderr wording

## Deferred Ideas

- Named-function audit and dhat — Phase 23
- Shared hot-path physics and ≤ 3× gate — Phase 24
- WASM/playground sanity — Phase 25
- SIMD / `unsafe` / in-process C ABI
