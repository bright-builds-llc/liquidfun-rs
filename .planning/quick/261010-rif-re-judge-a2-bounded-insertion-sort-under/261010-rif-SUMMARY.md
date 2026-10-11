---
phase: quick-261010-rif
plan: "01"
subsystem: particle-contact-scan
tags: [perf, phase-35, d-11, abba, contact-proxy-sort]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 261010-rif
generated_at: "2026-10-11T02:19:52.078Z"
requires:
  - "35-CONTEXT.md D-11 and its 2026-10-10 amendment (F = 4%)"
  - "target/phase35/attempts/A2.patch, abba-gated.sh, keep_rule.py, before-full.jsonl"
provides:
  - "A2r kept: bounded insertion sort for the retained contact-proxy order (commit edcafebc0)"
  - "35-PROFILES.md A2r record (commit f63709b1e)"
affects:
  - crates/liquidfun/src/particle/contact_scan.rs
  - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
tech-stack:
  added: []
  patterns: ["pre-registered protocol plus frozen judge before timing", "result-blind gated ABBA"]
key-files:
  created: []
  modified:
    - crates/liquidfun/src/particle/contact_scan.rs
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
decisions:
  - "A2r kept under amended D-11 (F = 4%): liquid-tumbler -5.414%/-6.306% and stacked-drip -11.283%/-5.569% gain in both targeted pairs, 25/25 fingerprints, nothing floor-confirmed; particles +7.417%/+2.609% (above 4% in pair 1 only) is not a confirmed regression"
metrics:
  duration: 87min
  completed: 2026-10-11
  tasks: 3
  files: 2
---

# Quick 261010-rif: Re-judge A2 (bounded insertion sort) under amended D-11 Summary

The frozen judge kept A2r, a fresh rebuild of A2's bounded insertion sort for the retained contact-proxy order on the A3r2 engine. It applied the amended D-11 with F = 4%. A2r is committed as `edcafebc0`. Liquid-tumbler gained −5.4% / −6.3% and stacked-drip −11.3% / −5.6%, with 25/25 fingerprints. Particles was slower in both targeted pairs, but by more than 4% only in pair 1.

## Decision

`DECISION: keep` from `target/phase35/rif_judge.py` (SHA-256 `ae10461d…`, frozen before timing and re-checked before the decision). `A2r.decision` was written at 00:57:46Z, before any diagnostic or cargo run. No diagnostic was run.

| Target | Base `spot-base-rif` median (min-max), pair 1 / pair 2 | After `spot-A2r` | ratio% p1 / p2 | Gain (both pairs) |
| --- | --- | --- | --- | --- |
| liquid-tumbler | 20.164 (20.136-21.017) / 20.279 (20.251-20.320) | 19.072 / 19.000 | −5.414 / −6.306 | yes |
| stacked-drip | 2.540 (2.410-2.560) / 2.393 (2.365-2.397) | 2.253 / 2.259 | −11.283 / −5.569 | yes |
| particles | 1.458 (1.448-1.465) / 1.480 (1.451-1.500) | 1.566 / 1.518 | +7.417 / +2.609 | no; floor_confirmed False (above 4% in pair 1 only), d11_as_written True, single-pair max +7.417 |

- Fingerprints: 25/25 `A2r-full-a1.jsonl` vs `before-full.jsonl`; no targeted-pair mismatch.
- Full catalog listing: impulse, liquid-bubbler, particles, washing-machine. The isolated set (listing minus targets) is impulse, liquid-bubbler and washing-machine.
- Isolated ratio% (pair 1 / pair 2), with single-pair maxima as information only:
  - impulse +2.093 / +1.835 (max +2.093; d11_as_written True)
  - liquid-bubbler +3.614 / +0.734 (max +3.614)
  - washing-machine +3.295 / −1.311 (max +3.295)
- Floor-confirmed: none.
- Cumulative `spot-before` vs `spot-A2r` (`A2r-cum-*`, evidence only):
  - liquid-tumbler: 23.944 → 19.748 / 24.183 → 19.078 (−17.5% / −21.1%)
  - stacked-drip: 2.588 → 2.255 / 2.626 → 2.243 (−12.9% / −14.6%)
  - particles: 1.551 → 1.506 / 1.598 → 1.524 (−2.9% / −4.6%)

  All three are below the before min in both pairs.

