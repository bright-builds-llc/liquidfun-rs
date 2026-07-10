---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "08"
subsystem: cpp-oracle-adapter
tags: [cpp, cmake, jsonl, sax, liquidfun, reset-proof, exact-float-bits]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Strict typed protocol fixtures and reviewed nlohmann/json 3.12.0 vendoring from Plans 02-05 and 02-07
  - phase: 01-oracle-provenance-and-repository-foundation
    provides: Pinned read-only LiquidFun oracle and external CMake wrapper from Plans 01-01 and 01-03
provides:
  - Process-only C++ oracle executable with protocol-only stdout and bounded stderr diagnostics
  - Bounded duplicate-aware SAX decoding into validated Phase-2 scenario domain values
  - Exact-bit semantic empty-world traces matching the Rust fixture authority byte-for-byte
  - Fresh b2World execution with complete reset proof and monotonically increasing reset epochs
affects: [02-09, 02-10, cpp-oracle, process-supervisor, differential-round-trip]

tech-stack:
  added: []
  patterns: [bounded SAX boundary parsing, deterministic JSONL encoding, fresh-world request isolation, exact-bit and length-prefixed SHA-256 compatibility]

key-files:
  created:
    - tools/reference/src/main.cpp
    - tools/reference/src/protocol.hpp
    - tools/reference/src/protocol.cpp
    - tools/reference/src/protocol_bits.cpp
    - tools/reference/src/oracle_adapter.hpp
    - tools/reference/src/oracle_adapter.cpp
    - tools/reference/tests/protocol_tests.cpp
  modified:
    - tools/reference/CMakeLists.txt
    - reference/source-map.toml

key-decisions:
  - "Parse untrusted input through a bounded custom SAX event sink, not a nlohmann mutable JSON DOM, so duplicate members and allocation limits fail before b2World construction."
  - "Keep every request scoped to a fresh b2World and emit trace_end only after world destruction, mapping cleanup, reset verification, and epoch increment."
  - "Derive the startup handshake from the pinned revision, repository-owned adapter source digest, CMake preset, compiler, target, flags, and sanitizer mode without modifying upstream."
  - "Split exact float-bit and SHA-256 compatibility logic into protocol_bits.cpp after the Bright Builds file-size simplification trigger fired."

patterns-established:
  - "Native boundary: one newline-complete request is validated into domain structs before any upstream API call."
  - "Output boundary: complete bounded records are written to stdout one at a time with an immediate flush; exceptions reach stderr and a nonzero exit only."
  - "Cross-language evidence: accepted request and trace fixtures must match C++ encoding, scenario hashes, build hashes, payload hashes, exact bits, and reset proof byte-for-byte."

requirements-completed:
  - COMP-04
  - COMP-05
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T09:08:05Z

duration: 26 min
completed: 2026-07-10
---

# Phase 2 Plan 08: Strict C++ Empty-World Oracle Adapter Summary

**A bounded duplicate-aware C++ JSONL process now executes the shared empty-world scenario in a fresh pinned `b2World`, emits exact semantic trace bytes, and proves reset epochs across reuse.**

## Performance

- **Duration:** 26 min
- **Started:** 2026-07-10T08:42:28Z
- **Completed:** 2026-07-10T09:08:05Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Registered warning-clean C++17 `liquidfun-reference` and `liquidfun-reference-protocol-tests` targets outside the read-only submodule, with private linkage to the pinned `Box2D` target and CTest registration.
- Added newline, byte, depth, string, collection, identifier, member, kind, version, source, command, checkpoint, reference, ordering, and empty-entity validation before upstream execution.
- Added deterministic handshake, trace-begin, checkpoint, and trace-end encoders whose exact scenario, identity, checkpoint-payload, float-bit, world-count, and reset fields match the Rust protocol authority.
- Added fresh-world request execution, public-API semantic counting, output bounds, protocol-only stdout flushing, bounded diagnostics, and reset epochs proven as 1 then 2 under reuse.
- Added source-map records for the repository-authored adapter files that are informed by the pinned public `b2World` API without copying upstream implementation or exposing private layout.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement the strict C++ protocol loop and empty-world adapter** - `ed5def4` (`feat`)

## Files Created/Modified

- `tools/reference/CMakeLists.txt` - Registers C++17 executable/library/test targets, keeps upstream headers read-only and system-scoped, and derives truthful local build fields.
- `tools/reference/src/main.cpp` - Emits one startup handshake, processes sequential newline-complete requests, flushes protocol records, and reports exceptions only to stderr.
- `tools/reference/src/protocol.hpp` - Defines bounded protocol constants and strict engine-neutral C++ domain records.
- `tools/reference/src/protocol.cpp` - Implements duplicate-aware bounded SAX parsing, cross-field validation, and deterministic JSONL encoding.
- `tools/reference/src/protocol_bits.cpp` - Implements exact IEEE-754 conversion and Rust-compatible length-prefixed SHA-256 identities without raw-memory casts.
- `tools/reference/src/oracle_adapter.hpp` - Defines the fresh-world adapter and reset-proven trace result.
- `tools/reference/src/oracle_adapter.cpp` - Steps a scoped `b2World`, captures semantic zero counts, destroys request state, and emits reset-proven traces.
- `tools/reference/tests/protocol_tests.cpp` - Covers every permanent rejected fixture class, parser bounds, exact cross-language bytes, references, stdout framing, and two-request reset isolation.
- `reference/source-map.toml` - Records pinned public-API provenance and no-copy alteration classifications for the adapter interface and implementation.

