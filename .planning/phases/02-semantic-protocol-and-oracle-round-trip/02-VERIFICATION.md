---
phase: 02-semantic-protocol-and-oracle-round-trip
verified: 2026-07-10T13:21:59Z
status: passed
score: "13/13 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T13:21:59Z
lifecycle_validated: true
overrides_applied: 0
requirements_checked:
  - COMP-03
  - COMP-04
  - COMP-05
  - COMP-06
  - COMP-07
  - COMP-08
  - COMP-09
  - DOCS-05
gaps: []
human_verification: []
---

# Phase 2: Semantic Protocol and Oracle Round Trip Verification

**Phase Goal:** Establish the semantic scenario/trace contract and prove an
isolated Rust-to-C++ oracle round trip before subsystem comparisons accumulate.

**Verified:** 2026-07-10T13:21:59Z  
**Status:** passed  
**Re-verification:** No — initial goal-backward verification after the Phase 2
code-review fixes

## Goal Achievement

The five roadmap success criteria and the detailed truths from all fourteen
plans reduce to the thirteen observable must-haves below. The roadmap criteria
are covered respectively by truths 2-3, 1/4-6, 7, 8-10, and 11-13.

### Observable Truths

| # | Truth | Status | Evidence |
| ---: | --- | --- | --- |
| 1 | Private protocol and differential tooling remains outside the published Cargo consumer boundary. | VERIFIED | `liquidfun-test-protocol` and `liquidfun-differential` are unpublished non-default workspace packages; `cargo tree -p liquidfun --edges normal` contains only `liquidfun`; `cargo xtask package verify` built and tested the seven-entry package outside the repository; a detached worktree with an uninitialized submodule passed Cargo-only `cargo xtask check`. |
| 2 | Contributors can express and strictly validate a bounded, independently versioned named or seeded scenario with exact float bits, typed semantic identities, ordered commands, and unique checkpoint requests. | VERIFIED | `scenario.rs`, `codec.rs`, `ids.rs`, and `limits.rs` parse one bounded JSONL record into invariant-bearing types. Focused named/seeded, N/N+1, exact-bit, duplicate/reference/order, empty-phase, and fixture tests passed. |
| 3 | Schemas, the tolerance profile, and accepted/rejected fixtures are deterministic read-only presentations of stricter typed authority. | VERIFIED | Four schema/profile tests and nine fixture tests passed, including byte-stable re-encoding, explicit version axes, closed records, reset proof, and distinct duplicate/unknown/partial/version/limit rejection categories. Repeated aggregate checks preserved the tracked evidence digest. |
| 4 | The repository-owned C++ oracle is a strict process boundary over the pinned read-only source, not an FFI, raw-memory, or layout boundary. | VERIFIED | `liquidfun-reference` is built outside the submodule and privately links Box2D. The C++ boundary incrementally bounds stdin, performs duplicate-aware typed parsing, constructs a fresh `b2World`, emits semantic JSONL only, and exposes no pointer/raw-memory representation. Debug and ASan/UBSan CTest both passed; the submodule remained clean. |
| 5 | The private native Rust adapter and the separately spawned C++ oracle execute the same empty-world request and produce complete compatible semantic traces. | VERIFIED | Fresh real `oracle-debug` one-shot comparison returned Match with C++/Rust epoch 1. Debug reuse returned two Matches with epochs 1/2. Reviewed replay returned Match. The native adapter tests also passed exact checkpoint, zero-count, time-bit, identity, and reset assertions. |
| 6 | One bounded supervisor safely provides one-shot, finite reuse, and sanitizer execution with provenance handshake, one request in flight, concurrent drains, reset proof, poison, kill, wait, reap, and output bounds. | VERIFIED | Eight supervisor-failure tests passed, including timeouts, exits/signals, sanitizer markers, malformed/oversized/over-limit output, identity/sequence/reset errors, bounded stderr, pipe pressure, and concurrent over-limit stderr. Fresh ASan/UBSan one-shot matched; the reused sanitizer profile matched two requests with epochs 1/2. |
| 7 | Semantic comparison is compatibility-gated, exact for discrete fields, field-specific for floats, and order-preserving except for explicitly typed set/multiset values. | VERIFIED | Eleven comparator tests passed exact IDs/counts, all four float policies at and beyond thresholds, NaN/infinity/signed-zero rules, ordered/set/multiset semantics, provenance-before-values, and physics mismatch separation. No global epsilon or generic JSON-path policy is used. |
| 8 | A mismatch is localized deterministically to its first checkpoint, phase, typed path, and mismatch kind, while non-semantic failures remain typed harness failures. | VERIFIED | First-divergence and stable-signature tests passed. `HarnessFailureKind` contains the process/protocol/provenance taxonomy and excludes physics mismatch. CLI tests proved distinct machine result kinds and exit codes. |
| 9 | Exact replay and deterministic minimization preserve source metadata, scenario validity, canonical bytes, and the same failure signature. | VERIFIED | Exact-request replay preserved serialized source metadata. Six pure minimizer tests passed reduction, invalid-candidate rejection, changed-signature rejection, deterministic order, and attempt/deadline bounds. The CLI minimization test persisted a smaller canonical scenario with the same signature. |
| 10 | Failures and reviewed reference evidence are bounded, attributable, replayable, and promoted only through a confined explicit-review transaction. | VERIFIED | Failure-bundle tests persisted bounded hash-indexed evidence and bound second-request failures/mismatches to the executed request and validated session identity. Thirteen fixture lifecycle tests passed staging, replay/diff, explicit review, same-signature regression evidence, path/symlink/race checks, no-clobber promotion, and post-commit manifest integrity. Provenance validates one reviewed trace. |
| 11 | Contributors have allowlisted, structured xtask commands and thin just aliases for compare, replay, minimize, lifecycle, and aggregate verification without arbitrary paths or hidden C++ requirements. | VERIFIED | Ten differential xtask tests passed canonical argument forwarding, allowlist rejection, and child failure propagation. Real xtask commands passed. `just check` passed, and the detached no-submodule worktree entered labeled Cargo-only mode and completed package, protocol, docs, and artifact-provenance checks. |
| 12 | `ARCHITECTURE.md` and `TESTING.md` accurately define dependency direction, protocol versions, diagnosis, review/minimization, expensive evidence, and the empty-world-only maturity boundary. | VERIFIED | `cargo xtask docs check` reports all twelve testing layers with complete DOCS-05 contracts; ten positive/negative docs tests and `mdformat --check` passed. Documentation matches the executed command shapes and does not claim broad physics parity or completed deferred fuzz/Miri/coverage work. |
| 13 | CI separates submodule-free Rust verification from real oracle and sanitizer evidence, remains read-only, and retains bounded failure artifacts. | VERIFIED | `actionlint` passed. Cargo CI contains private Rust protocol/comparator/minimizer/supervisor/docs/provenance coverage without CMake; oracle CI contains debug one-shot/reuse/replay and fail-fast sanitizer/reset commands. Workflow tests prove only code-change runs are cancellable and missing failure evidence fails the upload contract. |

