---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T05:05:08.196Z
---

# Phase 2: Semantic Protocol and Oracle Round Trip - Context

**Gathered:** 2026-07-10
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Establish a private, engine-neutral scenario and semantic-trace contract, then prove the same bounded empty-world scenario can run through native Rust and the pinned process-isolated C++ oracle with trustworthy provenance, failure classification, comparison, replay, and first-divergence diagnostics. This phase establishes the reusable harness seam; it does not implement broad physics behavior, settle the public Rust object model, or define final subsystem tolerance values.

</domain>

<decisions>
## Implementation Decisions

### Scenario and protocol contract

- **D-01:** Use a private UTF-8 JSON Lines protocol with one complete, validated scenario request per input record and typed streamed output records (`trace_begin`, ordered checkpoints, and `trace_end`). Stdout is protocol-only and every record ends with a newline; human diagnostics go to stderr.
- **D-02:** Version the transport envelope, scenario schema, trace schema, and tolerance profile independently with explicit integer versions. Reject unsupported versions, unknown record kinds, duplicate members, malformed record sequences, and trailing or partial records instead of guessing compatibility.
- **D-03:** Parse raw records into strict engine-neutral domain types before execution. Scenarios carry a stable scenario ID, optional reproducible seed, deterministic typed semantic entity IDs, explicit creation order, bounded commands, and uniquely identified checkpoint requests. Rust handles and C++ pointers or indices never cross the protocol.
- **D-04:** Encode authoritative `f32` inputs and outputs as exact `u32` bit patterns. Decimal rendering may appear only as diagnostic metadata and never becomes comparison input.
- **D-05:** Enforce named limits at every boundary: bytes before parsing; nesting, strings, and collections while parsing; entity, command, step, checkpoint, and observable counts after parsing; and per-record, per-trace, stderr-retention, and total-run output limits while executing. Limit failures are typed harness failures.
- **D-06:** Keep generated schemas and representative protocol fixtures checked in and deterministic, but treat Rust/C++ typed validation as the authority for cross-field invariants such as references, uniqueness, ordering, and aggregate bounds.

### Oracle lifecycle, provenance, and failure taxonomy

- **D-07:** Implement one process supervisor with an explicit request budget. A budget of one provides maximum isolation for focused reproduction and the initial proof; a finite budget provides bounded process reuse for corpora and sanitizer lanes. Requests remain sequential with exactly one in flight.
- **D-08:** Require a startup handshake before scenarios are accepted. It reports supported protocol/schema versions and build identity containing the pinned oracle revision, adapter revision or digest, CMake preset, compiler identity/version, target, and effective flags. Repeat a stable identity hash in every trace and validate it before physics comparison.
- **D-09:** Drain stdout and stderr concurrently, cap retained diagnostics, flush every protocol record, and always kill then wait/reap a timed-out or poisoned child. A poisoned session is never reused; the exact request, identity, exit status, and bounded stderr evidence are preserved before any independent continuation.
- **D-10:** Classify timeout, signal/nonzero exit, sanitizer report, unexpected EOF, malformed/oversized record, request-ID mismatch, schema incompatibility, wrong provenance, and adapter reset failure as harness failures. Only two fully validated traces may reach semantic comparison. Wrong schema or provenance aborts the run; deterministic scenario failures are not silently retried.
- **D-11:** Prove complete adapter reset between reused requests and periodically cycle reusable processes. Sanitizer configuration must fail fast for undefined behavior so sanitizer findings cannot be mistaken for successful traces.

### Comparison, diagnosis, and regression evidence

