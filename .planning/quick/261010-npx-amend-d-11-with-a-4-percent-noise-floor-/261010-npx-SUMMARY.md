---
phase: quick-261010-npx
plan: "01"
subsystem: particle-engine-performance
tags: [perf, phase-35, d-11, liquid-tumbler, chain-edge-hoist]
requires:
  - "D-11 amendment F = 4% (35-CONTEXT.md line 42)"
  - "target/phase35/attempts/A3r.patch, bin/spot-base-ibo, bin/spot-before, before-full.jsonl"
provides:
  - "A3r2 kept: chain child edge hoist committed as 154ace4bf"
  - "35-PROFILES.md A3r2 record"
affects:
  - crates/liquidfun/src/particle/body_contact.rs
  - crates/liquidfun/src/world/particle_coupling.rs
tech-stack:
  added: []
  patterns: ["pre-registered protocol plus frozen judge before timing", "decision file written before any diagnostic or cargo run"]
key-files:
  created:
    - crates/liquidfun/src/particle/body_contact/chain_edge_tests.rs
  modified:
    - crates/liquidfun/src/particle/body_contact.rs
    - crates/liquidfun/src/particle/body_contact/tests.rs
    - crates/liquidfun/src/world/particle_coupling.rs
    - crates/liquidfun/src/world/particle_coupling/moving_fixture_query_tests.rs
    - crates/liquidfun/src/world/particle_coupling/scratch.rs
    - crates/liquidfun/src/world/particle_coupling/scratch_tests.rs
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
key-decisions:
  - "A3r2 (fresh rebuild of A3r) kept under amended D-11 (F = 4%): liquid-tumbler -9.150% / -11.025%, 25/25 fingerprints, no floor-confirmed regression; commit 154ace4bf"
metrics:
  duration: "about 2 h 37 min of wall time (22:09Z to 00:46Z), including about 50 min lost to the killed first cargo test run"
  completed: 2026-10-11
  tasks: 3
  files: 8
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 261010-npx
generated_at: "2026-10-11T00:46:35.477Z"
---

# Quick 261010-npx: Re-judge the chain-edge hoist under amended D-11 Summary

A fresh, pre-registered rebuild of A3r (A3r2) was kept under the amended D-11 with F = 4%. Liquid-tumbler gained −9.150% / −11.025% against `spot-base-ibo`, and all 25 fingerprints matched. Of the 12 listed scenes, none was above +4% in either isolated pair. The change is committed as `154ace4bf`. Against `spot-before`, liquid-tumbler now measures −15.8% / −14.8%.

## Decision

**DECISION: keep.** This comes from the frozen judge `target/phase35/npx_judge.py` (SHA-256 `7fdbd876…`), recorded in `target/phase35/A3r2.decision` at 22:28:38Z, before any diagnostic or cargo run. No diagnostic was run.

| Clause (amended D-11) | Result |
| --- | --- |
| Target gain, both targeted pairs | held: pair 1 22.565 (22.483-22.672) → 20.500 (−9.150%), pair 2 22.707 (22.581-22.746) → 20.204 (−11.025%) |
| Fingerprints 25/25 vs `before-full.jsonl` | held: 25/25 equal, and no mismatch in either targeted pair |
| No floor-confirmed regression (> 4% in BOTH isolated pairs) | held: none of 12 listed scenes |

Isolated pairs (ratio% against the base median, pair 1 / pair 2). The single-pair maximum is information only.

| Scene | p1 | p2 | single-pair max |
| --- | --- | --- | --- |
| color-mixer | −0.373 | −0.400 | −0.373 |
| float-or-sink | −0.303 | −0.383 | −0.303 |
| hydraulic-fountain | +0.289 | −2.067 | +0.289 |
| liquid-timer | −0.609 | −2.287 | −0.609 |
| particles | +0.302 | −0.321 | +0.302 |
| rigid-particles | −0.123 | −1.160 | −0.123 |
| soup | +0.948 | −1.826 | +0.948 |
| soup-stirrer | +0.504 | −0.024 | +0.504 |
| surface-tension | −1.632 | −3.500 | −1.632 |
| tesla-valve | −1.169 | +0.133 | +0.133 |
| washing-machine | +0.665 | +0.012 | +0.665 |
| wave-tank | −0.303 | −2.143 | −0.303 |

