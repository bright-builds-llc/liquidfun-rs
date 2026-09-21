---
phase: 23-baseline-pair-and-named-audit
verified: 2026-09-21T05:25:16Z
status: passed
score: 12/12 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T05:25:16Z
lifecycle_validated: true
overrides_applied: 0
re_verification:
  previous_status: human_needed
  previous_score: 12/12
  gaps_closed:
    - "D-16 independent AI review of the complete Phase 23 diff and live evidence"
  gaps_remaining: []
  regressions: []
---

# Phase 23: Baseline pair and named audit Verification Report

**Phase Goal:** A developer can read a committed audit that names dominating extra work from a SHA-bound unprofiled Dam Break pair and samply profile, and can run a private heap dump only when that profile shows allocator or `Vec` time.

**Verified:** 2026-09-21T05:25:16Z
**Status:** passed
**Re-verification:** Yes — after D-16 independent AI review landed
**Lifecycle:** `yolo` / `23-2026-09-21T02-44-37` copied from CONTEXT.md and PLAN.md; SUMMARYs match; no `direct-fallback`
**Nyquist Dimension 8:** skipped (`workflow.nyquist_validation` is false)

## Goal Achievement

Implementation must-haves for PERF-AUDIT / PERF-HEAP still hold (regression check of the previous 12/12). CONTEXT D-16 is now satisfied by committed `23-REVIEW.md` at `bd4bbae` (`docs(23): independent AI review acknowledgment`). This verifier confirmed that artifact exists, is not advisory, and is bound to `review_digest` `c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d`. It did not perform a second independent review.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Developer can open committed `docs/native-performance-audit.md` that names dominating Rust functions (and C++ counterparts when extra work is a shape mismatch), classifies suspected causes (extra per-particle work, per-step allocation, checks that survive `--release`, algorithm/shape differences), records the unprofiled Dam Break Medium wall-time ratio at a recorded HEAD, and states what was not found so SIMD is not the first move | ✓ VERIFIED | Regression: file still 138 lines, **Unreviewed local sample** banner, ratio `327.53395024734476`, `particle_rows` named, **Not found** section present. Unchanged since digest HEAD `30ecd75`. |
| 2 | Developer can point at a dated `target/dam-break-perf/<utc-stamp>/` from that HEAD containing the unprofiled pair report and samply `rust.json.gz` used to write those names; profiled duration is discarded for the gate | ✓ VERIFIED | Regression: bundle `2026-09-21T04-34-32Z/` still present with `pair.json` `kind: unprofiled_pair`, identity `kind: audit_bundle` / `not_timing_authority: true`. |
| 3 | If samply shows allocator or `Vec` time, developer can run private dhat on the `dam-break-bench` binary only and find the dump under the same gitignored evidence root; if not, committed notes record that and skip the heap run | ✓ VERIFIED | Regression: heap stamp `2026-09-21T04-47-09Z/` still has `dhat-heap.json` (`dhatFileVersion: 2`, 594964 bytes) and `heap-identity.json` (`kind: dhat_heap`, `features: ["dhat-heap"]`). |
| 4 | `docs/playground-dam-break-timing.md` is refreshed as an unreviewed local sample, not a Phase 12 public claim; raw `.json.gz` / `.trace` files stay gitignored | ✓ VERIFIED | Regression: banner + ratio + historical `1e5cbcc…` pointer remain. `git ls-files` still has no `*.json.gz` / `*.trace` / `dhat-heap.json`. `reviewed_reports = []`. |
| 5 | `playground_cli` is a `foo.rs` dispatcher plus `playground_cli/` children, no `mod.rs`/`main.rs`, files ≤628 lines so bundle/heap cases fit | ✓ VERIFIED | Regression: dispatcher 14 lines; no `mod.rs`/`main.rs`; longest child `support.rs` 304 lines. |
| 6 | `just playground-dam-break-audit-bundle` is a one-line alias; command mints a new exclusive stamp, copies (never moves) pair/profile artifacts plus `*syms*`, writes `audit-bundle-identity.json`, and fails closed on missing sources, HEAD mismatch, non-`unprofiled_pair`, missing gzip, or unsafe stamp names | ✓ VERIFIED | Regression: justfile lines 155–156 remain `cargo xtask playground dam-break-audit-bundle` only. `bundle.rs` / `bundle/ops.rs` still present (343 / 431 lines). |
| 7 | Fake-stamp CLI tests cover copy success (sources unchanged) and fail-closed HEAD mismatch, missing gzip, path-traversal stamp, and samply-tainted `pair.json` | ✓ VERIFIED | Regression: `tools/xtask/tests/playground_cli/bundle.rs` still 297 lines. |
| 8 | `liquidfun-wasm` optional non-default `dhat-heap = ["dep:dhat"]` (`dhat` 0.3.3); `#[global_allocator]` only in `dam_break_bench.rs` behind `cfg(all(feature = "dhat-heap", not(target_arch = "wasm32")))`; `liquidfun` stays bitflags-only | ✓ VERIFIED | Regression: wasm `default = []`, `dhat-heap = ["dep:dhat"]`, `dhat` 0.3.3 optional. `liquidfun` production dep remains `bitflags`. |
| 9 | `just playground-dam-break-heap` is a one-line alias; USAGE lists all five playground commands; heuristic scans `rust.json.syms.json` first then other `*syms*` then gzip JSON; match runs dhat, no-needles / no-symbols fail closed with distinct messages | ✓ VERIFIED | Regression: justfile lines 158–159 remain one-line `cargo xtask playground dam-break-heap`. `heap.rs` 469 lines, `symbols.rs` 336 lines. |
| 10 | Fake-cargo heap CLI: needle sidecar succeeds with dump + `not_timing_authority` identity and source stamp without dump; no-needle sidecar nonzero with no-allocator wording; non-JSON gzip without sidecar nonzero with could-not-read-symbols | ✓ VERIFIED | Regression: `playground_cli/heap.rs` 274 lines; `fake_upstream_tool.rs` 282 lines. |
| 11 | Live SHA-bound pair, profile, and audit-bundle stamps exist under the durable primary-tree `target/dam-break-perf/` with MEASURED_HEAD recorded in `23-07-SUMMARY.md` | ✓ VERIFIED | Regression: stamps `2026-09-21T04-32-19Z`, `2026-09-21T04-32-56Z`, `2026-09-21T04-34-32Z` still on disk. Live `pair.json` ratio `327.53395024734476`, HEAD `6d98531ac799987c209d3fd1e572e482fcab5da6`. |
| 12 | Isolation and honesty: `liquidfun` has no dhat/samply/serde/flate2/cmake dep; just aliases stay flag-free; pair argv has no `--features dhat-heap`; `[profile.profiling]` unchanged; no `[profile.release]` debug override; empty `reviewed_reports`; no physics kernel diff this phase; implementer did not write a D-16 acknowledgment | ✓ VERIFIED | Regression: `[profile.profiling]` still `inherits = "release"`, `debug = true`, `strip = false`; no `[profile.release]`. Five just recipes remain one-line printers. `pair.rs` still `--release` without features. `git diff 121d741..HEAD -- crates/liquidfun/src` empty. D-16 acknowledgment is `23-REVIEW.md` from a separate `gsd-code-reviewer` (`implementing_or_fixing_executor: no`), not the implementing executor. |

