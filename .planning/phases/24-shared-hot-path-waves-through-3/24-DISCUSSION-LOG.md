# Phase 24: Shared hot-path waves through 3× - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-21T15:03:30.003Z
**Phase:** 24-shared-hot-path-waves-through-3
**Mode:** Yolo
**Areas discussed:** First-wave target, Public API vs internal indices, Wave admission and cadence, Later ranked frames, Release invariant policy, Stop-at-3× vs continue, Shared-path cheat ban, Spot-check recording, Second canary vs same-cluster, Correctness gates, Evidence honesty and isolation

---

[auto-select] Selected all gray areas: First-wave target, Public API vs internal indices, Wave admission and cadence, Later ranked frames, Release invariant policy, Stop-at-3× vs continue, Shared-path cheat ban, Spot-check recording, Second canary vs same-cluster, Correctness gates, Evidence honesty and isolation.

No pending todos matched Phase 24 (`todo_count: 0`). Advisor mode off (no USER-PROFILE.md).

---

## First-wave target

| Option | Description | Selected |
|--------|-------------|----------|
| `particle_rows` index-preserving shape | Carry dense rows through neighborhood/contact generation to match C++ `FindContacts_Reference` `Proxy.index` | ✓ |
| SIMD / NEON contact solver first | Treat ~328× as a vectorization problem | |
| HashMap `ParticleId` → row cache | Keep linear identities, add a map in front of `position` scans | |
| Unsafe indexing / lift `forbid` | Drop bounds checks as the first optimization | |

**User's choice:** [auto] `particle_rows` index-preserving shape (recommended default)
**Notes:** Phase 23 audit names ~93.6% self; SIMD/PGO/`unsafe` are in **Not found**. D-01/D-02.

---

## Public API vs internal indices

| Option | Description | Selected |
|--------|-------------|----------|
| Keep public `ParticleId`; dense rows internal | `ParticleContact` / `ParticleNeighborPair` stay handle-typed | ✓ |
| Change public contacts to integer indices | Match C++ layout in the public API | |
| You decide | Planner picks representation | |

**User's choice:** [auto] Keep public `ParticleId`; dense rows internal (recommended default)
**Notes:** Phase 9 locked typed handles. Experimental APIs may evolve, but this phase does not need a public break. D-03.

---

## Wave admission and cadence

| Option | Description | Selected |
|--------|-------------|----------|
| PERF-ADMIT as written, one concern per wave, new exclusive pair stamp | Named share + unprofiled ratio improves + tests pass; physics mismatch fails | ✓ |
| Land any faster sample even if physics drifts | Speed first | |
| Batch every ranked frame in one change | One giant kernel rewrite | |

**User's choice:** [auto] PERF-ADMIT as written, one concern per wave (recommended default)
**Notes:** Roadmap: one concern per wave; re-pair into a new dated directory; preserve failed records. D-04/D-05.

---

## Later ranked frames

| Option | Description | Selected |
|--------|-------------|----------|
| Retarget samply; admit leftover only with remaining share; stop at ≤3× | Order from Phase 23 table, not hunt-list paste | ✓ |
| Implement the whole hunt list regardless of share | Full-world clone, `to_vec`, `from_view` as required waves | |
| Gold-plate every ~0.2% frame after the gate | Keep optimizing past 3× in this phase | |

**User's choice:** [auto] Retarget samply; stop at ≤3× (recommended default)
**Notes:** D-06/D-07.

---

## Release invariant policy

| Option | Description | Selected |
|--------|-------------|----------|
| Later wave only: `debug_assertions` analog of C++ `NDEBUG` if still named | Do not do this first; keep typed API errors fail-closed | ✓ |
| Gate `check_invariants` in wave 1 | 2.2% as the Dam Break closer | |
| Keep full invariant scans in `--release` forever | Safety over C++ shape | |

**User's choice:** [auto] Later wave only (recommended default)
**Notes:** Pitfall 1 box2d-rust analog is real but cannot close ~328×. D-08.

---

## Stop-at-3× vs continue

