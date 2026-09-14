# Phase 14 prerequisite CI lint repair

## Scope and authority

This adjacent CI recovery runs under the Phase 14 execution workflow and the standing autonomous iteration authorization in `AGENTS.md`. It changes only the differential CLI evidence test structure; it does not repair Windows particle allocation or establish Phase 14 acceptance.

The repo-local guidance, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the code-shape, testing, verification, and Rust standards informed the extraction and verification. The active global and repository lessons were read in full (11,548 bytes combined).

## Failure evidence

- Run: https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34799215304
- Source candidate: `f4ef73930a62aa884b820ce91ffe35a4608b2b8b`
- Job: `103838288467`, Linux quality and isolation
- Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `evidence.rs:61`: `cli_reuse_and_sanitizer_bundles_bind_the_second_request_and_session_identity` had 123 lines against Clippy's 100-line limit.
- `evidence.rs:113`: the closure borrowed an already borrowed root in `run_cli_with_root(&root, behavior, &arguments)`.

The failed job log was inspected directly. The repair starts after cleanup commit `49e382f17c42b25b9483e9c6134994dd2bc1c201`.

## Checkable repair plan

- [x] Extract the coherent per-case evidence assertion block into `assert_second_request_bundle` in the same file.
- [x] Remove the needless borrow inside the scoped worker closure.
- [x] Preserve all four concurrent cases, expected outcomes, canonical request checks, manifest/report bindings, session identity assertions, and diagnostic context.
- [x] Run the focused evidence test and warning-denying Clippy.
- [x] Run formatting, core Clippy, all-target core build, and core tests in the required order.
- [x] Run managed checks and review the diff; limit the repair commit to the test and this record.

## Verification

Verification uses the existing Linux Docker route to avoid the diagnosed native macOS executable-start stall: image `sha256:5f3d56072c0c734ebb2381c59f0a20db1330b8884134c4e3a9d369de2d810ea8`, repository mounted at `/workspace`, `RUSTUP_TOOLCHAIN=1.97.0`, `CARGO_TARGET_DIR=/cargo-target`, and the existing Rustup, Cargo registry, and target volumes. No warning suppression, acceptance waiver, test removal, or physics change is part of this repair.

All checks passed on 2026-09-14 UTC:

1. `cargo test -p liquidfun-differential --test round_trip cli_reuse_and_sanitizer_bundles_bind_the_second_request_and_session_identity -- --exact`: 1 passed, 0 failed; all four concurrent cases remain inside this test.
1. `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed across the full workspace, including the previously failing round-trip test target.
1. Required ordered gate: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`: all exited successfully. These default-member checks exercise the core crate; core unit, integration, and doc tests passed.
1. `bun scripts/bright-builds-check.ts all`: 932 files scanned, 0 findings.
1. `git diff --check`: passed. Manual diff review confirms unchanged cases, concurrency, diagnostics, and assertions; the helper receives existing case context and keeps all bundle checks together. This was the simplification pass; no new abstraction or dependency is needed.

## Completion and remaining acceptance

The two source lint errors are resolved with no new warnings or test failures. No new runtime surface or placeholder was introduced. This is prerequisite lint recovery only: Windows allocation diagnosis, implementation, regression proof, and hosted Phase 14 acceptance remain owned by the phase executor. The root coordinator owns publication and planning state; unrelated planning artifacts are excluded from this repair commit.
