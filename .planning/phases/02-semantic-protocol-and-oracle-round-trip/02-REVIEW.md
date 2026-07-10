---
phase: 02-semantic-protocol-and-oracle-round-trip
reviewed: "2026-07-10T12:05:47Z"
depth: standard
files_reviewed: 94
scope:
  - ".github/workflows/ci.yml"
  - ".github/workflows/oracle.yml"
  - "ARCHITECTURE.md"
  - "Cargo.lock"
  - "Cargo.toml"
  - "TESTING.md"
  - "THIRD_PARTY_NOTICES.md"
  - "crates/liquidfun-differential/Cargo.toml"
  - "crates/liquidfun-differential/src/canonical.rs"
  - "crates/liquidfun-differential/src/comparator.rs"
  - "crates/liquidfun-differential/src/fixtures.rs"
  - "crates/liquidfun-differential/src/fixtures/domain.rs"
  - "crates/liquidfun-differential/src/fixtures/lifecycle.rs"
  - "crates/liquidfun-differential/src/fixtures/replay.rs"
  - "crates/liquidfun-differential/src/fixtures/storage.rs"
  - "crates/liquidfun-differential/src/lib.rs"
  - "crates/liquidfun-differential/src/main.rs"
  - "crates/liquidfun-differential/src/minimizer.rs"
  - "crates/liquidfun-differential/src/report.rs"
  - "crates/liquidfun-differential/src/runner.rs"
  - "crates/liquidfun-differential/src/rust_adapter.rs"
  - "crates/liquidfun-differential/src/supervisor.rs"
  - "crates/liquidfun-differential/src/supervisor/capture.rs"
  - "crates/liquidfun-differential/src/supervisor/executable.rs"
  - "crates/liquidfun-differential/src/supervisor/failure.rs"
  - "crates/liquidfun-differential/src/supervisor/profile.rs"
  - "crates/liquidfun-differential/src/supervisor/stdio.rs"
  - "crates/liquidfun-differential/tests/comparison.rs"
  - "crates/liquidfun-differential/tests/fixture_workflow.rs"
  - "crates/liquidfun-differential/tests/fixtures/fake_oracle.rs"
  - "crates/liquidfun-differential/tests/minimizer.rs"
  - "crates/liquidfun-differential/tests/round_trip.rs"
  - "crates/liquidfun-differential/tests/rust_adapter.rs"
  - "crates/liquidfun-differential/tests/supervisor_failures.rs"
  - "crates/liquidfun-test-protocol/Cargo.toml"
  - "crates/liquidfun-test-protocol/src/codec.rs"
  - "crates/liquidfun-test-protocol/src/failure.rs"
  - "crates/liquidfun-test-protocol/src/float_bits.rs"
  - "crates/liquidfun-test-protocol/src/ids.rs"
  - "crates/liquidfun-test-protocol/src/lib.rs"
  - "crates/liquidfun-test-protocol/src/limits.rs"
  - "crates/liquidfun-test-protocol/src/provenance.rs"
  - "crates/liquidfun-test-protocol/src/scenario.rs"
  - "crates/liquidfun-test-protocol/src/scenario/reduction.rs"
  - "crates/liquidfun-test-protocol/src/scenario/tests.rs"
  - "crates/liquidfun-test-protocol/src/schema.rs"
  - "crates/liquidfun-test-protocol/src/schema/tests.rs"
  - "crates/liquidfun-test-protocol/src/tolerance.rs"
  - "crates/liquidfun-test-protocol/src/trace.rs"
  - "crates/liquidfun-test-protocol/src/trace/tests.rs"
  - "crates/liquidfun-test-protocol/tests/fixtures.rs"
  - "justfile"
  - "protocol/fixtures/accepted/empty-world-request.jsonl"
  - "protocol/fixtures/accepted/empty-world-trace.jsonl"
  - "protocol/fixtures/rejected/duplicate-member.jsonl"
  - "protocol/fixtures/rejected/oversized-id.jsonl"
  - "protocol/fixtures/rejected/partial-record.jsonl"
  - "protocol/fixtures/rejected/unknown-record-kind.jsonl"
  - "protocol/fixtures/rejected/unsupported-version.jsonl"
  - "protocol/schemas/protocol-v1.schema.json"
  - "protocol/schemas/scenario-v1.schema.json"
  - "protocol/schemas/trace-v1.schema.json"
  - "protocol/tolerances/phase2-v1.toml"
  - "reference/artifacts/manifest.toml"
  - "reference/artifacts/traces/empty-world-v1.jsonl"
  - "reference/source-map.toml"
  - "scenarios/phase-02/empty-world.json"
  - "scenarios/regressions/README.md"
  - "tools/reference/CMakeLists.txt"
  - "tools/reference/CMakePresets.json"
  - "tools/reference/src/build_identity.hpp.in"
  - "tools/reference/src/main.cpp"
  - "tools/reference/src/oracle_adapter.cpp"
  - "tools/reference/src/oracle_adapter.hpp"
  - "tools/reference/src/protocol.cpp"
  - "tools/reference/src/protocol.hpp"
  - "tools/reference/src/protocol_bits.cpp"
  - "tools/reference/tests/protocol_tests.cpp"
  - "tools/reference/vendor/nlohmann/LICENSE.MIT"
  - "tools/reference/vendor/nlohmann/SHA256SUMS"
  - "tools/reference/vendor/nlohmann/json.hpp"
  - "tools/xtask/src/differential.rs"
  - "tools/xtask/src/docs.rs"
  - "tools/xtask/src/main.rs"
  - "tools/xtask/src/provenance.rs"
  - "tools/xtask/src/provenance/artifact.rs"
  - "tools/xtask/src/provenance/artifact/trace.rs"
  - "tools/xtask/src/upstream.rs"
  - "tools/xtask/tests/differential_cli.rs"
  - "tools/xtask/tests/docs_contract.rs"
  - "tools/xtask/tests/fixtures/fake_differential_tool.rs"
  - "tools/xtask/tests/fixtures/fake_upstream_tool.rs"
  - "tools/xtask/tests/provenance_cli.rs"
  - "tools/xtask/tests/upstream_cli.rs"
