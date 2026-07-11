---
status: all_fixed
findings_in_scope: 12
fixed: 12
skipped: 0
iteration: 3
---

# Phase 04 Code Review Fix Report

## Outcome

All findings from the three capped Phase 04 review passes are fixed: six findings from iteration 1, four from iteration 2, and two from iteration 3. The six atomic fix commits preserve Cargo-only consumer isolation and keep local Apple Clang evidence at non-promotable D2 authority. The review artifacts remain intentionally uncommitted.

## Iteration 3 findings

### WR-01: Every Phase 4 comparison failure carries typed evidence

**Commit:** `a0f88d4` (`fix(04): type all math comparison failures`)

- Added one closed `Phase4ComparisonEvidence` enum with numeric mismatch, discrete mismatch, and harness-failure variants. `DifferentialError` retains and serializes this enum for the actual xtask command path.
- Added bounded `Phase4HarnessFailureReport` evidence with deterministic signature, exact request/scenario hashes, policy identity, effective tier, both build hashes, closed reason, bounded expected/actual context, and at most one neighboring case on either side.
- Closed harness reasons cover result count, case ID echo, operation echo, policy-path echo, horizon echo, float-value count, discrete count, discrete-field echo, unregistered policy, policy horizon, and policy tier.
- Sequence, echo, and policy/configuration violations are categorized as `harness-failure`, not physics mismatches.
- Added `Phase4DiscreteMismatchReport` for genuine exact boolean differences, including exact expected/actual field and values, stable signature, request/scenario/case/operation identity, full policy identity/rules, exact horizon, effective tier, both build hashes, and bounded neighboring context.
- Numeric typed reports now include the selected collection policy and policy justification in addition to comparison, zero, and non-finite rules.
- Actual-path regressions exercise every closed harness reason plus numeric and discrete mismatch variants. Typed harness output is bounded below 4 KiB in the regression corpus.

### WR-02: Unavailable horizon always forces D3

**Commit:** `a0f88d4` (`fix(04): type all math comparison failures`)

- Legacy report authority now derives from both build tiers and horizon authority.
- `DivergenceHorizon::Unavailable` unconditionally produces `EvidenceTier::D3Exploratory`.
- Regressions prove both D1+D1 and D1+D2 identities remain D3 when the report horizon is unavailable.
- Reviewed Phase 4 policy horizons continue to preserve their exact operation, four-step, or 32-step authority.

## Iteration 2 findings

### CR-01: Closed canonical code-generation authority

**Commit:** `24644dc` (`fix(04): close math identity trust boundaries`)

- D1 requires exact base/Phase-4 compiler and target agreement, pinned compiler release and Linux triple, baseline CPU, compiler-specific target-feature allowlist, required FP semantics, and runtime witnesses.
- Explicit Rust CPU/features/LLVM arguments and C++ fixed CPU, SIMD/FMA, unsafe FP, or fast contraction options prevent D1.
- Rust witnesses use opaque inputs and C++ witnesses use volatile inputs.
- Regressions cover Haswell, AVX2, FMA, nested LLVM unsafe FP, and compiler/target mismatch.

### CR-02: Native identity bound to executed math inputs

**Commit:** `24644dc` (`fix(04): close math identity trust boundaries`)

- A reviewed native source manifest covers the adapter, executor, and every authoritative math implementation module.
- Build-time and independent xtask hashing bind the native identity to the same complete input set.
- Cargo rebuild dependencies track every manifest entry, and package verification proves publishable-crate isolation.

### WR-01: Typed numeric Phase 4 first-divergence evidence

**Commit:** `3cb247b` (`fix(04): emit typed math mismatch evidence`)

- The real xtask numeric mismatch path records stable signature, request/scenario/case/field identity, policy identity/rules, exact horizon/tier, both build hashes, exact IEEE bits/classes/signs/distances, and bounded context.
- Regressions force operation/D1, four-step/D2, and 32-step/D3 numeric mismatches.
- Legacy numeric evidence uses explicit unavailable horizon instead of checkpoint ordinal.

### WR-02: Private, relocation-stable C++ command evidence

**Commit:** `24644dc` (`fix(04): close math identity trust boundaries`)

