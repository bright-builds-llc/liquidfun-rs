# Debug Session: CI Output Budget and Provenance History

## Status

- State: resolved
- Started: 2026-07-10 10:59 CDT
- Resolved: 2026-07-10 11:10 CDT
- Goal: find and fix
- Approved scope: supervisor output-budget classification and Oracle checkout history

## Symptoms

- Expected: over-limit supervisor fixtures return `TotalOutputExceeded`.
- Actual: two supervisor failure tests return `RequestTimeout` after producing more than the configured total output budget.
- Expected: Oracle provenance validation resolves a valid historical generator revision.
- Actual: GitHub Actions checks out one commit, so `git cat-file` cannot resolve historical revision `a9b3bd8...`.

## Reproduction

- `cargo test -p liquidfun-differential --all-features --test supervisor_failures -- --nocapture`
- GitHub Actions Cargo CI run `29096187230`
- GitHub Actions Oracle CI run `29096187182`

## Working Hypotheses

1. Per-byte rolling stderr retention consumes enough CPU while draining 65 MiB that the supervisor reaches its deadline before observing or classifying the output overflow.
1. The timeout path does not give an already-observed output budget overflow precedence at the deadline.
1. Oracle jobs use the checkout action's shallow default even though provenance validation intentionally resolves historical commits.

## Investigation Log

- 2026-07-10 10:59 CDT: User approved the focused fix plan; repository synced to `origin/main` before edits.
- 2026-07-10 11:03 CDT: Local supervisor suite reproduced both failures in 19.08s. `total_overflow` returned `RequestTimeout`; the concurrent overflow case returned a valid trace for one profile instead of rejecting the request.
- 2026-07-10 11:03 CDT: Confirmed stderr retention rotates a `VecDeque` once per byte after the first 128 KiB, while the drain publishes progress in 16 KiB chunks. This couples pipe-drain throughput to retained-evidence bookkeeping.
- 2026-07-10 11:03 CDT: Confirmed the main request receive path returns `RequestTimeout` at the deadline without consulting the atomic byte count. The later reconciliation path already checks the atomic total before returning timeout.
- 2026-07-10 11:03 CDT: Confirmed all four Oracle checkout steps run provenance validation and omit `fetch-depth`, leaving Actions at its one-commit default.
- 2026-07-10 11:08 CDT: Replaced per-byte deque rotation with slice-based first/last retention backed by a bounded tail ring; focused wrap and oversized-chunk tests pass.
- 2026-07-10 11:09 CDT: Added deadline classification that checks the atomic output count only after a startup/request timeout result. The original supervisor suite now passes all 9 tests in 16.44s, including the focused overflow-at-deadline case and existing true-timeout case.
- 2026-07-10 11:09 CDT: Added full-history checkout configuration to all four Oracle jobs plus a workflow contract regression test.
- 2026-07-10 11:10 CDT: Three repeated concurrent overflow runs passed but exposed only ~1.3s of deadline headroom. Replaced four full sanitizer-marker scans per chunk with one prefix-dispatched pass; the same focused run fell from ~8.7s to 2.62s, and cross-chunk sanitizer detection remains covered.
- 2026-07-10 11:10 CDT: Targeted package tests, clippy, all-target builds, workflow lint/contract checks, provenance validation, formatting, and diff checks all pass.

## Confirmed Root Cause

1. The stderr drain's per-byte deque rotation and four repeated sanitizer-marker scans per chunk delay pipe consumption enough for the 10-second request deadline and 50ms reconciliation quiet period to race ahead of output-budget observation.
1. The main deadline error path gives timeout unconditional precedence even when workers have already atomically counted output beyond the request budget.
1. Oracle CI asks provenance validation to resolve repository-history identities from shallow one-commit checkouts.

## Resolution

- Retain stderr evidence with slice copies into a bounded first/last buffer and tail ring.
- Scan each stderr byte at most once for sanitizer prefixes while preserving overlap detection across reads.
- On an actual startup/request timeout, prefer `TotalOutputExceeded` only when the atomic per-request count is already over budget.
- Fetch complete superproject history for every Oracle checkout and enforce that contract in tests.

## Verification

- `cargo fmt --all -- --check` — passed.
- `cargo clippy -p liquidfun-differential -p xtask --all-targets --all-features -- -D warnings` — passed.
- `cargo build -p liquidfun-differential -p xtask --all-targets --all-features` — passed.
- `cargo test -p liquidfun-differential --all-features` — passed, including all 9 supervisor failure tests.
- Focused concurrent overflow test — passed three consecutive pre-optimization runs and one post-optimization run; post-optimization runtime was 2.62s.
- `cargo test -p xtask --test docs_contract` — 11 passed.
- `cargo xtask provenance check` — passed for oracle revision `7f20402173fd143a3988c921bc384459c6a858f2` and one artifact.
- `actionlint .github/workflows/oracle.yml` — passed.
- `git diff --check` — passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
- `cargo build --workspace --all-targets --all-features` — passed.
- `cargo test --workspace --all-features` — passed, including unit, integration, contract, provenance, package-isolation, and doctest surfaces.

## Residual Risk

- GitHub Actions has not been rerun from these uncommitted local changes, so remote runner behavior remains to be confirmed after the orchestrator publishes or otherwise tests the patch.
