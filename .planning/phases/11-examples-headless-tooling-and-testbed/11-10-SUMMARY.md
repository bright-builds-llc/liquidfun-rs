---
phase: 11-examples-headless-tooling-and-testbed
plan: "10"
subsystem: cross-process-checkpoint-protocol
tags: [jsonl, checkpoint, resolved-scenario, exact-bits, bounded-decode, schema]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "03"
    provides: Deterministic catalog resolution and exact canonical resolved bytes
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "08"
    provides: Owned semantic observations and closed profile names
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "09"
    provides: Renderer-neutral debug primitives with stable semantic keys
provides:
  - Strict run requests carrying exact resolved bytes, content hash, and pinned provenance requirements
  - Bounded canonical checkpoints with structural and numeric observations, occurrences, sets, debug primitives, and profile names
  - Byte-stable closed JSON Schema presentation for run requests and checkpoints
affects: [phase11-headless-runner, phase11-testbed, phase11-evidence, differential-harness]
tech-stack:
  added: []
  patterns:
    - Decode exact resolved bytes and verify redundant identity instead of rerunning generators across process boundaries
    - Preserve source-significant order and canonicalize only explicitly unordered semantic sets
key-files:
  created:
    - crates/liquidfun-test-protocol/src/catalog/wire.rs
    - crates/liquidfun-test-protocol/src/checkpoint.rs
    - crates/liquidfun-test-protocol/src/checkpoint/observation.rs
    - crates/liquidfun-test-protocol/src/checkpoint/primitive.rs
    - crates/liquidfun-test-protocol/src/schema/checkpoint.rs
    - crates/liquidfun-test-protocol/tests/checkpoint_protocol.rs
    - protocol/schemas/checkpoint-v1.schema.json
  modified:
    - crates/liquidfun-test-protocol/src/catalog.rs
    - crates/liquidfun-test-protocol/src/codec.rs
    - crates/liquidfun-test-protocol/src/lib.rs
    - crates/liquidfun-test-protocol/src/schema.rs
    - crates/liquidfun-test-protocol/src/schema/tests.rs
key-decisions:
  - "Cross-process execution accepts exact canonical resolved bytes plus their SHA-256 identity and never reconstructs a run by rerunning a generator."
  - "Checkpoint equality contains semantic structure, exact float bits paired with closed Phase 4 policy paths, and profile names only; wall-clock durations and renderer coordinates remain outside the contract."
  - "Primitive ordering is explicit per record: source-significant order is retained while only declared canonicalized subsets require stable semantic-key order."
patterns-established:
  - "Wire acceptance: strict JSONL framing, typed bounded decode, hash verification, redundant identity verification, then semantic validation."
  - "Checkpoint boundary: action or logical-step ordinal identities replace frame, wall-time, UI, and private-storage coordinates."
