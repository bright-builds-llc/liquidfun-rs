---
phase: 04
slug: math-settings-and-numerical-policy
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-11
generated_by: gsd-secure-phase
lifecycle_mode: yolo
phase_lifecycle_id: 4-2026-07-11T04-16-20
---

# Phase 04 — Security

> Per-phase security contract for the 24 threats declared across Plans 04-01 through 04-07.

## Trust Boundaries

| Boundary | Description | Data crossing |
| --- | --- | --- |
| Public math API | Consumer-provided IEEE-754 values enter native Rust math and checked sweep operations. | Float bit patterns, vectors, matrices, transforms, and fractions. |
| Policy and scenario decode | Checked-in or external policy/scenario records enter strict typed protocol models. | Semantic paths, comparison rules, horizons, operation IDs, and exact operands. |
| Rust/C++ oracle boundary | A supervised child emits identity and math-probe JSONL records to Rust. | Sanitized build identity, typed results, terminal reset proof, and bounded diagnostics. |
| Build environment | Cargo, CMake, compiler, target, flags, and runtime FP state determine evidence authority. | Normalized digests, curated summaries, target/compiler identity, and runtime witnesses. |
| Local/CI orchestration | Contributors and CI select reviewed verification operations. | Closed xtask scenario, preset, profile, and run-count values. |
| Evidence to documentation | Machine results are translated into public compatibility claims. | Policy identity, evidence tier, inventory status, and reproducible commands. |

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
| --- | --- | --- | --- | --- | --- |
| T01-1 | Tampering | `math::settings` | mitigate | Immutable `pub const` values with exact-bit tests. | closed |
| T01-2 | Elevation of privilege | `liquidfun::math` | mitigate | Crate-wide unsafe prohibition and no unchecked/raw math access. | closed |
| T01-3 | Denial of service | Scalar/vector operations | accept | Fixed, allocation-free arithmetic without unbounded loops. | closed |
| T02-1 | Tampering | `Sweep` state | mitigate | Private state, checked construction/advance/transform, and validate-before-mutate behavior. | closed |
| T02-2 | Elevation of privilege | Matrix/transform representation | mitigate | Private initialized storage without raw layout or unchecked indexing. | closed |
| T02-3 | Denial of service | Matrix/sweep kernels | accept | Bounded direct arithmetic without data-dependent unbounded iteration. | closed |
| T03-1 | Tampering | Policy profile/hash | mitigate | Exact lookup, wildcard/default rejection, duplicate rejection, and complete canonical hashing. | closed |
| T03-2 | Repudiation | Mismatch reports | mitigate | Closed typed numeric, discrete, and harness evidence with stable signatures, policy/build identity, exact horizon/tier, and bounded context. | closed |
| T03-3 | Information disclosure | Diagnostics | mitigate | Typed allowlisted evidence and bounded context; no environment or command serialization. | closed |
| T03-4 | Denial of service | Policy/collection comparison | mitigate | Bounded policy, record, collection, process-output, timeout, and diagnostic limits. | closed |
| T04-1 | Tampering | Probe request | mitigate | Strict closed decoding, operation/input/path/horizon validation, stable IDs, and duplicate rejection. | closed |
| T04-2 | Elevation of privilege | Native dispatch | mitigate | Exhaustive closed Rust dispatch with no executable, path, pointer, or function-name input. | closed |
| T04-3 | Denial of service | Corpus/horizon execution | mitigate | Maximum 256 cases and 32 steps enforced before execution. | closed |
| T05-1 | Spoofing | Build identity | mitigate | Exact D1 compiler/target/feature allowlists, independent Rust/C++ source and command digests, and opaque/volatile runtime witnesses. | closed |
| T05-2 | Tampering | Float transport | mitigate | Size-asserted `memcpy` transport and exceptional-bit round-trip tests. | closed |
| T05-3 | Elevation of privilege | C++ dispatch | mitigate | Closed C++ operation/policy decoders and exhaustive dispatch. | closed |
| T05-4 | Denial of service | Adapter records | mitigate | Strict decoders plus shared bounded supervisor deadlines, drains, kill, and reap. | closed |
| T05-5 | Information disclosure | Identity/diagnostics | mitigate | Raw commands stay local; handshakes carry normalized digests and sanitized reviewed summaries only. | closed |
| T06-1 | Elevation of privilege | xtask routing | mitigate | Named scenario, preset, profile, subcommand, and fixed run-count allowlists. | closed |
| T06-2 | Tampering | CI tools/actions | mitigate | Read-only permissions, full-SHA actions, checksummed tools, and exact version assertions. | closed |
| T06-3 | Repudiation | Evidence mutation | mitigate | Canonical runs finish with scoped read-only evidence diff checks. | closed |
| T07-1 | Spoofing | Compatibility claims | mitigate | Inventory-derived, contract-tested claims limited to proven Phase 4 rows. | closed |
| T07-2 | Repudiation | Policy/evidence description | mitigate | Policy identity, horizons, tiers, commands, and limits are documented and machine-checked. | closed |
| T07-3 | Information disclosure | Documentation | accept | Public documentation contains no secrets and rejects absolute user paths. | closed |

*Status: open · closed. Disposition: mitigate · accept · transfer.*

## Accepted Risks Log

| Risk ID | Threat ref | Rationale | Accepted by | Date |
| --- | --- | --- | --- | --- |
| AR-04-01 | T01-3 | Fixed scalar/vector arithmetic can produce ordinary IEEE exceptional values but cannot amplify input into unbounded resource use. | Phase 04 threat contract | 2026-07-11 |
| AR-04-02 | T02-3 | Matrix and sweep kernels are deliberately bounded arithmetic; numerical domain errors are handled by checked public boundaries. | Phase 04 threat contract | 2026-07-11 |
| AR-04-03 | T07-3 | Phase documentation is public by design; contract tests prevent local-path and unsupported identity disclosure. | Phase 04 threat contract | 2026-07-11 |

## Security Audit Trail

| Audit date | Threats total | Closed | Open | Run by |
| --- | --- | --- | --- | --- |
| 2026-07-11 | 24 | 21 | 3 | GSD security auditor, initial audit |
| 2026-07-11 | 24 | 23 | 1 | GSD security auditor, post-fix audit |
| 2026-07-11 | 24 | 24 | 0 | GSD security auditor, final audit |

## Verification Evidence

- Final standard code review: 87 files, zero critical/warning/info findings.
- Canonical-identity regressions cover fixed/native CPU, SIMD/FMA, explicit target features, nested LLVM unsafe-FP options, base/Phase 4 target agreement, independent source/command digests, and runtime witnesses.
- Actual xtask comparison regressions cover typed numeric and discrete mismatches plus every closed harness-failure reason.
- Supervisor regressions cover startup/request timeout, oversized and partial records, 1 MiB stderr pressure, bounded retention, forced kill, and child reaping.
- Debug and release CMake/CTest, 39-case comparisons, replay, two-run D0 determinism, docs, inventory, provenance, package isolation, warning-denied rustdoc, and the full Rust gate pass.

## Sign-Off

- [x] All threats have a disposition.
- [x] Accepted risks are documented.
- [x] `threats_open: 0` is confirmed.
- [x] `status: verified` is set in frontmatter.

**Approval:** verified 2026-07-11