**Score:** 13/13 truths verified

## Required Artifacts and Key Links

`gsd-tools verify artifacts` passed all 43 declared artifacts. It directly
verified 30 of 32 declared key links. The two pattern-check misses were manually
verified: root `Cargo.toml` still declares `default-members =
["crates/liquidfun"]`, and manifest-v2 `artifact_kind` validation lives in the
cohesive `tools/xtask/src/provenance/artifact.rs` child module reached from
`provenance.rs`.

| Plan | Artifacts | Key links | Result |
| --- | ---: | ---: | --- |
| 02-01 | 2/2 | 3/3 | VERIFIED; sole-default-member link confirmed manually. |
| 02-02 | 3/3 | 2/2 | VERIFIED. |
| 02-03 | 3/3 | 3/3 | VERIFIED. |
| 02-04 | 3/3 | 2/2 | VERIFIED. |
| 02-05 | 4/4 | 3/3 | VERIFIED. |
| 02-06 | 4/4 | 2/2 | VERIFIED. |
| 02-07 | 2/2 | 2/2 | VERIFIED; vendor hashes also passed. |
| 02-08 | 3/3 | 2/2 | VERIFIED by build and CTest. |
| 02-09 | 3/3 | 2/2 | VERIFIED by structured configure/build and identity tests. |
| 02-10 | 4/4 | 3/3 | VERIFIED by real and injected execution. |
| 02-11 | 1/1 | 1/1 | VERIFIED by lifecycle tests. |
| 02-12 | 2/2 | 1/1 | VERIFIED; child-module link confirmed manually and by provenance check. |
| 02-13 | 3/3 | 2/2 | VERIFIED by xtask tests and real commands. |
| 02-14 | 6/6 | 4/4 | VERIFIED by docs/workflow checks. |

## Requirements Coverage