findings:
  critical: 1
  warning: 10
  info: 0
  total: 11
status: issues_found
---

# Phase 2 Code Review

## Scope and standards

Reviewed the 94 non-planning files declared as created or modified by Phase 2 summaries `02-01` through `02-14`. The review applied the repository's Bright Builds architecture, code-shape, verification, testing, and Rust standards, with correctness, security, evidence integrity, and maintainability in scope. Performance-only observations and style-only preferences were excluded.

## Critical

### C1 — a lock-cleanup error can commit a manifest entry and then delete its artifact

- File: `crates/liquidfun-differential/src/fixtures/storage.rs:43`
- Issue: `update_manifest_atomically` replaces and syncs the manifest before removing its lock file, but reports an `Io` error if lock removal fails after that commit. `promote_candidate` treats every returned error as an uncommitted transaction and deletes the already-linked destination. The result is a committed manifest record whose referenced artifact has been removed; subsequent promotion is also blocked by the stale manifest record and possibly the stale lock.
- Severity: Critical
- Fix: Give the manifest transaction an explicit committed outcome. After the manifest replacement succeeds, never report the transaction as uncommitted; treat lock cleanup as separately recoverable/diagnostic, or retain/reconcile the destination on cleanup failure. Add a fault-injection test that makes lock removal fail after manifest replacement and asserts the manifest and destination cannot diverge.

## Warning

### W1 — `minimize` is accepted but never runs the minimizer

- File: `crates/liquidfun-differential/src/main.rs:310`
- Issue: `CommandConfig` stores the parsed action in `_action`, and `run` dispatches every named action through the same `run_named` path. Repository-wide call-site search finds `minimize(...)` only in minimizer tests. Consequently, `liquidfun-differential minimize ...` performs an ordinary comparison, emits the ordinary match/mismatch report, and never produces a reduced scenario or minimization status. The xtask wrapper advertises and forwards this command, so users receive successful but incorrect command behavior.
- Severity: Warning
- Fix: Dispatch on `Action` in `run`. For `Minimize`, require an initial physics mismatch, pass its exact `FailureSignature` to the deterministic minimizer, re-evaluate candidates through the differential runner, and emit/persist the bounded minimization result. Add a CLI integration test that proves the minimized canonical scenario is smaller and retains the same signature.

### W2 — reuse profiles reject otherwise-valid maximum-length request IDs

- File: `crates/liquidfun-differential/src/runner.rs:208`
- Issue: `requests_for_profile` creates the second request ID by appending `-reuse-2` to the caller's validated ID. Protocol IDs may already be 128 bytes, so a request accepted in one-shot mode can fail locally in reuse or sanitizer mode before reaching either engine. Session-profile selection should not change whether the same valid scenario request is representable.
- Severity: Warning
- Fix: Derive a deterministic bounded second ID, for example a fixed prefix plus a hash of the original ID, and validate it once. Add coverage using an exactly 128-byte request ID for both reuse and sanitizer profiles.

### W3 — native Rust build identity does not identify its source or toolchain

- File: `crates/liquidfun-differential/src/rust_adapter.rs:47`
- Issue: The native identity records literal placeholders (`repository-toolchain`, `rust-default`, and `cargo-default`), while `native_adapter_content_sha256` hashes only the package name, package version, and a fixed label. Editing the adapter implementation without changing the crate version therefore preserves both its content digest and reported build identity. Native traces from materially different Rust source, compiler versions, targets, features, or flags can be presented as the same engine build, defeating the provenance contract used to trust differential evidence.
- Severity: Warning
- Fix: Generate the native identity from build-time inputs that bind the adapter source digest, exact `rustc -vV` identity/host, target, profile, enabled features, and effective encoded flags. Inject those reviewed values at compile time, and add a test showing that changing adapter source input changes `adapter_content_sha256`.

### W4 — fixture provenance can claim a clean commit for dirty generator code