- **D-12:** Use a typed, exhaustive comparison policy rather than generic JSON-path rules or a global epsilon. IDs, flags, counts, membership, record kinds, and other discrete observables compare exactly. Floating observables declare a versioned field-specific policy such as exact bits, absolute/relative tolerance, or ULP distance.
- **D-13:** Preserve order for checkpoints, solver-significant sequences, callbacks, and destruction events. Canonicalize only fields explicitly modeled as unordered sets or multisets, using stable semantic keys and deterministic tie-breakers. NaN, infinity, signed zero, and missing entities receive explicit policy and diagnostics rather than silent normalization.
- **D-14:** Validate provenance first, then compare checkpoints and named phases in order, stopping the primary diagnostic at the first divergent semantic path while retaining enough surrounding context for investigation. Reports distinguish protocol/harness failure from physics mismatch and remain machine-readable with a concise human rendering.
- **D-15:** Reproduction accepts a checked-in scenario name or the exact serialized scenario plus seed. Minimization must preserve scenario validity and the same failure signature; persist the minimized scenario value, not only a generator seed, because generator strategies may evolve.
- **D-16:** Reference generation writes to a temporary location, validates the result by replay, shows a reviewable diff, and only then atomically replaces an accepted fixture. Every reviewed trace or minimized regression records content/scenario hashes, protocol and comparator versions, oracle and adapter identities, compiler/target/flags, tolerance-profile hash, notices, and review status.

### Agent's Discretion

- Exact private crate and module names, provided protocol and differential tooling remain outside the published `liquidfun` dependency graph.
- Exact serialization spelling for typed semantic IDs and trace record payloads, provided it is deterministic, validated, and human-reviewable.
- Exact default byte/count limits and process request budgets, provided they are named, tested at the boundary, configurable only through reviewed harness configuration, and recorded in diagnostics.
- Exact diagnostic layout and minimization algorithm, provided the machine-readable failure taxonomy and same-signature requirement are preserved.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Scope and acceptance

- `.planning/ROADMAP.md` § Phase 2 — fixed goal, dependencies, requirements, success criteria, and research/ADR flags.
- `.planning/PROJECT.md` — native-Rust, Cargo-only consumer, reference isolation, determinism, semantic-testing, licensing, and transparency constraints.
- `.planning/REQUIREMENTS.md` — COMP-03 through COMP-09 and DOCS-05.
- `.planning/phases/01-oracle-provenance-and-repository-foundation/01-CONTEXT.md` — locked Phase 1 boundaries, oracle selection, private tooling, provenance, artifact-review, and package-isolation decisions.

### Reconciled research

- `.planning/research/SUMMARY.md` — Phase 2 research flags and recommended process-isolated differential architecture.
- `.planning/research/STACK.md` — JSON Lines boundary, exact float-bit fields, long-lived process rationale, tooling crates, testing tiers, and C++ JSON dependency recommendation.
- `.planning/research/ARCHITECTURE.md` — protocol/differential component responsibilities, semantic trace model, comparison policies, data flow, reference-data flow, and deterministic ordering ownership.
- `.planning/research/PITFALLS.md` — wrong-oracle, raw-memory, global-tolerance, ordering, provenance, and false-parity failure modes.

### Existing repository boundaries and evidence

- `ARCHITECTURE.md` — Cargo/C++ dependency direction, private `xtask` shell, read-only oracle, and deferred protocol boundary.
- `TESTING.md` — required Rust verification sequence, current CI tiers, deterministic retry policy, oracle commands, and future differential evidence expectations.
- `UPSTREAM.md` — canonical oracle identity, wrapper build, toolchain identity, notice obligations, and intentional update procedure.
- `reference/upstream-lock.toml` — machine-readable canonical oracle identity.
- `reference/artifacts/manifest.toml` — current artifact provenance and review model to extend for protocol evidence.
- `tools/reference/CMakeLists.txt` — repository-owned, out-of-submodule C++ adapter boundary.
- `tools/reference/CMakePresets.json` — canonical reference build presets and output locations.
- `tools/xtask/src/upstream.rs` — existing allowlisted process invocation and tool-identity patterns.
- `tools/xtask/src/provenance.rs` — fail-closed schema, path, hash, oracle-revision, and reviewed-artifact validation patterns.

### Repository standards

