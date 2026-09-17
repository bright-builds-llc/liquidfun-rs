# Project scope

## Current direction

The owner clarified on 2026-09-16 that liquidfun-rs is a fun, experimental Rust physics project. Prioritize useful simulations, learning, approachable development, and fixing concrete bugs. Production certification and exhaustive upstream parity are optional ambitions, not ordinary project-completion criteria.

Linux x64 qualification is optional. Ordinary development and completion do not require a Linux x64 machine, canonical Linux oracle run, dedicated benchmark runner, or `PERFORMANCE_CONTROLLED_HOST_IDENTITY`. Local Rust checks and relevant regression tests remain the baseline. The owner selected local checks plus one hosted macOS Cargo CI smoke job. Cross-platform matrices, C++ comparisons, Miri/sanitizers, long fuzzing, coverage, and benchmarks are optional manual checks; expensive suites have no automatic schedules. Bright Builds managed standards and update automation remain separate from physics validation.

This owner-directed scope change supersedes older mandatory Linux requirements, including PLAT-01 and the Phase 15 controlled-host blocker. The old frozen-candidate constraint does not prevent work under the new scope. The results collected at `0f5bdd8bf99ff328810381de695a432419e6852d` remain historical evidence for that source, not evidence for later edits.

## Optional strict qualification

Existing canonical evidence, benchmark, release-audit, and attestation commands retain their original meaning when explicitly selected. Their Linux/toolchain/hardware requirements are prerequisites for those specific results, not for working on this hobby project. Missing evidence still means an uncertified result; it is never converted into a passing strict audit.

The current strict status remains **not release-ready**. That describes the parity-bearing release qualification, not whether the project is useful for experiments. Package publication remains a separate user-authorized action. Preserve licenses, existing evidence, regression tests, and accurate descriptions of compatibility and limitations.

## Further simplifications for discussion

The CI baseline and manual-only expensive checks above are accepted decisions. The following remaining recommendations are for discussion:

| Area                        | Proposed lighter approach                                                                                                     |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Cross-platform support      | Best-effort portability; test extra targets when someone needs them.                                                          |
| Benchmarks                  | Run useful local comparisons on demand; reserve controlled hardware for serious published performance claims.                 |
| Feature and behavior parity | Improve incrementally and document differences rather than block on exhaustive closure.                                       |
| Release paperwork           | A small experimental-release checklist; keep frozen-source attestations available for an explicitly requested strict release. |
| API and MSRV promises       | Keep the development toolchain pinned; decide durable compatibility promises when preparing a public release.                 |

A new publishing checklist and durable API/MSRV promises remain undecided. Existing optional tools and historical contracts remain available.