requirements-completed: [RIGD-10, EXMP-02, EXMP-03, EXMP-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T02:02:11Z
duration: 20 min
completed: 2026-07-21
---

# Phase 11 Plan 10: Cross-Process Checkpoint Protocol Summary

**Strict JSONL run requests now carry exact resolved scenario bytes and pinned provenance, while canonical checkpoints encode bounded semantic observations, exact-bit policy-bound numbers, stable debug primitives, explicit ordering, and profile names without private or renderer state.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-07-22T01:42:09Z
- **Completed:** 2026-07-22T02:02:11Z
- **Tasks:** 1
- **Files modified:** 12

## Accomplishments

- Added a strict `CatalogRunRequest` that transports the exact canonical resolved bytes, verifies their asserted SHA-256, checks redundant catalog/generator/settings identity, and carries required build, limits-profile, and evidence-tier provenance.
- Added a versioned `CanonicalCheckpoint` with strict count and aggregate bounds, finite exact-bit values, closed Phase 4 numeric policy paths, duplicate-aware semantic identities, explicit source-significant versus canonicalized ordering, and renderer-neutral debug primitives.
- Added adversarial integration coverage for unknown fields and kinds, duplicate members and identities, unsupported versions, wrong hashes, non-finite values, dangling checkpoint identities, order violations, forbidden private/render fields, and the first record beyond a reviewed bound.
- Added a deterministic, recursively closed, newline-terminated checkpoint schema presentation whose tracked bytes are tested against the typed renderer.

## TDD Evidence

- **RED:** `cargo test -p liquidfun-test-protocol --test checkpoint_protocol` failed with unresolved imports for the new run-request/checkpoint API, establishing the missing protocol boundary. The initial test also exposed and corrected a fixture-construction mistake that attempted to call `push` directly on `serde_json::Value`.
- **GREEN:** All 6 focused checkpoint protocol tests and all 11 existing fixture tests pass. Both schema byte-stability and recursively closed-record assertions pass.
- **REFACTOR:** Primitive observation and geometry vocabulary were separated into focused checkpoint submodules, and strict codec error classification was normalized case-insensitively without changing the redacted public error surface.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement the resolved execution and semantic checkpoint protocol** - `0d1c89e` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/catalog/wire.rs` - Exact resolved-byte run request, provenance requirements, and strict decode/identity validation.
- `crates/liquidfun-test-protocol/src/checkpoint.rs` - Versioned checkpoint envelope, bounded decode, semantic identity/order/finite validation, and redacted error categories.
- `crates/liquidfun-test-protocol/src/checkpoint/observation.rs` - Structural, numeric, occurrence, and explicitly unordered set observations.
- `crates/liquidfun-test-protocol/src/checkpoint/primitive.rs` - Exact-bit engine-neutral debug primitive mirror with stable semantic keys and explicit ordering declarations.
- `crates/liquidfun-test-protocol/src/schema/checkpoint.rs` and `protocol/schemas/checkpoint-v1.schema.json` - Typed renderer and tracked deterministic schema presentation.
- `crates/liquidfun-test-protocol/tests/checkpoint_protocol.rs` - Round-trip, negative, boundary, and forbidden-field behavior coverage.
- `crates/liquidfun-test-protocol/src/catalog.rs`, `src/lib.rs`, `src/schema.rs`, and `src/schema/tests.rs` - Public routing and schema byte checks.
- `crates/liquidfun-test-protocol/src/codec.rs` - Case-insensitive stable classification of typed decode messages.

## Decisions Made

- Exact resolved bytes are authoritative across the process boundary. Redundant slug, version, generator, seed, and settings fields are checked for contradiction but never used to regenerate the scenario.
- Structural observations compare exactly. Numeric observations retain raw IEEE-754 bits and name one closed reviewed Phase 4 policy path so later comparators cannot invent local tolerances.
- Source-significant occurrence and primitive order remains untouched. Only explicitly declared set/canonicalized primitive regions receive stable-ID ordering rules.
- Profile instrumentation crosses the boundary as closed names only; durations and timing samples remain diagnostic-only state.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Normalized codec failure classification**

- **Found during:** Task 1 focused GREEN run
- **Issue:** Existing string classification recognized lowercase `unsupported version`, but the catalog version error rendered `UnsupportedVersion`, incorrectly categorizing the strict failure as malformed.
- **Fix:** Normalize decoder messages to lowercase before matching all stable codec categories.
- **Files modified:** `crates/liquidfun-test-protocol/src/codec.rs`
- **Verification:** The wrong catalog schema version adversarial test now reports `CodecErrorKind::UnsupportedVersion`; existing fixture classification tests still pass.
- **Commit:** `0d1c89e`

**2. [Rule 2 - Missing Critical Functionality] Added tracked checkpoint schema bytes**

- **Found during:** Task 1 schema presentation work
- **Issue:** The plan required byte-checking a schema presentation but did not list a tracked checkpoint schema file among task paths.
- **Fix:** Added `protocol/schemas/checkpoint-v1.schema.json` and registered it in the existing byte-stability and recursively closed-record tests.
- **Files modified:** `protocol/schemas/checkpoint-v1.schema.json`, `crates/liquidfun-test-protocol/src/schema/tests.rs`
- **Verification:** Both checkpoint schema presentation tests pass.
- **Commit:** `0d1c89e`

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical functionality).
**Impact on plan:** Both changes are required for stable strict failure semantics and an independently auditable schema presentation; no architectural scope was added.

## Issues Encountered

- The first schema closed-record check correctly rejected generic nested `type: object` presentations. Their renderer and tracked bytes were tightened with `additionalProperties: false`, empty property maps, and explicit required lists before the gate.
- The shared worktree contained four unrelated pre-existing edits. They remained unstaged and were not committed or reverted.

## Security Verification

- All incoming byte and collection sizes are bounded during typed decode before semantic acceptance, including exact resolved bytes, observations, occurrences, sets, primitives, vertices, labels, and profiles.
- Unknown fields/kinds, duplicate JSON members, duplicate semantic identities, unsupported versions, wrong hashes, contradictory identities, non-finite numbers, invalid primitive geometry, dangling checkpoint identities, and ordering violations fail closed with redacted stable categories.
- The protocol exposes semantic IDs, exact bits, and inert bounded labels only. It contains no pointer, arena slot, proxy index, dense row, renderer pixel, file path, process command, markup, secret, authentication value, or timing duration.
- No network endpoint, file-access path, authentication boundary, schema migration, dependency, foreign runtime, or `unsafe` surface was introduced.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Known Stubs

None.

## Requirements Status

Plan 11-10's `RIGD-10`, `EXMP-02`, `EXMP-03`, and `EXMP-05` mappings are achieved at the protocol boundary and retained in summary frontmatter. Their global requirement checkboxes remain intentionally unchanged until later Phase 11 integration and evidence plans verify the complete end-to-end requirement scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The headless runner and testbed can consume the same exact resolved run identity and checkpoint vocabulary without generator drift or renderer coupling.
- Later evidence plans can compare structural fields exactly and numeric fields only through the named closed Phase 4 policies.
- Phase execution state must continue to respect the earliest incomplete wave-order plan rather than treating Plan 11-10's out-of-order completion as phase completion.

## Self-Check: PASSED

- Confirmed all seven created and five modified Plan 11-10 files exist.
- Confirmed task commit `0d1c89e` exists and contains only the twelve Plan 11-10 artifacts.
- Confirmed the 6 focused checkpoint protocol tests, 11 fixture tests, 2 schema presentation tests, focused deny-warnings Clippy, and exact ordered full-workspace format, Clippy, build, test, and doctest gate pass with `/tmp/liquidfun-rs-phase11-11-10`.
- Confirmed no known stub or unplanned threat surface was introduced and all four unrelated shared-tree edits remain unstaged.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