**Score:** 12/12 truths verified. Previously outstanding D-16 human item is closed (see below); not counted as a 13th implementation truth.

### D-16 independent review (previously human_needed)

| Check | Status | Evidence |
| --- | --- | --- |
| Artifact exists and is committed | ✓ VERIFIED | `.planning/phases/23-baseline-pair-and-named-audit/23-REVIEW.md` landed in `bd4bbae4a5a15d8761d73a4c4e18b14a7b47ca5b` (`docs(23): independent AI review acknowledgment`). Only that file changed vs advisory `30ecd75`. |
| Not advisory | ✓ VERIFIED | Frontmatter has **no** `not_d16_phase_approval` key. Body: “This file **is** the CONTEXT D-16 digest-bound phase acknowledgment. It is not advisory.” Footer: `_D-16 independent phase acknowledgment: yes_`. Status `clean`, findings 0. |
| Digest-bound | ✓ VERIFIED | `review_digest` / `digest`: `c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d`. Independently recomputed SHA-256 of `git show HEAD:<path>` for the 26 listed files in listed order; matches. Those files are unchanged `30ecd75..HEAD`. Supporting `diff_digest` `28113adb401439bc7d02abb6def57ea6a5e861db258c560924ae1c234d49d7c9` is recorded, not the binding digest. |
| Reviewer identity / time / not implementer | ✓ VERIFIED | `reviewer_disclosure: AI reviewer, not a human`. `implementing_or_fixing_executor: no`. Reviewer Cursor Grok 4.6 (`gsd-code-reviewer`). `reviewed: 2026-09-21T05:21:07Z`. `diff_base: 121d741…`, `local_head: 30ecd75…` (implementation HEAD before the review commit). |
| This verifier is not a second reviewer | ✓ VERIFIED | Confirmation only: artifact exists, D-16 flags, digest binding. No re-review of the Phase 23 diff. |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `docs/native-performance-audit.md` | Named-function audit with Not found | ✓ VERIFIED | Still present; in review digest file list |
| `docs/playground-dam-break-timing.md` | Unreviewed SHA-bound pair sample | ✓ VERIFIED | Banner + locked recipe + 23-07 numbers |
| `BENCHMARKING.md` | Pointer to audit, no public speed claim | ✓ VERIFIED | Still links the audit |
| `justfile` | Thin audit-bundle and heap aliases | ✓ VERIFIED | One-line printers |
| `tools/xtask/src/playground.rs` | Dispatcher + USAGE | ✓ VERIFIED | Still present (71 lines) |
| `tools/xtask/src/playground/bundle.rs` + `bundle/ops.rs` | Copy-only same-HEAD bundle | ✓ VERIFIED | Still present |
| `tools/xtask/src/playground/stamp.rs` | `mint_exclusive_stamp` / `parse_stamp_name` | ✓ VERIFIED | Still present |
| `tools/xtask/src/playground/symbols.rs` | `classify_profile_symbols` | ✓ VERIFIED | Still present |
| `tools/xtask/src/playground/heap.rs` | Gated dhat spawn | ✓ VERIFIED | Still present |
| `crates/liquidfun-wasm/Cargo.toml` | Optional `dhat-heap` | ✓ VERIFIED | Not default; dhat 0.3.3 optional |
| `crates/liquidfun-wasm/src/bin/dam_break_bench.rs` | cfg-gated Alloc + Profiler | ✓ VERIFIED | Still present |
| `crates/liquidfun/Cargo.toml` | bitflags-only production deps | ✓ VERIFIED | No dhat/samply/serde |
| `reference/performance/manifest.toml` | Empty reviewed_reports | ✓ VERIFIED | `reviewed_reports = []` |
| `tools/xtask/tests/playground_cli.rs` + children | Split CLI tests | ✓ VERIFIED | Dispatcher + filled bundle/heap modules |
| `tools/xtask/tests/fixtures/fake_upstream_tool.rs` | Fake cargo dhat writer | ✓ VERIFIED | Still present |
| `target/dam-break-perf/` | Live gitignored stamps | ✓ VERIFIED | Four dated stamps still on disk |
| `23-REVIEW.md` | D-16 digest-bound acknowledgment | ✓ VERIFIED | Committed at `bd4bbae`; digest matches HEAD bytes |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `justfile` | `tools/xtask/src/playground.rs` | `dam-break-audit-bundle` / `dam-break-heap` | WIRED | One-line aliases unchanged |
| `docs/native-performance-audit.md` | durable `target/dam-break-perf/<bundle>/` | cited stamp | WIRED | Stamp `2026-09-21T04-34-32Z` still live |
| `docs/native-performance-audit.md` | `pair.json` ratio | unprofiled wall-clock only | WIRED | Ratio still matches live `pair.json` |
| `23-REVIEW.md` | listed implementation files | `review_digest` SHA-256 of `git show HEAD:<path>` | WIRED | Digest recomputed and matched; files unchanged after `30ecd75` |