- `AGENTS.md` — Rust quality gates, GSD workflow contract, project constraints, and task-artifact rules.
- `AGENTS.bright-builds.md` — sync-first, deep-module, functional-core/imperative-shell, and verification defaults.
- `standards-overrides.md` — local exception registry; no substantive active override currently replaces the defaults.
- `standards/core/architecture.md` — boundary parsing, illegal-state modeling, and functional-core guidance.
- `standards/core/code-shape.md` — shallow control flow, module sizing, and rerunnable diagnostic tooling guidance.
- `standards/core/testing.md` — focused behavior tests with Arrange/Act/Assert structure.
- `standards/core/verification.md` — repository-native pre-commit verification requirements.
- `standards/languages/rust.md` — Rust modules, guards, optional naming, invariant types, and verification guidance.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `tools/xtask`: established private imperative shell with explicit command dispatch, typed errors, repository-root discovery, and integration-test fixtures.
- `tools/xtask/src/provenance.rs`: strict `serde` boundary parsing, unknown-field rejection, hash validation, path confinement, oracle agreement, and reviewed-artifact enforcement.
- `tools/xtask/src/upstream.rs`: allowlisted presets, actionable tool identity checks, clean-submodule verification, and structured external command invocation.
- `tools/reference`: existing CMake wrapper and presets already isolate generated outputs below `target/reference/` and keep changes outside the read-only submodule.
- `reference/`: existing lock, source map, compatibility ledger, discovery snapshot, and artifact manifest provide the provenance vocabulary for new traces and fixtures.

### Established Patterns

- `liquidfun` is the sole default and publishable crate; private developer tooling may use `serde`, `serde_json`, subprocesses, and C++ while the consumer crate may not.
- Machine-readable evidence is authoritative, generated human documentation is presentation, and checks validate evidence read-only instead of silently regenerating it.
- Boundary data uses strict typed parsing, stable sorting, explicit schemas, fail-closed validation, and focused command-level tests.
- C++ remains a read-only development oracle reached through repository-owned adapters; runtime linkage or delegation is prohibited.

### Integration Points

- Add private protocol and differential workspace packages without changing `default-members` or adding path dependencies/features to `liquidfun`.
- Extend `tools/reference` with a small C++ oracle executable and adapter sources while leaving `third_party/liquidfun` untouched.
- Extend `xtask`, `justfile`, and oracle CI with transparent scenario, compare, replay, and evidence-check entrypoints.
- Extend `reference/artifacts/manifest.toml`, `TESTING.md`, and `ARCHITECTURE.md` with protocol, failure-taxonomy, and fixture-review contracts.

</code_context>

<specifics>
## Specific Ideas

- Treat the empty-world round trip as the smallest vertical proof of the permanent protocol seam, not as a disposable ad hoc executable.
- Preserve exact `f32` bits on the wire even when a field later compares with tolerance; transport fidelity and comparison policy are separate decisions.
- Make one-shot isolation and bounded reuse two configurations of the same supervisor so focused debugging, pull-request corpora, and sanitizer runs do not need competing harnesses.
- Promote minimized serialized scenarios to named regressions only after replay proves they retain the same first-divergence signature.

</specifics>

<deferred>
## Deferred Ideas

- Broad rigid-body, joint, and particle observables and their final numeric tolerance values — later implementation phases define them from pinned-source audits and differential evidence.
- Public Rust world/object handles and callback/mutation semantics — Phase 3.
- In-process C ABI or FFI acceleration — only after profiling shows process IPC is a material bottleneck; the process backend remains the sanitizer and diagnosis reference.
- Property-based randomized scenario generation at scale — later testing phases; Phase 2 establishes a replayable/minimizable contract and may use only a bounded proof corpus.
- Concurrent multi-request RPC, cancellation, and distributed oracle services — outside the current sequential local/CI harness scope.

</deferred>

***

*Phase: 02-semantic-protocol-and-oracle-round-trip*
*Context gathered: 2026-07-10*
