---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
stopped_at: Completed 07-02-PLAN.md
last_updated: "2026-07-13T02:04:03.612Z"
last_activity: 2026-07-13
progress:
  total_phases: 12
  completed_phases: 6
  total_plans: 74
  completed_plans: 64
  percent: 86
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-07-12)

**Core value:** Deliver an independent, maintainable Rust physics engine whose LiquidFun behavior is demonstrated against a pinned upstream oracle through explicit inventory, tests, differential evidence, and documented tolerances.
**Current focus:** Phase 7 — Rigid Solver, World Operations, and CCD

## Current Position

Phase: 7 (Rigid Solver, World Operations, and CCD) — EXECUTING
Plan: 3 of 13
Status: Ready to execute
Last activity: 2026-07-13

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**

- Total plans completed: 61
- Average duration: Not available
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
| --- | --- | --- | --- |
| 1 | 5 | - | - |
| 2 | 14 | - | - |
| 3 | 5 | - | - |
| 4 | 7 | - | - |
| 5 | 8 | - | - |
| 6 | 22 | - | - |

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
| Phase 03 P03 | 10 min | 3 tasks | 5 files |
| Phase 03 P04 | 7 min | 3 tasks | 9 files |
| Phase 03 P05 | 10 min | 3 tasks | 5 files |
| Phase 04 P01 | 8 min | 2 tasks | 5 files |
| Phase 04 P03 | 15 min | 2 tasks | 6 files |
| Phase 04 P02 | 14 min | 2 tasks | 5 files |
| Phase 04 P04 | 28 min | 1 tasks | 8 files |
| Phase 04 P05 | 29 min | 2 tasks | 17 files |
| Phase 04 P06 | 31 min | 1 tasks | 12 files |
| Phase 04 P07 | 17 min | 1 tasks | 7 files |
| Phase 06 P14 | 9 min | 1 tasks | 5 files |
| Phase 06 P15 | 16 min | 2 tasks | 12 files |
| Phase 06 P16 | 15 min | 2 tasks | 13 files |
| Phase 06 P17 | 14 min | 2 tasks | 10 files |
| Phase 06 P18 | 13 min | 2 tasks | 9 files |
| Phase 06 P19 | 8 min | 1 tasks | 9 files |
| Phase 06 P21 | 16 min | 2 tasks | 9 files |
| Phase 06 P20 | 24 min | 2 tasks | 12 files |
| Phase 06 P22 | 15 min | 2 tasks | 7 files |
| Phase 07 P01 | 21 min | 2 tasks | 6 files |
| Phase 07 P10 | 23min | 2 tasks | 13 files |
| Phase 07 P02 | 22 min | 2 tasks | 14 files |

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
- [Phase 03]: Keep contacts transient and expose only borrow-scoped read-only views or owned fixture snapshots. — No durable contact identity or internal reference can escape a hook.
- [Phase 03]: Apply bounded typed commands sequentially after unlock and continue after recoverable invalid-handle failures. — Request order and every application result remain deterministic evidence.
- [Phase 03]: Resume hook panics after restoring the lock, discarding pending commands, and poisoning coherent-state operations. — Partial step progress can never masquerade as healthy state.
- [Phase 03]: Keep stable ParticleId values separate from private ephemeral dense particle indices. — World and particle-system scope are validated before dense lookup, so reorder and compaction never leak storage position.
- [Phase 03]: Use one validate-then-commit particle permutation transaction for all representative lanes and derived indices. — Lane alignment, identity maps, proxies, contacts, pairs, triads, lifetime order, and group ranges cannot partially diverge.
- [Phase 03]: Preserve pending-delete snapshots until compaction advances or retires the particle identity generation. — Pending and stale states remain distinct while destruction evidence stays owned.
- [Phase 03]: Keep owned particle lane bundles and declared fixed capacity private until Phase 9. — Phase 3 proves ownership and teardown without publishing raw buffers, bulk mutation, or solver API.
- [Phase 03]: Keep the Phase-3 consumer surface limited to opaque identities, owned records, restricted hooks, typed commands, and typed side tables. — Dense particle storage and future external buffers remain private until their Phase-9 contract is implemented and verified.
- [Phase 03]: Use ARCHITECTURE.md as the evidence-linked sign-off for every locked Phase-3 decision. — Each disposition cites executable code or tests and avoids claiming broad solver parity.
- [Phase 04]: Keep scalar and vector implementation modules private behind the curated liquidfun::math surface while exposing math::settings as a documented namespace. — This preserves a cohesive deep module and future representation freedom without hiding consumer-required math behavior.
- [Phase 04]: Preserve the selected b2_pi decimal token and every derived settings expression grouping exactly. — Exact f32 encodings and source-order compatibility take precedence over substituting superficially equivalent standard-library constants.
- [Phase 04]: Keep matrix, rotation, transform, and sweep storage private behind initialized APIs, and validate exact sweep advance candidates before mutation. — Preserves representation freedom and finite checked state without changing valid kernel grouping.
- [Phase 04]: Mirror the closed Rust math-probe contract in external C++ with memcpy bit transport, and gate canonical D1 evidence on complete compiler/runtime identity while noncanonical D2/D3 results cannot promote. — One exact cross-language contract preserves IEEE payloads and keeps unsupported local floating capabilities explicit rather than silently weakening canonical evidence.
- [Phase 04]: Route Phase 4 verification through closed named xtask commands with typed native comparison and fixed two-run D0 replay. — Contributor and CI entrypoints cannot substitute paths, executables, compiler flags, profiles, policies, or arbitrary run counts; canonical evidence remains read-only.
- [Phase 04]: Treat local Phase 4 probe passes as scoped D2 differential evidence and leave D1/platform validation unclaimed. — Evidence dimensions remain independent and cannot be promoted from configured CI or one local noncanonical toolchain.
- [Phase 04]: Keep b2Settings differential validation absent until the complete settings surface is directly probed. — Implementation and unit tests do not substitute for cross-language semantic evidence.
- [Phase 06]: Build a complete candidate BodyState before aggregate mass mutation. — Candidate-first source ordering and one final replacement make create/reset failures effect-free.
- [Phase 06]: Require at least one dynamic body before contact admission. — This mirrors the pinned ShouldCollide predicate and rejects every non-dynamic pair before filtering or allocation.
- [Phase 06]: Use separate exact post-overlap witnesses for static/kinematic and kinematic/kinematic admission. — Declaration-first zero-contact checkpoints prevent a shared omission from passing cross-engine equality.
- [Phase 06]: Admit only the exact fixed Phase 6 step tuple at every protocol boundary. — Keeps configurable solver semantics deferred to Phase 7 while preventing cross-engine input divergence.
- [Phase 06]: Use one 128-action maximum across Rust, schema, and C++. — A single reviewed bound prevents native acceptance from becoming an oracle harness error.
- [Phase 06]: Reject custom mass data through source-ordered centered-inertia intermediates before effects. — Malformed or overflowing mass data remains an input validation failure instead of reaching either engine.
- [Phase 06]: Execute C++ protocol tests and rigid-world comparison under the sanitizer preset before read-only assertion. — Compile-only coverage cannot prove the rigid decode, contact, teardown, and trace paths are sanitizer-clean.
- [Phase 06]: Admit only one-shot rigid-world compare for the oracle-asan-ubsan Phase 6 command shape. — Keep sanitizer execution closed without widening replay, minimization, reuse, or public step configuration.
- [Phase 06]: Demote overriding-option only for the read-only upstream Box2D target when sanitizer flags are present. — Apple Clang can build the reviewed sanitizer preset while repository-authored warning denial and canonical D1 constraints remain intact.
- [Phase 06]: Build the complete target BodyState before body-type contact destruction or mutation, then install it once. — Implicit aggregate failure must be typed and effect-free.
- [Phase 06]: Pass explicit fixture removal a prevalidated remaining-fixture BodyState while body cascades skip mass recomputation. — The explicit path needs atomic reset semantics, while a destroyed parent has no mass state to preserve.
- [Phase 06]: Use one private current-checkout identity core for ordinary rigid execution and fixture lifecycle paths. — A single adapter and compile digest implementation prevents provenance drift between compare, stage, review, and promotion.
- [Phase 06]: Recompute rigid checkout identity during every candidate replay before review or promotion effects. — Fresh validation prevents a stage-time result from authorizing evidence after adapter or effective compile database drift.
- [Phase 06]: Treat exact origin inertia zero as the pinned no-inertia branch without evaluating the parallel-axis subtraction. — Matches the pinned SetMassData branch while positive origin inertia remains strictly validated before effects.
- [Phase 06]: Require finite source-ordered intermediates and strictly positive centered inertia for positive origin inertia. — Prevents the equality boundary from reaching an assertion or inverse-inertia divide by zero.
- [Phase 07]: Represent body booleans in one compact private flag set while exposing only named semantic accessors and builders. — Keeps snapshots copyable, avoids a new dependency, and satisfies warning-denied code shape.
- [Phase 07]: Preserve upstream ignored branches before value validation for non-dynamic and asleep PreserveSleep force/impulse calls. — Successful no-effects remain source-recognizable without a public outcome taxonomy.
- [Phase 07]: Build every fallible body-control result on a copied BodyState and replace world state exactly once after complete validation. — Invalid handles, inputs, and derived overflow cannot partially mutate body state.
- [Phase 07]: Retain the two-family Phase 6 REQUIRED corpus while exposing PHASE7_REQUIRED and ALL registries. — Keeps accepted Phase 6 fixtures valid while closing schema and adapter inputs over all nine bounded families.
- [Phase 07]: Expose only semantic CCD completion and bounded partial-progress classification. — Candidate indices, caches, counters, queues, and sweep bookkeeping remain private implementation state.
- [Phase 07]: Canonicalize query multiplicity and equal-fraction ray ties only in evidence policy. — Production callback traversal remains source-faithful and does not acquire a new ordering contract.
- [Phase 07]: Use the already-reviewed Phase 7 protocol maximum of 1024 for both public solver-iteration bounds. — Production and closed evidence boundaries now share one resource ceiling.
- [Phase 07]: Compute the warm-start ratio as previous_inverse_time_step * current_time_step and retain the previous inverse across zero-duration calls. — This preserves the pinned source expression and variable-step history.
- [Phase 07]: Expose only Complete and ContinuousPending while applying automatic force clearing through one status-independent successful-step finalizer. — Later CCD can remain private without changing successful clearing semantics.

### Pending Todos

None yet.

## Session Continuity

Last session: 2026-07-13T02:04:03.609Z
Stopped at: Completed 07-02-PLAN.md
Resume file: None