Prior wiring (bundle→stamp, heap→symbols, heap→dhat spawn, fake cargo dump writer, package `FORBIDDEN_PREFIXES`) was verified in the initial pass; files still exist and were in the D-16 digest set.

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `docs/native-performance-audit.md` | `rust_over_cpp_ratio` / walls | Bundle `pair.json` (`kind: unprofiled_pair`) | Yes — exact `327.53395024734476` still in live JSON and docs | ✓ FLOWING |
| `docs/native-performance-audit.md` | Heap run | `heap-identity.json` + `dhat-heap.json` | Yes — real dhat v2 dump still on disk | ✓ FLOWING |
| `docs/playground-dam-break-timing.md` | Current sample table | Same `pair.json` | Yes — identical ratio | ✓ FLOWING |
| `23-REVIEW.md` | `review_digest` | Concatenated HEAD bytes of 26 listed files | Yes — recomputed hash matches frontmatter | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| D-16 frontmatter is not advisory | Parse `23-REVIEW.md` YAML | `not_d16_phase_approval` absent; `implementing_or_fixing_executor: no`; `status: clean` | ✓ PASS |
| Digest binding | SHA-256 of `git show HEAD:<listed files>` in listed order | `c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d` | ✓ PASS |
| Review commit identity | `git show bd4bbae` | Rewrites only `23-REVIEW.md` (+229/−35) | ✓ PASS |
| Unprofiled ratio still matches live `pair.json` | Read bundle `pair.json` vs audit/timing docs | Exact `327.53395024734476` | ✓ PASS |
| Private dhat dump still real | Inspect heap stamp | `dhatFileVersion: 2`, 594964 bytes | ✓ PASS |
| Profile blobs not committed | `git ls-files '*.json.gz' '*.trace' '*dhat-heap.json'` | Empty | ✓ PASS |
| No kernel diff this phase | `git diff 121d741..HEAD -- crates/liquidfun/src` | Empty | ✓ PASS |
| `playground_cli` / `package verify` / `cargo tree` | Full cargo runs | Skipped — compile/verify exceeds 10s fast-verify budget | ? SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| PERF-AUDIT | 23-01, 23-02, 23-03, 23-07, 23-08, 23-09 | Committed audit names dominating functions, classifies causes, records unprofiled ratio, states what was not found | ✓ SATISFIED | `docs/native-performance-audit.md` + SHA-bound bundle stamp; REQUIREMENTS.md maps Complete |
| PERF-HEAP | 23-01, 23-04, 23-05, 23-06, 23-08, 23-09 | Private dhat on `dam-break-bench` only after allocator/`Vec` time; skip recorded otherwise | ✓ SATISFIED | Optional `dhat-heap`, live dump after needle match; REQUIREMENTS.md maps Complete |