| Requirement | Status | Executable and implementation evidence |
| --- | --- | --- |
| COMP-03 | SATISFIED | Strict `ScenarioRequestRecord`/`ValidatedScenarioV1`, named and seeded sources, independent versions, exact bits, typed IDs, ordered commands/checkpoints, N/N+1 limits, and accepted/rejected fixture tests. Phase-2 scenario schema intentionally permits only the empty entity list; nonempty physics definitions remain later-phase breadth. |
| COMP-04 | SATISFIED | The same validated request produced real Rust/C++ Matches in one-shot, reuse, replay, and sanitizer modes. The C++ executable is process-isolated and its protocol contains semantic records only. |
| COMP-05 | SATISFIED | Handshake/trace and manifest-v2 evidence bind scenario/schema/tolerance versions and hashes, pinned upstream, adapter/source digest, compiler/target/flags, source/seed, payload, notices, review, and stable identity. `cargo xtask provenance check` passed twice with one reviewed artifact. |
| COMP-06 | SATISFIED | Comparator tests prove exact discrete values and exhaustive ExactBits/Absolute/AbsoluteRelative/ULP policy behavior, including special floats and thresholds. |
| COMP-07 | SATISFIED | Ordered values remain ordered; only typed `Set` and `Multiset` paths canonicalize with stable keys/tie-breakers. Checkpoint ordering is exercised; future solver/callback/destruction payload breadth remains explicitly deferred. |
| COMP-08 | SATISFIED | Named and exact serialized replay work; seeded source metadata survives decoding/replay; first-divergence signatures are stable; CLI minimization persists a smaller same-signature scenario; reviewed/minimized evidence uses the replay/review/promotion lifecycle. |
| COMP-09 | SATISFIED | The exhaustive harness taxonomy and injected supervisor tests classify startup/request timeout, exits/signals, sanitizers, framing/size/sequence/request/identity/provenance/reset and adapter failures separately from physics mismatch, with bounded bundles. |
| DOCS-05 | SATISFIED | `TESTING.md` contains the machine-checked twelve-layer contract, protocol/diagnosis/reference-review/minimization guidance, and actionable current/deferred fuzz, Miri, sanitizer, coverage, local, PR, scheduled, and manual-release placement. |

No Phase 2 requirement mapped in `REQUIREMENTS.md` is orphaned from the plans.

## Fresh Verification Evidence

| Command or check | Result |
| --- | --- |
| `gsd-tools verify lifecycle 2 --expect-id 2-2026-07-10T04-59-34 --expect-mode yolo --require-plans --allow-stale-verification` | Passed; context, 14 plans, and 14 summaries share valid lifecycle provenance. |
| `cargo fmt --all --check` | Passed. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed. |
| `cargo build --workspace --all-targets --all-features` | Passed. |
| `cargo test --workspace --all-features` | Passed: 174 Rust tests plus doctests, including the review-fix regressions. |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` | Passed. |
| Debug and ASan/UBSan configure/build plus CTest | Both presets passed the registered native protocol test; upstream checkout remained clean. |
| Debug one-shot/reuse/replay | Match; one-shot/replay epoch 1, reuse C++/Rust epochs 1/2. |
| ASan/UBSan one-shot and sanitizer profile | Match; sanitizer reused one child for two requests with C++/Rust epochs 1/2. |
| Focused comparator/minimizer/supervisor/failure-bundle/fixture tests | Passed: 11 comparator, 6 minimizer, 8 supervisor, 13 lifecycle tests plus CLI persistence/identity cases. |
| `cargo xtask docs check` and docs tests | Passed: 12 complete layers and 10 contract tests. |
| `cargo xtask provenance check` twice | Passed with oracle `7f204021…` and one reviewed artifact. |
| `cargo xtask package verify` | Passed: seven entries built/tested outside the repository. |
| `cargo xtask check` and `just check` | Passed all initialized-repository gates. |
| Detached worktree with uninitialized submodule: `cargo xtask check` | Passed in explicit Cargo-only mode, including package, protocol, docs, and artifact provenance. |
| `actionlint`, `mdformat --check`, nlohmann `SHA256SUMS`, and `git diff --check` | Passed. |
| Repeated read-only checks | Protocol, scenarios, reference evidence, compatibility data, and published-crate aggregate digest stayed `831a8e0c7628f8f6a33750d12a8abe3147da4a56359f41e5448d1357cc181edb`. |

## Disconfirmation and Residual Risks

- A normal round-trip integration test can explicitly skip when the real oracle
  executable is absent. This did not create false evidence here: both native
  presets were freshly configured/built, CTest ran, and five real xtask
  comparison/replay commands executed successfully.
- Phase 2 proves an empty-world seam. Nonempty semantic entity definitions and
  real solver/callback/destruction sequences are not runtime-compatible physics
  claims; they are intentionally deferred to the object-model and later rigid
  phases. The current contract nevertheless fixes typed IDs, ordering policy,
  and strict extension boundaries.
- Failure-bundle success and boundedness are tested, while low-level disk-full
  or permission failure injection is not. Persistence errors propagate visibly,
  and oracle CI is configured to fail rather than silently ignore a missing
  failure bundle.
- Local native evidence used CMake 3.27.9 and Apple Clang 21.0.0, which xtask
  correctly reported as noncanonical. The canonical Linux workflow pins CMake
  4.3.3, Ninja 1.13.2, and Clang 22.1.8; that CI environment was not reproduced
  locally.

These are explicit scope or environment limitations, not Phase 2 goal gaps.
There is no subjective behavior requiring human verification.

## Human Verification Required

None.

## Gaps Summary

No blocking gaps. The semantic contract, real isolated empty-world round trip,
comparison/failure semantics, replay/minimization/evidence lifecycle, package
isolation, and documentation contract are implemented and executable without
overstating broad physics compatibility.

_Verified: 2026-07-10T13:21:59Z_  
_Verifier: the agent (gsd-verifier)_
