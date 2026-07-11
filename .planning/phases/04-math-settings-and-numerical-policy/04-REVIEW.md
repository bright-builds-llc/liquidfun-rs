---
status: findings
depth: standard
files_reviewed: 45
findings:
  critical: 2
  warning: 4
  info: 0
  total: 6
---

# Phase 04 Code Review

## Scope and review basis

Reviewed the 45 requested Phase 04 source and evidence files against the phase context, research, seven plans and summaries, the diff from `e5de39725661e735f105b50a6a41bf99ea559d45`, and the pinned upstream math implementation. The review was materially informed by `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the local architecture, code-shape, verification, testing, and Rust standards. `crates/liquidfun-differential/build.rs` and adjacent protocol helpers were inspected as auxiliary evidence needed to follow the requested provenance and parsing paths.

Focused protocol, differential, xtask CLI, documentation-contract, and public-math tests pass. The findings below are contract and adversarial-path gaps not exercised by those passing tests.

## Critical findings

### CR-01: Native-tuned Rust builds can be classified as canonical D1 evidence

- **Evidence:** `crates/liquidfun-differential/build.rs:43-53` converts `CARGO_ENCODED_RUSTFLAGS` to a hexadecimal display string before searching it for `target-cpu=native`. The search can therefore never match a nonempty encoded flag string, and `LIQUIDFUN_NATIVE_TARGET_CPU` is reported as `baseline`.
- **Evidence:** `crates/liquidfun-differential/src/rust_adapter.rs:73-106` feeds those encoded flags into an opaque feature string while unconditionally reporting `fp_model = "precise"`, `fp_contract = "off"`, and `denormal_mode = "ieee"`; these properties are not derived from the effective Rust compiler invocation.
- **Evidence:** `crates/liquidfun-test-protocol/src/provenance.rs:534-572` grants D1 when the reported Rust version merely contains `1.97.0`, the target is canonical, the reported CPU is `baseline`, and the hardcoded floating fields look canonical. Its forbidden-flag scan does not decode the hexadecimal flags.
- **Evidence:** `tools/xtask/src/differential.rs:538-544` computes the native side with `NativeMathProbeExecutor` directly, so the Phase 4 comparison is not bound to the native `BuildIdentity` at all; only the C++ handshake identity is validated earlier.
- **Impact:** A Linux Rust build using `-C target-cpu=native`, explicit target features, or other noncanonical codegen can be labeled D1 and can contribute results to a command described as canonical. This breaks the fail-closed provenance boundary and can authorize evidence whose native half is not reproducible under the pinned scalar baseline.
- **Recommendation:** Preserve a parsed raw flag vector for validation and use encoding only for safe display. Derive CPU/features and floating semantics from the effective compiler invocation, require exact compiler-version matching, and bind both native and C++ identities into every math-probe comparison/evidence record. Add a subprocess regression that rebuilds with `RUSTFLAGS='-C target-cpu=native'` and proves the identity is D3/non-promotable.

### CR-02: The field-policy registry does not identify or enforce the actual observable and horizon

- **Evidence:** `crates/liquidfun-test-protocol/src/scenario/math_probe.rs:600-612` maps every operation not covered by four special branches to `math.constants.pi`. Consequently `abs`, `min`, `max`, `clamp`, inverse square root, dot/cross, matrix solves/inverses, cancellation, rounding, overflow, underflow, and the FMA witness all use a constant-PI policy. The checked-in request at `protocol/fixtures/accepted/math-probe-request.jsonl:1` contains these aliases, including `max-nan-order` and all arithmetic witnesses.
- **Evidence:** `protocol/tolerances/phase4-v1.toml:25-32` justifies that path only as the pinned PI token, gives it `exact_bits_transport`, and fixes its horizon to one operation. Using it for arithmetic NaN results permits exact NaN equality under a transport/pass-through exception, contrary to the phase rule that arithmetic NaN is a mismatch.
- **Evidence:** `protocol/tolerances/phase4-v1.toml:15-22` declares one 32-step horizon for `math.composite.transform`, while the checked-in corpus uses the same policy for one-step transform composition, 32-step composition, and a four-step sweep advance.
- **Evidence:** `tools/xtask/src/differential.rs:548-572` verifies only that Rust and C++ echoed the same request horizon, then looks up a policy by the aliased path. It never checks the request horizon against `FieldPolicy::horizon()` and never checks the build tier against `FieldPolicy::evidence_tier()`.
- **Impact:** The nominally closed registry is not a policy for each authoritative observable. A mismatch can be accepted under unrelated thresholds, special-value rules, justification, horizon, and authority. The current green 39-case result therefore does not establish the Phase 4 numerical-policy contract required by COLL-08.
- **Recommendation:** Define paths at least per operation and, where semantics differ, per result field. Bind each case to exactly one compatible policy horizon and compare the actual build tier with the policy tier before values are observed. Keep transport-only NaN policies on explicit pass-through probes; arithmetic kernels must use rejecting nonfinite rules. Add negative tests for horizon/tier mismatch and for aliased/unregistered operation fields.

## Warning findings

### WR-01: The new math-probe process path drops the existing bounded supervisor guarantees

- **Evidence:** `tools/xtask/src/differential.rs:425-480` pipes stderr but does not drain it until after all stdout records have been read, has no startup/request timeout, and returns early on decode/read failures without explicitly killing and reaping the child.
- **Evidence:** `tools/xtask/src/differential.rs:488-502` uses `BufRead::read_until` with no record-byte limit. `MathProbeResult` directly derives `Deserialize` over boxed strings and collections at `crates/liquidfun-test-protocol/src/trace.rs:57-61` and `crates/liquidfun-test-protocol/src/trace.rs:148-155`, so result decoding also bypasses the bounded raw-to-domain codec used elsewhere.
- **Impact:** A malformed, compromised, or simply buggy oracle can emit an unbounded line, fill stderr and deadlock, or never terminate, hanging local verification and CI or exhausting memory. This regresses guarantees already covered by the repository's normal supervisor tests.
- **Recommendation:** Route math probes through the existing concurrent bounded supervisor, or implement equivalent byte limits, strict raw/domain result validation, concurrent stderr retention, deadlines, and kill-wait cleanup on every error path. Add oversized-result, large-stderr, partial-line, and timeout integration tests for the exact xtask path.

### WR-02: `Sweep::transform_at` publicly bypasses the checked fraction invariant

- **Evidence:** `crates/liquidfun/src/math/sweep.rs:188-200` deliberately accepts any raw `f32`, including NaN, infinity, and values outside `0.0..=1.0`, and returns a public `Transform` rather than a typed error. In contrast, `advance` validates the same domain at `crates/liquidfun/src/math/sweep.rs:203-214`.
- **Impact:** A checked `Sweep` can produce a nonfinite transform or arbitrary extrapolation through its ordinary consumer API. This conflicts with the locked Phase 4 decision that ordinary physics boundaries reject nonfinite input and raw compatibility kernels remain private, and with the architecture documentation's checked normalized-fraction description.
- **Recommendation:** Make the public operation validate the fraction and return `Result<Transform, SweepError>`. Keep any raw kernel private and use it only after validation; the current checked-in probes use bounded endpoint fractions and do not require a public unchecked escape hatch. Add NaN, infinity, below-zero, and above-one tests that prove failure without state mutation.

### WR-03: Mismatch reports always claim an operation-horizon D1 comparison

- **Evidence:** `crates/liquidfun-differential/src/report.rs:369-405` constructs every report with `DivergenceHorizon::Operation` and `EvidenceTier::D1Canonical` regardless of the compared trace identities, field policy, scenario horizon, or host platform. The comparison test at `crates/liquidfun-differential/tests/comparison.rs:408-410` locks in those constants instead of deriving them.
- **Impact:** A mismatch produced on a supported D2 or exploratory D3 build is serialized as D1 canonical evidence, and a multi-step/phase-local mismatch is serialized as a one-operation result. This makes failure bundles and human diagnostics misleading precisely where evidence authority matters.
- **Recommendation:** Carry the validated field policy and build-derived evidence tier into report construction. If the legacy Phase 2 path cannot establish either value, represent that state explicitly or conservatively as exploratory rather than defaulting to D1. Test D1, D2, D3, operation, phase-local, and repeated-step report cases.

### WR-04: The recorded C++ “compile command” digest is a synthetic descriptor, not the effective command

- **Evidence:** `tools/reference/CMakeLists.txt:346-354` hashes a hand-built string containing compiler path/id/version, target, build type, a summarized flags string, and two source names. It omits the actual per-translation-unit command ordering, defines, include paths, implicit/default target features, linker-driver details, and other generated build inputs. Elsewhere the same file records `target_features` as `<none>` and libc/libm only as generic names rather than versions.
- **Impact:** Materially different math/probe compilations can share the same `compile_command_sha256`, so the identity cannot reproduce or distinguish the build whose numerical results are being cited. This falls short of the phase's explicit effective-command and OS/libc/libm identity requirement.
- **Recommendation:** Export and normalize the actual compile database entries for every probe/math translation unit, hash those exact effective commands after generation, and record versioned runtime-library identities where available. Fail canonical configuration if the effective entries cannot be captured or contain unreviewed flags, and regression-test that changing a define, include, target feature, or per-source option changes the digest.

## Verification performed

- `cargo test -p liquidfun --all-features`
- `cargo test -p liquidfun-test-protocol --all-features`
- `cargo test -p liquidfun-differential --all-features`
- `cargo test -p xtask --test differential_cli --test docs_contract`
- `git diff --check e5de39725661e735f105b50a6a41bf99ea559d45..HEAD`

All commands passed; the six findings remain because the affected adversarial and cross-contract paths are not asserted by the current tests.