| Option | Description | Selected |
|--------|-------------|----------|
| Continue additional scalar samply-retargeted waves until PERF-GATE; no SIMD/`unsafe` | Phase does not complete without ≤3× | ✓ |
| Stop after ranked table even if still >3× | Document leftover and finish | |
| Allow SIMD/`unsafe` if ranked waves fail | Lift baseline to chase the canary | |

**User's choice:** [auto] Continue scalar waves until PERF-GATE (recommended default)
**Notes:** Standing authorization continues diagnosis; PERF-BASELINE still holds. Criterion only near 3×. D-09.

---

## Shared-path cheat ban

| Option | Description | Selected |
|--------|-------------|----------|
| Shared `liquidfun` particle/rigid kernels only | No Dam Break-only, WASM-copy, skipped-solver, or smaller particle count | ✓ |
| Dam Break scene special-case is enough | Fastest path to a number | |
| Shrink Medium to 192 particles | Fake the gate | |

**User's choice:** [auto] Shared kernels only (recommended default)
**Notes:** PERF-SHARED / Pitfall 3. D-10.

---

## Spot-check recording

| Option | Description | Selected |
|--------|-------------|----------|
| Native `--release` headless via existing scene builders; stamp + unreviewed notes table | Five scenes; no C++ pair; hangs fail | ✓ |
| Per-scene C++ pairs | Mini Phase 12 matrix | |
| Skip spot-checks if Dam Break hits 3× | Gate-only | |

**User's choice:** [auto] Headless scene builders + notes (recommended default)
**Notes:** PERF-SPOT. D-11.

---

## Second canary vs same-cluster

| Option | Description | Selected |
|--------|-------------|----------|
| Default same-cluster note; second canary only if a spot-check looks different | Not a 32-case matrix | ✓ |
| Always samply a second playground scene | Extra profile regardless | |
| Skip PERF-CANARY2 | Gate-only | |

**User's choice:** [auto] Default same-cluster note (recommended default)
**Notes:** After `particle_rows`, other particle scenes should share the cluster. D-12.

---

## Correctness gates

| Option | Description | Selected |
|--------|-------------|----------|
| `cargo test -p liquidfun` plus existing particle/determinism/differential checks | No Linux oracle or Phase 12 matrix per wave | ✓ |
| Full C++ sanitizer + Phase 12 sealed matrix every wave | Strict qualification | |
| Speed samples without tests | Land on ratio alone | |

**User's choice:** [auto] Native particle/determinism gates (recommended default)
**Notes:** Independent AI review after phase verification; no self-approval. D-13.

---

## Evidence honesty and isolation

| Option | Description | Selected |
|--------|-------------|----------|
| Unprofiled pair stamp is 3×; empty manifest; bitflags-only `liquidfun`; `unsafe_code = "forbid"` | Refresh unreviewed docs; gitignore blobs | ✓ |
| Copy ratio into `manifest.toml` / README | Public claim | |
| Add profiler deps to `liquidfun` | Convenient instrumentation | |

**User's choice:** [auto] Unprofiled pair + isolation (recommended default)
**Notes:** D-14/D-15. WASM sanity is Phase 25.

---

## Claude's Discretion

- Exact internal dense-index types and whether neighborhood pairs store `(row, row)` privately while still exposing `ParticleId`.
- Exact wave split if `particle_rows` itself needs more than one commit.
- Exact headless spot-check binary/recipe name and notes table layout.
- How many leftover frames to list after the gate.
- Whether to keep a one-line pointer to the Phase 23 MEASURED_HEAD sample after later pair refreshes.

## Deferred Ideas

- WASM/playground sanity — Phase 25 (`PERF-WASM`)
- Remaining Dam Break delta close notes — Phase 25 (`PERF-NOTES`)
- SIMD / Rayon / relaxing `unsafe_code = "forbid"` — later opt-in
- Filling `reference/performance/manifest.toml` — out of this milestone
- Required C++ samply wrap — not needed for first waves
- Phase 12 sealed 32-case matrix — conflicts with this milestone