## Decisions Made

- Followed `standards/core/architecture.md` by parsing once at the process boundary into validated domain values and keeping `main.cpp` as a thin imperative shell.
- Followed `standards/core/testing.md` with focused Arrange/Act/Assert native tests, including byte-for-byte accepted fixture evidence and independently classified rejection cases.
- Followed `standards/core/code-shape.md` by moving the cohesive exact-bit/hash core out of the initial 805-line protocol file; `protocol.cpp` now remains focused on bounded parsing and encoding.
- Kept the adapter on public `b2World`, body, fixture, and particle accessors only; no pointer, dense index, object byte, STL value, exception, or private layout enters the wire contract.

## Verification Evidence

- TDD RED: CMake configured successfully and the native test target failed at link on the intentionally missing parser/adapter definitions before implementation.
- `cmake --preset oracle-debug`, `cmake --build --preset oracle-debug --target liquidfun-reference-protocol-tests liquidfun-reference`, and `ctest --test-dir ../../target/reference/oracle-debug --output-on-failure` passed.
- Native tests matched the accepted Rust request and all four trace records byte-for-byte, including scenario SHA-256, identity SHA-256, checkpoint payload SHA-256, exact time bits, zero counts, and reset proof.
- A reused live process emitted one handshake and reset epochs 1 then 2; malformed duplicate input exited nonzero after the handshake with no trace-looking success record.
- `cargo xtask package verify` and `cargo xtask provenance check` passed, preserving the Cargo-only consumer boundary and validating the two new source-map records.
- Vendored `json.hpp` and `LICENSE.MIT` checksums remain valid.
- The required ordered repository gate passed before the task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Acceptance scans found no `reinterpret_cast`, `memcpy`, `void *`, `printf`, stdout diagnostic mixing, stderr trace mixing, placeholder, TODO, or FIXME pattern in repository-owned adapter sources.
- `git -C third_party/liquidfun status --short` remained empty and scoped `git diff --check` passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Isolated warnings from read-only upstream headers**

- **Found during:** Task 1 native GREEN build
- **Issue:** Repository-owned targets correctly enabled warning denial, but Apple Clang surfaced a legacy deprecation warning from the untouched pinned `b2ParticleColor` header and blocked compilation.
- **Fix:** Marked only the upstream Box2D include directory as a CMake `SYSTEM` include while retaining `-Wall -Wextra -Wpedantic -Werror` for all repository-owned C++ sources.
- **Files modified:** `tools/reference/CMakeLists.txt`
- **Verification:** Both native targets compile warning-clean, CTest passes, and the submodule remains byte-for-byte clean.
- **Committed in:** `ed5def4`

**2. [Rule 1 - Bug] Corrected stale GSD progress presentation**

- **Found during:** Plan metadata update
- **Issue:** `state update-progress` correctly computed 68% only in frontmatter and `roadmap update-plan-progress 02` reported eight summaries without changing the stale body values of 63% and 7/14.
- **Fix:** Synchronized the human-readable state progress bar and Phase-2 roadmap row with the disk-derived values returned by the GSD tools.
- **Files modified:** `.planning/STATE.md`, `.planning/ROADMAP.md`
- **Verification:** Eight Phase-2 summaries exist, the roadmap reports 8/14, and both state progress representations report 68%.
- **Committed in:** Plan metadata commit

***

**Total deviations:** 2 auto-fixed (1 blocking build-boundary issue, 1 workflow-tool bug)
**Impact on plan:** The fixes preserve strict local warning enforcement, the read-only upstream contract, and internally consistent GSD progress without changing adapter scope.

## Issues Encountered

- TDD RED was observed but not committed because repository policy requires the complete ordered Rust gate to pass before every commit. The completed GREEN task was committed atomically after all native, package, provenance, acceptance, and Rust checks passed.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for Plan 02-09 to harden immutable configured build identity, allowlisted xtask integration, and fail-fast sanitizer configuration around the working adapter.
- Ready for Plan 02-10 to supervise the long-lived process, validate startup/trace sequencing, classify poisoned sessions, and exercise request budgets.
- No parser, semantic encoding, reset isolation, CMake target, provenance, package-isolation, or upstream-cleanliness blocker remains.

## Self-Check: PASSED

- All nine implementation/provenance files listed in this summary exist.
- Task commit `ed5def4` exists and contains exactly the nine scoped Plan 02-08 files.
- Summary lifecycle metadata and all three requirement IDs match Plan 02-08.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