`d11_as_written` (above the base max in both pairs) was false for all 12, so the unamended rule would also have kept it.

Cumulative pair (`spot-before` vs `spot-A3r2`, evidence only): 24.448 (24.341-24.802) → 20.586, 24.546 (24.483-24.765) → 20.905, which is −15.8% / −14.8%.

## Identity and protocol

- HEAD `1cbfbb32b`. A clean-HEAD release build had SHA-256 `bd2b2eb5…`, the same as `spot-base-ibo`.
- `spot-A3r2` has SHA-256 `f028bf58…` and is byte-identical to `spot-A3r`. The source digest is `8c747d84…`, equal to A3r's, and `attempts/A3r2.patch` is byte-identical to `A3r.patch`.
- `npx-protocol.txt` was written at 22:24:23Z, before the first leg at 22:25:00Z.
- The judge selftest on the A3r data reproduced the published what-if exactly. The judge was not changed after hashing.
- Counted sets were all first runs. There were no voids, GATE-WAITED entries, GATE-TIMEOUTs or PRE-WAITs:
  - targeted 22:25:00–22:26:20Z (load 3.63–4.86)
  - full 22:26:34–22:27:20Z (3.90–4.86)
  - isolated 22:27:30–22:28:16Z (4.13–5.42)
- The cumulative pair ran 22:28:41–22:30:21Z, with one 15 s GATE-WAITED and no void.

## Tasks and commits

| Task | Result | Commit |
| --- | --- | --- |
| 1 Judge, fresh build, pre-registration | done (local `target/` files only) | none (gitignored) |
| 2 Gated protocol and decision | keep, cumulative pair done | none (gitignored) |
| 3 Checks, code commit, record | all six checks passed | `154ace4bf` perf(35), `28e1c4e69` docs(35) |

Checks (`target/phase35/A3r2-checks.results`), all exit 0:

- fmt
- workspace clippy `-D warnings`
- workspace build
- wasm32 `--lib` build
- bright-builds-check
- `cargo test --workspace --all-features`: 179 `test result` lines, 2,774 passed, 0 failed, 1 ignored

Post-commit, `git diff 154ace4bf~1 154ace4bf -- crates/ | shasum -a 256` equals `8c747d84…`, and the Co-Authored-By trailer is present on both commits.

## Interruption and restart

The first `cargo test --workspace --all-features` run started at 23:34Z. Test binaries were stalling about 2 minutes each at launch under the macOS security scan. The run was killed when the Claude session restarted, after 18 test binaries had passed and none had failed, and it recorded no exit code. After the restart, the macOS Developer Tools grant covered the process tree, which resolved the stall. Only `cargo test` was re-run (`A3r2-checks-resume.sh`, 00:26–00:46Z, appended to `A3r2-checks.log` and `.results`), with the tree digest re-checked at `8c747d84…`. The five checks that had already passed were not re-run, because the source was unchanged.

## Deviations from Plan

- **Phase summary Gain column:** besides the Attempts, Kept and Final median columns the plan named, the liquid-tumbler row's "Gain beyond noise" cell also got the after-A3r2 cumulative figures. Without them, the row would show a stale percentage next to the new final median.
- **Editor cargo check before timing:** after the patch was applied, the editor's rust-analyzer `cargo check` ran 22:14–22:24Z. The warm launch waited for it to exit, and it finished before the protocol was written and before any timed leg. This is recorded in the protocol and the run notes.
- **Resumed test run:** described above. The orchestrator directed it.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: target/phase35/npx-protocol.txt, npx_judge.py, A3r2.decision, A3r2-cum.keep.txt, A3r2-checks.results, attempts/A3r2.patch
- FOUND: commits 154ace4bf and 28e1c4e69
- crates/ digest of 154ace4bf equals A3r2.source-sha; no prior Attempts row changed (the Task 3 verify passed)
