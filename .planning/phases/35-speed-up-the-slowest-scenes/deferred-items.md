# Phase 35 Deferred Items

## From 35-01 (out of scope, pre-existing or environmental)

1. `cargo build -p liquidfun-wasm --target wasm32-unknown-unknown` (without `--lib`) fails before and after 35-01: the native-only bins `dam-break-bench`, `dam-break-timers` and `playground-scene-spot` import items gated `#[cfg(not(target_arch = "wasm32"))]` in `lib.rs`. Verified pre-existing by stashing the 35-01 changes. The web build compiles the library; `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown` passes. A fix would gate those bins (for example `required-features` or a `cfg` main stub) and is unrelated to Phase 35.
1. On this host, macOS syspolicyd was saturated (about 60% CPU, with another repository's builds running concurrently), so freshly linked executables sat at `_dyld_start` for minutes before running. Test targets that compile and launch new fixture executables at run time then exceed their own limits or stall: `xtask --test canonical_toolchain_workflow` (5 tests hit "installer test exceeded 15 seconds"), `xtask --test catalog_cli` (all 7 stalled for more than 5 minutes), and, late in the session, `xtask --test playground_cli` (the same target passed 4/4 spot tests in 1.46 s earlier on identical code). The merged liquidfun doctest binary also stalled for more than 45 minutes on the second run; it passed on the first run (Task 1), and `crates/liquidfun` is unchanged in 35-01. Re-run these targets on an idle host.

## From 35-04 (out of scope, pre-existing)

1. `cargo build --release -p liquidfun-wasm --bin playground-scene-spot` prints `warning: unreachable expression` at `crates/liquidfun/src/particle/solver/boundary/support.rs:98`. In `validate_candidate`, the `#[cfg(not(debug_assertions))]` block returns early, so the trailing `Ok(())` is unreachable in release builds only. The warning dates from `2dcc3822a` (24-04) and appears at plan start on unchanged code. Debug clippy with `-D warnings` does not see it. A fix would put the debug-only check and the final `Ok(())` under one `cfg` so each build has one return path.

## From 35-08 (out of scope, pre-existing)

1. `just web-smoke` fails 1 of 62 Playwright tests: `web/e2e/rust-wasm-proof.spec.ts:170` ("runs Rust WASM, visibly moves, and freezes after disposal") expects `getByRole('status')` to read `Loading Rust/WASM session…`. No file under `web/src/` contains that text, and the page now has four `role=status` outputs (fps, session status and two scene-control outputs), so the strict locator fails. The spec was last changed in `3a047bf99` and `web/src/` in `aa48f26d0`, both before Phase 35; Phase 35 changed nothing under `web/`. A fix would update the spec to the current loading text and scope the locator to `.session-status`.
