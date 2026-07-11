# Codex Tasks

## task-ci-output-budget-and-provenance-history | 2026-07-10 10:59 CDT | Fix CI output budget and provenance history

- [x] Reproduce and trace the supervisor output-budget misclassification.
- [x] Replace per-byte stderr retention with bounded chunk-based first/last retention.
- [x] Ensure an observed output overflow wins over timeout without masking true timeouts.
- [x] Add focused regression tests.
- [x] Fetch full history in Oracle jobs that run provenance validation.
- [x] Run targeted Rust and provenance/workflow verification.
- [x] Review the diff and record residual risk.

Completion review: The supervisor now drains over-limit stderr with deadline headroom, preserves bounded first/last evidence and sanitizer detection, and classifies already-observed overflow ahead of timeout. Every Oracle checkout fetches the history required by provenance validation. Full workspace formatting, lint, build, tests, provenance validation, workflow lint, and diff checks pass. A remote Actions rerun remains pending because no commit or push was requested or performed.

## task-oracle-clang22-debug-build | 2026-07-10 11:41 CDT | Fix canonical Clang 22 oracle build

- [x] Confirm the Clang 22 failure and stdout-only diagnostic loss in current code.
- [x] Apply legacy warning compatibility options only to upstream Box2D for all Clang-family presets.
- [x] Retain stdout and stderr from failed CMake/Ninja processes.
- [x] Add focused regression coverage for stdout-only process failures.
- [x] Run targeted Rust and local oracle-debug verification.
- [x] Reproduce the oracle build in isolated Ubuntu 24.04 with Clang 22.1.8.
- [x] Run full required Rust checks, review the diff, and record residual risk.

Completion review: Canonical Ubuntu 24.04 with CMake 4.3.3, Ninja 1.13.2, and Clang 22.1.8 now builds `liquidfun-reference`; the legacy diagnostics remain visible as warnings only on upstream `Box2D`, while repository-authored C++ retains strict `-Werror`. Xtask now retains labeled stdout-only compiler diagnostics, covered by a command-level regression test. Targeted tests, clean local oracle configure/build, exact container verification, full workspace format/lint/build/tests, provenance, inventory, differential compare/replay, Markdown formatting, and diff checks pass. Remote Actions confirmation remains pending because this delegated debug task did not commit or push.

## task-04-05-pinned-probes-identity | 2026-07-11 01:03 CDT | Execute Phase 4 Plan 05

- [x] Implement bit-faithful external C++ math-probe dispatch and exact exceptional-bit transport.
- [x] Add complete strict Rust/C++ build identity and D1/D2/D3 evidence classification.
- [x] Enforce required and forbidden floating-point/compiler flags with D1 fail-closed behavior.
- [x] Run every focused C++/Rust/oracle check and the exact full Rust gate before each atomic task commit.
- [x] Run final debug/release probe comparisons, inspect effective flags/handshake, and verify Cargo-only isolation.
- [x] Create the lifecycle-bound 04-05 summary, update GSD state/roadmap/requirements, and commit metadata.

Completion review: The external C++ adapter now executes the complete 39-case Phase 4 math-probe corpus with exact `uint32_t`/`float` bit transport, bounded closed dispatch, unknown-operation rejection, and reset/reuse proof. Rust and C++ carry the same strict 17-field floating build identity, with D1 fail-closed required/forbidden flag enforcement and non-promotable D2/D3 evidence. Debug and release probes, C++ protocol tests, focused identity tests, effective compile-command and handshake inspection, Cargo package isolation, upstream verification, and the exact ordered full Rust gate all pass. The local Apple Clang 21 compiler lacks `-fdenormal-fp-math-fp32=ieee`, so it is correctly recorded as D2; canonical Clang 22 D1 requirements remain unchanged and fail closed.

## task-04-06-verification-entrypoints | 2026-07-11 01:48 CDT | Execute Phase 4 Plan 06

- [x] Add closed math-probe compare, replay, and fixed two-run D0 commands.
- [x] Compare typed native/C++ results under the reviewed Phase 4 field-policy registry.
- [x] Regenerate deterministic closed scenario and trace schema presentations.
- [x] Add transparent just recipes and supported/canonical CI coverage.
- [x] Run focused schema, fixture, CLI, debug/release/replay/D0, and workflow checks.
- [x] Run the exact ordered full Rust gate and review the task diff before committing.
- [x] Create the lifecycle-bound summary and update GSD progress artifacts.

Completion review: All 39 ordered probe cases compare successfully in debug and release, replay passes, and two independent debug processes produce byte-identical D0 output. The CLI rejects arbitrary paths, executables, compiler flags, profiles, presets, and run counts before effects; schemas remain byte-stable; canonical CI retains exact tools, SHA-pinned actions, read-only permissions, and a final evidence diff assertion. The local Apple Clang evidence remains non-promotable D2, while pinned Linux Clang 22 owns D1 evidence.

## task-04-07-numerical-policy-signoff | 2026-07-11 01:58 CDT | Execute Phase 4 Plan 07

- [x] Document the public math/settings contract and safe-Rust differences.
- [x] Document exact transport, all float policies, special-value rules, collection semantics, horizons, and D0-D3 authority.
- [x] Publish exact debug, release, replay, and two-run D0 commands with prerequisites and evidence limits.
- [x] Update only the three Phase 4 math/settings compatibility rows from executable evidence.
- [x] Extend documentation contracts for required claims, cross-file counts, and accidental local paths.
- [x] Run docs, inventory, provenance, rustdoc, package, oracle, overclaim, diff, and exact ordered Rust gates.
- [x] Create the lifecycle-bound summary and update GSD progress artifacts.

Completion review: The Phase 4 public math, numerical-policy, command, and evidence contracts are now precise and machine-audited. All 39 ordered math probes pass in debug and release, replay is successful, and two independent debug processes are byte-identical. Compatibility status remains conservative: three rows are implemented and unit-tested, only b2Math and the common subsystem carry scoped D2 differential evidence, and zero rows are platform validated. Canonical D1, settings differential parity, shapes, collision, solvers, particles, performance, and production maturity remain pending.