## Tasks

| Task | Name | Commit | Files |
| --- | --- | --- | --- |
| 1 | Freeze judge, build binaries, pre-register protocol | none (gitignored `target/phase35/` artifacts) | rif_judge.py, rif-protocol.txt, bin/spot-base-rif, bin/spot-A2r, A2r.source-sha, attempts/A2r.patch, rif-warm.txt |
| 2 | Gated protocol and A2r.decision | none (gitignored) | A2r*.jsonl, A2r*.uptime, A2r*.keep.txt, A2r.decision, A2r-cum-* |
| 3 | Checks, commit, record | `edcafebc0` (perf), `f63709b1e` (docs) | crates/liquidfun/src/particle/contact_scan.rs, 35-PROFILES.md |

## Evidence

- `spot-base-rif` is a fresh build of clean HEAD `7c0d9b9d2` (SHA-256 `f028bf58…`), byte-identical to `spot-A3r2`. `spot-base-ibo` was not used.
- `spot-A2r` has SHA-256 `a7d837b1…`. The source digest is `0fac8e71…`, and `attempts/A2r.patch` is byte-identical to `A2.patch`.
- Selftest on the old A2 data (`rif-selftest.txt`) reproduced every expected value and printed keep. The negative selftest (`rif-selftest-neg.txt`) printed `DECISION: revert ((c) floor-confirmed regression: particles)`. `nosuch` exited 2 with INPUT ERROR.
- Protocol written 00:54:36Z, before the first leg at 00:54:39Z. The counted sets were all first runs: targeted 00:54:39–00:56:07Z (load 3.91–5.92), full 00:56:12–00:56:56Z (4.26–5.61) and isolated 00:57:08–00:57:22Z (4.97–5.23). There were 0 voids, 0 GATE-WAITED, 0 GATE-TIMEOUT and 0 PRE-WAIT.
- The cumulative pair ran 01:01:10–01:02:46Z with one GATE-WAITED of 195 s and no void.
- All six checks exited 0 (`A2r-checks.results`):
  1. `cargo fmt --all --check` (01:03Z)
  1. clippy with `-D warnings` (01:03–01:25Z)
  1. `cargo build --workspace --all-targets --all-features` (01:25–01:55Z)
  1. the wasm32 `--lib` build (01:55–01:57Z)
  1. `bun scripts/bright-builds-check.ts all` (01:57Z)
  1. `cargo test --workspace --all-features` (01:57–02:18Z): 165 `test result:` lines, 2,775 passed, 0 failed and 1 ignored, including `insertion_budget_exhaustion_matches_full_sort`
- `git diff HEAD -- crates/` before the commit, and `git diff edcafebc0~1 edcafebc0 -- crates/` after it, both hash to `0fac8e71…`. The commit trailer is present.

## Deviations from Plan

- **A2r.decision footer reworded (formatting only).** My clause footer repeated the text `DECISION: keep`, so the file had two DECISION lines, while the plan's verification expects exactly one. At 00:57:51Z, five seconds after writing it and before any other run, I reworded the footer to "Outcome: keep, per the frozen judge line above." The judge output and the decision are unchanged. 35-PROFILES.md records this.
- **Line-number drift.** The plan's line 381 is the A7 item. The A2 idea is the item before it, beginning "  1. The bounded insertion sort", which the plan's verify regex names. I edited that item.
- **Base build reused cached artifacts.** `cargo build --release` for clean HEAD finished in 0.19 s because `target/release` was already built from the identical A3r2 tree. The output is byte-identical to `spot-A3r2`, as the plan expected.

## Known Stubs

None.

## Notes

- Particles was slower than the base in both targeted pairs (+7.4% / +2.6%). Under the amended D-11 this is not a confirmed regression, because it exceeded F = 4% in only one pair. D-11 as written would have rejected A2r on particles and impulse. Against `spot-before`, particles is still below the before min in both pairs.
- A pre-existing release-only `unreachable expression` warning at `boundary/support.rs:98` is already listed in `deferred-items.md`.

## Self-Check: PASSED

- FOUND: target/phase35/rif-protocol.txt, rif_judge.py, A2r.decision, bin/spot-base-rif, bin/spot-A2r, A2r.source-sha, attempts/A2r.patch
- FOUND: commit edcafebc0, commit f63709b1e
