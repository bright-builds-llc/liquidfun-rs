---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
stopped_at: Completed 03-02-PLAN.md
last_updated: "2026-07-11T02:44:20.738Z"
last_activity: 2026-07-11
progress:
  total_phases: 12
  completed_phases: 2
  total_plans: 24
  completed_plans: 21
  percent: 88
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-07-10)

**Core value:** Deliver an independent, maintainable Rust physics engine whose LiquidFun behavior is demonstrated against a pinned upstream oracle through explicit inventory, tests, differential evidence, and documented tolerances.
**Current focus:** Phase 3 — Rust Object Model and Storage Architecture

## Current Position

Phase: 3 (Rust Object Model and Storage Architecture) — EXECUTING
Plan: 3 of 5
Status: Ready to execute
Last activity: 2026-07-11

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**

- Total plans completed: 19
- Average duration: Not available
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
| --- | --- | --- | --- |
| 1 | 5 | - | - |
| 2 | 14 | - | - |

**Recent Trend:**

- Last 5 plans: None
- Trend: Not available

*Updated after each plan completion*
| Phase 02 P01 | 5 min | 1 tasks | 6 files |
| Phase 02 P02 | 10 min | 1 tasks | 6 files |
| Phase 02 P03 | 32 min | 2 tasks | 7 files |
| Phase 02 P04 | 18 min | 2 tasks | 7 files |
| Phase 02 P05 | 9 min | 1 tasks | 9 files |
| Phase 02 P06 | 27 min | 3 tasks | 11 files |
| Phase 02 P07 | 6 min | 1 tasks | 4 files |
| Phase 02 P08 | 26 min | 1 tasks | 9 files |
| Phase 02 P09 | 12 min | 1 tasks | 7 files |
| Phase 02 P10 | 33 min | 3 tasks | 13 files |
| Phase 02 P11 | 29 min | 1 tasks | 9 files |
| Phase 02 P12 | 28 min | 1 tasks | 16 files |
| Phase 02 P13 | 10 min | 1 tasks | 6 files |
| Phase 02 P14 | 22 min | 2 tasks | 11 files |
| Phase 03 P01 | 8 min | 2 tasks | 4 files |
| Phase 03 P02 | 14 min | 2 tasks | 5 files |

## Accumulated Context

### Decisions

Decisions are logged in the `PROJECT.md` Key Decisions table. Current roadmap constraints:

