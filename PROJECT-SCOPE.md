# Project scope

## Current direction

The owner clarified on 2026-09-16 that liquidfun-rs is a fun, experimental Rust physics project. Prioritize useful simulations, learning, approachable development, and fixing concrete bugs. Production certification and exhaustive upstream parity are optional ambitions, not ordinary project-completion criteria.

Linux x64 qualification is optional. Ordinary development and completion do not require a Linux x64 machine, canonical Linux oracle run, dedicated benchmark runner, or `PERFORMANCE_CONTROLLED_HOST_IDENTITY`. Local Rust checks and relevant regression tests remain the baseline. The owner selected local checks plus one hosted macOS Cargo CI smoke job. Cross-platform matrices, C++ comparisons, Miri/sanitizers, long fuzzing, coverage, and benchmarks are optional manual checks; expensive suites have no automatic schedules. Bright Builds managed standards and update automation remain separate from physics validation.

This owner-directed scope change supersedes older mandatory Linux requirements, including PLAT-01 and the Phase 15 controlled-host blocker. The old frozen-candidate constraint does not prevent work under the new scope. The results collected at `0f5bdd8bf99ff328810381de695a432419e6852d` remain historical evidence for that source, not evidence for later edits.

## Optional strict qualification

Existing canonical evidence, benchmark, release-audit, and attestation commands retain their original meaning when explicitly selected. Their Linux/toolchain/hardware requirements are prerequisites for those specific results, not for working on this hobby project. Missing evidence still means an uncertified result; it is never converted into a passing strict audit.

The current strict status remains **not release-ready**. That describes the parity-bearing release qualification, not whether the project is useful for experiments. Package publication remains a separate user-authorized action. Preserve licenses, existing evidence, regression tests, and accurate descriptions of compatibility and limitations.

## Accepted experimental baseline

Phase 15's revised discussion adopted these decisions for ordinary development and package preparation:

| Area                        | Accepted approach                                                                                                             |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Cross-platform support      | Non-macOS targets are best effort; test extra targets when needed. macOS CI records tested coverage, not a platform warranty. |
| Benchmarks                  | Run useful local comparisons on demand; reserve controlled hardware for serious published performance claims.                 |
| Feature and behavior parity | Improve incrementally and document differences rather than block on exhaustive closure.                                       |
| Release paperwork           | A small experimental-release checklist; keep frozen-source attestations available for an explicitly requested strict release. |
| API and MSRV promises       | APIs are experimental and may evolve; document incompatible changes before release. Defer durable API/MSRV guarantees.        |

Follow the experimental preparation checklist in [RELEASE.md](RELEASE.md#experimental-package-preparation) before preparing a package. Completion records the checked source, actual local and macOS CI results, limitations, licenses/notices, and independent reviewer identity, time and digest. It does not authorize a version, tag or publication.

Keep Rust 1.97.0 as the development pin and Cargo's declared `rust-version = "1.92"`. That declared minimum still means consumers can build with Rust 1.92: verify it before publication, or deliberately revise the manifest and documentation together with the consumer impact explained. Ordinary hobby work does not require a recurring MSRV matrix.

Preserve safe handles, checked mutation, native Cargo-only consumption and existing regressions as APIs evolve. Known compatibility differences remain visible; local checks do not prove exhaustive parity. Local benchmarks support on-demand diagnosis, and published comparisons identify workload, hardware, compiler and limitations. Existing optional tools and historical contracts remain available.
