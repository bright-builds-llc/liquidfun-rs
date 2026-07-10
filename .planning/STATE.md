---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
stopped_at: Completed 02-09-PLAN.md
last_updated: "2026-07-10T09:28:25.975Z"
last_activity: 2026-07-10
progress:
  total_phases: 12
  completed_phases: 1
  total_plans: 19
  completed_plans: 14
  percent: 74
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-07-09)

**Core value:** Deliver an independent, maintainable Rust physics engine whose LiquidFun behavior is demonstrated against a pinned upstream oracle through explicit inventory, tests, differential evidence, and documented tolerances.
**Current focus:** Phase 2 — Semantic Protocol and Oracle Round Trip

## Current Position

Phase: 2 (Semantic Protocol and Oracle Round Trip) — EXECUTING
Plan: 10 of 14
Status: Ready to execute
Last activity: 2026-07-10

Progress: [███████░░░] 74%

## Performance Metrics

**Velocity:**

- Total plans completed: 5
- Average duration: Not available
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
| --- | --- | --- | --- |
| 1 | 5 | - | - |

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

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 1 must resolve the final oracle revision, license/alteration obligations, legacy CMake build behavior, and canonical compiler before implementation assumptions harden.
- Phase 3 must prove identity, invalidation, callback, and particle-remapping semantics before those choices spread through public APIs.

## Session Continuity

Last session: 2026-07-10T09:28:25.973Z
Stopped at: Completed 02-09-PLAN.md
Resume file: None