- Phase 1 freezes oracle, ancestry, licensing/provenance, build/toolchain, architecture/risk evidence, and repository foundations before broad physics work.
- Production remains a cohesive Cargo-first native Rust engine; C++ stays isolated to development-time oracle workflows.
- Compatibility evidence is added per subsystem, and performance optimization waits for the complete scalar baseline.
- [Phase 02]: Separate engine-neutral protocol contracts from the effectful differential runner so parsing and comparison do not depend on orchestration.
- [Phase 02]: Keep both harness crates unpublished and outside default-members while preserving liquidfun as the unchanged sole default consumer package.
- [Phase 02]: Validate protocol versions, semantic IDs, and SHA-256 identities at construction or deserialization so downstream code cannot receive unchecked primitives.
- [Phase 02]: Expose only named immutable phase-2 limit profiles for one-shot, reusable-corpus, and sanitizer execution.
- [Phase 02]: Keep physics mismatch outside HarnessFailureKind while preserving bounded request, provenance, process, stderr, and limit evidence.
- [Phase 02]: Decode newline-complete JSONL directly into bounded strict raw structs before constructing validated scenario and trace domain values.
- [Phase 02]: Accept trace comparison input only after request/provenance identity, ordered checkpoints, payload hash, and adapter reset proof all validate.
- [Phase 02]: Keep phase2-v1 simulation time exact-bit and checkpoint order explicit while exposing typed synthetic numeric policies for later comparator tests.
- [Phase 02]: Keep schema and tolerance renderers test-only so ordinary protocol builds expose no regeneration or filesystem-write path. — Presentation artifacts are review surfaces; typed protocol code remains the runtime authority.
- [Phase 02]: Limit Phase-2 numeric presentation to exact simulation-time bits plus synthetic comparator-coverage policies. — Broad rigid-body, joint, and particle tolerances remain deferred until subsystem evidence exists.
- [Phase 02]: Use exact 0.5-second timestep bit patterns so two ordered empty-world checkpoints have distinguishable, exactly representable simulation times.
- [Phase 02]: Canonicalize checked-in request, handshake, and trace records only in memory through validated public protocol values; verification never rewrites the corpus.
- [Phase 02]: Keep malformed corpus cases minimal so each rejected file reaches one intended stable codec category.
- [Phase 02]: Reject incompatible request, scenario, tolerance, schema, and engine-role identities before semantic comparison.
- [Phase 02]: Identify failures by checkpoint, phase, typed semantic path, and mismatch kind so values may shrink without changing failure identity.
- [Phase 02]: Keep reduction pure and deterministic through typed injected signatures, logical elapsed time, and protocol-owned candidate revalidation.
- [Phase 02]: Use the official nlohmann/json v3.12.0 single-header release asset and immutable tag license. — Exact upstream bytes, source URLs, and local SHA-256 verification make the private C++ parser dependency reproducible and reviewable.
- [Phase 02]: Keep nlohmann/json entirely under private tools/reference vendoring. — Published Rust crates and ordinary Cargo paths remain free of C++ parser dependencies and build-time downloads.
- [Phase 02]: Parse C++ oracle requests with a bounded duplicate-aware SAX event sink rather than a mutable JSON DOM.
- [Phase 02]: Scope every C++ oracle request to a fresh b2World and emit trace_end only after destruction, mapping cleanup, reset proof, and epoch increment.
- [Phase 02]: Keep exact IEEE-754 and length-prefixed SHA-256 compatibility in a cohesive protocol_bits module while protocol.cpp owns typed parsing and deterministic encoding.
- [Phase 02]: Bind every C++ oracle handshake to independently checked lock and adapter identities through a configured out-of-tree header. — xtask and CMake derive the same fixed-source digest before the child can report successful provenance.
- [Phase 02]: Allow only reviewed oracle presets and the liquidfun-reference build target in xtask. — Structured fixed arguments prevent contributor-provided paths or unrelated native targets from entering oracle orchestration.
- [Phase 02]: Keep ASan and UBSan fail-fast while demoting only two legacy upstream warnings under Clang sanitizer builds. — Sanitizer evidence must terminate unsuccessfully without modifying the pinned upstream or weakening warning denial for repository-authored code.
- [Phase 02]: Keep Phase-2 native execution private and limited to exact-bit empty-world traces with reset epochs.
- [Phase 02]: Use one synchronous enum state machine for one-shot, finite reuse, and sanitizer child supervision.
- [Phase 02]: Drain child stdout and stderr concurrently, retain bounded first/last diagnostics, and reap every poisoned child before returning.
- [Phase 02]: Fixture promotion derives accepted paths from typed artifact kind and scenario ID; explicit candidate-bound review and no-clobber atomic publication are mandatory.
- [Phase 02]: Require manifest-v2 records to resolve to one strict trace or regression variant before provenance validation.
- [Phase 02]: Stage reviewed traces from exact supervised oracle JSONL and bind accepted evidence to source, identity, policy, notice, and explicit review metadata.
- [Phase 02]: Parse every differential command into a closed canonical invocation before any upstream verification or child execution. — This prevents invalid contributor input from causing effects and makes every runner argument auditable.
- [Phase 02]: Keep Cargo-only aggregate checks useful by validating protocol presentations, fixtures, package isolation, and artifact provenance without an initialized C++ submodule. — Artifact evidence remains fail-closed while checkout identity stays reserved for the full initialized mode.
- [Phase 02]: Make the exact twelve-row TESTING.md layer table executable policy through a strict read-only xtask checker. — Required commands, prerequisites, artifacts, retry policy, placement, and semantic interpretation now fail closed instead of depending on prose review.
- [Phase 02]: Keep Cargo CI submodule-free while canonical oracle CI alone owns real C++ round trips, sanitizer execution, and read-only evidence assertions. — The trust split preserves ordinary Cargo isolation and keeps exact tool and upstream prerequisites confined to the evidence lane.
- [Phase 02]: Run the sanitizer profile as a bounded two-request reused session. — The scheduled command must prove both fail-fast sanitizer handling and adapter reset epochs 1 then 2 rather than duplicate one-shot coverage.
- [Phase 03]: Use complete private world-key, slot, and u64 generation identity for every typed handle. — Complete identity prevents stale-slot and cross-world aliasing without exposing layout.
- [Phase 03]: Use deterministic LIFO vacant-slot reuse with explicit ascending-slot iteration. — Allocation and traversal remain reproducible without hash iteration.
- [Phase 03]: Permanently retire a slot when its generation cannot advance. — Generation wrap can never resurrect an ancient handle.
- [Phase 03]: Centralize typed world destruction cascades in documented occurrence order. — Validate the root before mutation, keep adjacency consistent, and retain owned post-invalidation evidence.
- [Phase 03]: Keep user associations in sealed application-owned typed side tables. — Avoid raw pointers, type erasure, and lifetime coupling while making cascade cleanup explicit.

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 1 must resolve the final oracle revision, license/alteration obligations, legacy CMake build behavior, and canonical compiler before implementation assumptions harden.
- Phase 3 must prove identity, invalidation, callback, and particle-remapping semantics before those choices spread through public APIs.

## Session Continuity

Last session: 2026-07-11T02:44:16.618Z
Stopped at: Completed 03-02-PLAN.md
Resume file: None
