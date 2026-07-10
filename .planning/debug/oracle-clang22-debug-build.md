# Debug Session: Oracle Clang 22 Debug Build

## Status

- State: resolved
- Started: 2026-07-10 11:41 CDT
- Resolved: 2026-07-10 11:59 CDT
- Goal: find and fix
- Approved scope: upstream Clang warning compatibility and xtask failed-process diagnostics

## Symptoms

- Expected: `cargo xtask upstream build --preset oracle-debug` builds the pinned read-only LiquidFun oracle under canonical Clang 22.1.8.
- Actual: canonical Linux compilation fails in legacy `b2ParticleSystem.cpp`, after configure and all identity/provenance checks pass.
- Expected: failed CMake/Ninja/compiler commands retain actionable diagnostics.
- Actual: xtask emits `<no stderr>` when Ninja places the compiler failure on stdout.

## Reproduction

- GitHub Actions Oracle CI run `29107592042`, job `86411840030`.
- Isolated Ubuntu 24.04 with official Clang 22.1.8 and a writable copy of `tools/reference` plus `third_party/liquidfun`.
- Failing diagnostics: `b2ParticleSystem.cpp:553` `memset` and `b2ParticleSystem.cpp:493` `memcpy` on `b2ParticleColor` are promoted by `-Werror,-Wnontrivial-memcall`.

## Working Hypotheses

1. The existing legacy Clang compatibility flags are incorrectly gated on sanitizer flags, leaving the canonical debug and release presets exposed to the same modern diagnostic.
1. `process_output` selects only stderr when formatting failures, so stdout-only CMake/Ninja diagnostics are discarded.

## Investigation Log

- 2026-07-10 11:41 CDT: User approved the focused fix plan; repository fetched and confirmed synchronized with `origin/main` before edits.
- 2026-07-10 11:41 CDT: Confirmed `tools/reference/CMakeLists.txt` attaches both compatibility options only when the compiler is Clang-family and `CMAKE_CXX_FLAGS` contains `sanitize`; the options target only upstream `Box2D`.
- 2026-07-10 11:41 CDT: Confirmed `tools/xtask/src/upstream.rs::process_output` converts both streams but reports only trimmed stderr, substituting `<no stderr>` when it is empty.
- 2026-07-10 11:44 CDT: Applied the two existing compatibility options to upstream `Box2D` for every Clang-family configure and changed failed-process formatting to retain labeled stdout and stderr.
- 2026-07-10 11:45 CDT: Added a fake stdout-only CMake/compiler failure and command-level assertion; all 12 upstream CLI tests pass.
- 2026-07-10 11:46 CDT: Local oracle-debug reconfiguration and build passed under AppleClang 21.0.0, keeping all three legacy diagnostics visible as warnings.
- 2026-07-10 11:55 CDT: A disposable Ubuntu 24.04.4 container verified exact CMake 4.3.3, Ninja 1.13.2, and Clang 22.1.8, then completed all 60 build steps for `liquidfun-reference`. The two `-Wnontrivial-memcall` diagnostics remained visible as warnings.
- 2026-07-10 11:55 CDT: Canonical container `compile_commands.json` showed both compatibility options only on `b2ParticleSystem.cpp`; repository-authored `protocol.cpp` retained `-Wall -Wextra -Wpedantic -Werror` with neither exception.
- 2026-07-10 11:59 CDT: Clean local configure/build, full workspace Rust checks, provenance/inventory validation, and one-shot/reuse/replay differential commands all passed.

## Confirmed Root Cause

1. The compatibility options required by the read-only 2014 upstream code are scoped to sanitizer presets instead of every Clang-family build.
1. Failed-process formatting ignores captured stdout even though CMake/Ninja may write compiler diagnostics there.

## Approved Resolution

- Keep the two warnings visible but non-fatal on upstream `Box2D` for all Clang-family builds; retain strict `-Werror` on repository-authored C++ targets and do not modify the pinned upstream tree.
- Include every non-empty captured process stream in xtask failure diagnostics and cover stdout-only failure behavior.

## Resolution

- Removed the sanitizer-flags condition from the existing Clang-family compatibility block while retaining target-local `PRIVATE` options on upstream `Box2D`.
- Render failed-process diagnostics from every non-empty captured stream with explicit `stdout:` and `stderr:` labels; use `<no stdout or stderr>` only when both are empty.
- Added command-level regression coverage using a fake CMake process that exits with status 42 after writing its compiler diagnostic only to stdout.
- Relied on the real canonical build plus compile-command inspection as the durable CMake scoping guard instead of adding a brittle source-text contract test.

## Verification

- `cargo test -p xtask --test upstream_cli -- --nocapture` — 12 passed, including stdout-only failure retention.
- Clean `cargo xtask upstream configure --preset oracle-debug` and `cargo xtask upstream build --preset oracle-debug` — passed locally with CMake 3.27.9, Ninja 1.13.2, and AppleClang 21.0.0.
- Disposable Ubuntu 24.04.4 `linux/amd64` container — verified CMake 4.3.3 archive SHA-256, Ninja 1.13.2 archive SHA-256, llvm.sh SHA-256, and Clang 22.1.8; clean configure and all 60 `liquidfun-reference` build steps passed.
- Canonical container compile-command assertions — upstream `b2ParticleSystem.cpp` includes `-Wno-error=unused-result` and `-Wno-error=nontrivial-memcall`; repository-authored `protocol.cpp` includes `-Werror` and neither compatibility option.
- Recomputed adapter content identity after the behavior-affecting CMake change: `a63b12b7fc1f59c413bbddbcba3841f4bca1de3b2e9bf7ed11f13d8216e93f34`; CMake independently accepted it.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
- `cargo build --workspace --all-targets --all-features` — passed.
- `cargo test --workspace --all-features` — passed, including 12 upstream CLI tests and all unit, integration, command, and doctest surfaces.
- `cargo xtask provenance check` — passed for one artifact record.
- `cargo xtask inventory check` — passed for 177 compatibility rows.
- One-shot compare, two-request reuse compare, and reviewed-trace replay — all matched.

## Residual Risk

- GitHub Actions has not rerun from these uncommitted local changes, so remote runner confirmation remains pending until the orchestrator commits and pushes.