- File: `crates/liquidfun-differential/src/main.rs:233`
- Issue: `generator_revision` records only `git rev-parse HEAD`. Fixture staging does not reject tracked or untracked generator changes, and later provenance validation checks only that this commit exists. An artifact generated by uncommitted harness or adapter code can therefore be promoted with a manifest that attributes it to the clean `HEAD` commit, making the recorded generator provenance false.
- Severity: Warning
- Fix: Before staging, require a clean relevant worktree (including untracked generator/build inputs), or record and validate an explicit dirty-state/content digest alongside the revision. Add a fixture CLI test that modifies a generator source and verifies staging fails closed.

### W5 — the C++ adapter digest omits an executable parser dependency

- File: `tools/xtask/src/upstream.rs:19`
- Issue: `ADAPTER_SOURCES` hashes seven wrapper files but omits `tools/reference/vendor/nlohmann/json.hpp`, even though `protocol.cpp` compiles that header directly into the oracle. `tools/reference/CMakeLists.txt` duplicates the same incomplete list for configure-time verification. Changing the vendored parser can therefore change accepted/rejected protocol behavior while preserving `adapter_content_sha256` and the resulting build identity.
- Severity: Warning
- Fix: Define one reviewed digest manifest that includes every behavior-affecting adapter input, at minimum the vendored JSON header and relevant CMake build recipe, and use the same manifest from xtask and CMake. Add a test proving that mutating the vendored-header input changes or invalidates the configured adapter digest.

### W6 — Rust accepts a scenario that the schema and C++ oracle reject

- File: `crates/liquidfun-test-protocol/src/scenario.rs:539`
- Issue: `validate_checkpoint` copies `raw.phase` without requiring it to be nonempty. The checked-in scenario schema declares `minLength: 1`, `CheckpointRecord::new` rejects an empty phase, and the C++ decoder rejects it too. The Rust protocol authority can therefore validate an empty-phase scenario request that cannot complete a Rust trace and is rejected at the oracle boundary, turning a schema violation into a later harness failure.
- Severity: Warning
- Fix: Reject empty checkpoint phases during scenario validation with a typed `ScenarioErrorKind`, and add the same fixture to the Rust schema/runtime and C++ cross-language rejection tests.

### W7 — the C++ input boundary enforces its byte limit only after unbounded allocation

- File: `tools/reference/src/main.cpp:35`
- Issue: `std::getline` reads an entire request into an unbounded `std::string`; only afterward does `decode_scenario_request` check the 1 MiB record limit. A malformed or direct caller can force memory growth well beyond the reviewed protocol bound before rejection, so the advertised resource limit is not actually enforced at the process boundary.
- Severity: Warning
- Fix: Replace `std::getline` with a bounded incremental line reader that stops and fails as soon as `kMaximumRecordBytes` would be exceeded. Add a process-level test that streams an oversized line and verifies bounded rejection without reading the remainder into memory.

### W8 — promised differential failure artifacts are never created

- File: `.github/workflows/oracle.yml:154`
- Issue: The sanitizer lane uploads `target/differential/failures` with `if-no-files-found: ignore`, and `TESTING.md` names that directory as the source of bounded request, identity, report, and stderr evidence. No differential runner, renderer, or supervisor path creates or writes this directory. On the failures where diagnostics matter most, CI silently uploads nothing.
- Severity: Warning
- Fix: Persist a bounded failure bundle for every harness failure and physics mismatch before returning the exit code, then make the artifact upload fail when a failed differential step produced no bundle. Add an integration test that injects a harness failure and verifies the expected bounded files and hashes.

### W9 — scheduled and manual evidence runs can be cancelled mid-publication

- File: `.github/workflows/oracle.yml:14`
- Issue: Workflow-level concurrency sets `cancel-in-progress: true` for every event. Scheduled and manually dispatched oracle/sanitizer evidence runs on the same ref can therefore cancel one another or be cancelled by another run, contrary to the repository's explicit rule that scheduled/release evidence generation must not be interrupted halfway through artifact publication.
- Severity: Warning
- Fix: Enable cancellation only for superseded pull-request (and, if desired, ordinary push) runs; keep `schedule` and `workflow_dispatch` non-cancelling, using event-aware groups if concurrent manual evidence runs must coexist.

### W10 — concurrent stderr can bypass the total-output budget

- File: `crates/liquidfun-differential/src/supervisor.rs:530`
- Issue: The stdout and stderr workers increment the shared byte counter and then enqueue progress events from different threads, while `run_request` returns immediately when it receives `trace_end`. A valid stdout record can therefore arrive before a large stderr worker's queued progress event. One-shot teardown never performs a final total-output check, and a reuse request takes a new baseline after those bytes have already accumulated, so output exceeding the per-request cap can be accepted and hidden by the next baseline.
- Severity: Warning
- Fix: Make request completion reconcile the authoritative shared byte count against the request's original baseline after worker output is synchronized, and carry the previous request's accounting boundary forward rather than sampling a fresh baseline that forgives late bytes. Add a deterministic fake-oracle test that emits a valid trace concurrently with over-limit stderr and assert `TotalOutputExceeded` in one-shot and reuse modes.

## Info

None.
