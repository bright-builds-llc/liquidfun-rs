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