- Raw compile commands remain local for validation.
- Repository/build paths are normalized before hashing and xtask independently reproduces the digest.
- Handshakes transport only the digest and curated compiler/FP/target/sanitizer summaries.
- Relocation and path-leak regressions protect the boundary.

## Iteration 1 findings

### CR-01 and WR-04: Effective Rust/C++ build identity

**Commit:** `e3acc7f` (`fix(04): bind effective math build identity`)

- Decoded Rust flags, exact compiler identity, actual C++ compile-database hashing, runtime library identity, and independent adapter/command validation established the initial effective identity boundary.

### CR-02 and WR-03: Observable policy, horizons, and evidence authority

**Commit:** `4a33700` (`fix(04): enforce observable numerical policies`)

- Exactly 25 closed observable paths, operation-specific horizons, arithmetic NaN/infinity rules, policy hash gates, and weaker-build tier derivation replaced generic or hardcoded authority.

### WR-01 and WR-02: Bounded process supervision and checked Sweep input

**Commit:** `07667a3` (`fix(04): bound math probe execution`)

- Math probes use the bounded shared supervisor and strict result/end decoding.
- Public `Sweep::transform_at` rejects non-finite and out-of-range fractions before the source-ordered kernel.

## Security closure

### T03-1: Policy tampering

Closed by the complete explicit path registry, rejection of wildcard/default-like paths, canonical policy hashing, exact request/profile hash agreement, and closed operation/path/horizon mapping.

### T03-2: Repudiation-resistant evidence

Fully closed in iteration 3. Numeric and discrete physics mismatches and all sequence/echo/policy violations now emit distinct typed evidence through the real xtask path. Every record has a stable signature, exact request/scenario identity, policy identity where applicable, both build hashes, exact authority, and bounded context. Unknown legacy horizon can never claim D1 or D2.

### T05-1: Compiler/build identity tampering

Closed by the canonical allowlist, decoded nested flags, exact compiler/target agreement, runtime FP witnesses, reviewed native input manifest, normalized effective C++ command digest, and independent xtask rehashes.

### T05-5: Sensitive build-path disclosure

Closed by keeping raw commands local, normalizing hash inputs, transporting sanitized summaries only, and scanning generated headers and live handshakes.

## Cumulative commit mapping

| Review | Commit | Closed scope |
| --- | --- | --- |
| Iteration 1 | `e3acc7f` | Effective Rust/C++ identity and actual command binding |
| Iteration 1 | `4a33700` | Observable policy, exact horizons/tier gates, report authority |
| Iteration 1 | `07667a3` | Bounded supervision, strict decode, checked Sweep input |
| Iteration 2 | `24644dc` | Closed D1 model, native manifest, private normalized C++ evidence |
| Iteration 2 | `3cb247b` | Typed live numeric mismatch evidence and unavailable legacy horizon |
| Iteration 3 | `a0f88d4` | Typed harness/discrete evidence and forced-D3 unavailable horizon |

## Verification evidence

The iteration-3 commit was created only after this exact ordered gate passed:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`

Post-commit verification passed:

- `cargo test -p liquidfun-test-protocol -p liquidfun-differential -p xtask --all-features`
- All 11 closed harness-reason branches plus numeric and discrete actual-path evidence regressions
- Unavailable-horizon D1+D1 and D1+D2 authority regressions
- `cargo xtask check`
- `cargo xtask docs check`
- `cargo xtask inventory check`
- `cargo xtask provenance check`
- `cargo xtask package verify`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps`
- Debug and release oracle configure/build
- Debug and release CTest protocol suites
- Debug and release math-probe comparison: 39 ordered cases each
- Debug math-probe replay: 39 ordered cases
- Debug determinism: two byte-identical runs
- Generated-header and live-handshake path/raw-command leak scan
- `git diff --check`

## Residuals

- The local machine uses CMake 3.27.9 and Apple Clang 21 rather than canonical CMake 4.3.3 and Clang 22.1.8. These successful runs remain D2 supported evidence and cannot promote D1 fixtures.
- No review finding or security re-audit item remains skipped or partially fixed.