No orphaned Phase 23 IDs. Mapping table lists only PERF-AUDIT and PERF-HEAP for this phase. Phase 24 covers 3× hot-path waves, not D-16, so nothing is deferred.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `target/dam-break-perf/2026-09-21T04-34-32Z/audit-bundle-identity.json` | `worktree_dirty: true` | Capture tree was dirty (`.planning/config.json` + `?? .vscode/`) | ℹ️ Info | Recorded in 23-07 SUMMARY as authorized exception; physics HEAD still `6d98531…`. Not a goal failure. |
| `docs/native-performance-audit.md` | Named-functions table | `ParticleContactUpdate::generate` is not a sidecar `string_table` string | ℹ️ Info | Documented as inclusive parent of `particle_rows`, which is in the sidecar. Consistent with inlining. |

The previous INFO that `23-REVIEW.md` was advisory is closed: that file was rewritten as D-16 at `bd4bbae`.

### Human Verification Required

None. The only prior human item was D-16; it is now a committed digest-bound independent AI acknowledgment. This verifier is not a second reviewer.

### Gaps Summary

No implementation gaps and no remaining human items. Phase 23 goal artifacts exist, are substantive, wired, and backed by live SHA-bound evidence. Independent review (CONTEXT D-16) landed in `23-REVIEW.md` at `bd4bbae`, bound to `c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d`.

---

_Verified: 2026-09-21T05:25:16Z_
_Verifier: Claude (gsd-verifier)_
