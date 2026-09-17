# Phase 15: Experimental preparation results

## Checked source and scope

Implementation: `75ead0edcbde68d01b804e2c466f2f4b2a4d30da`, containing Plan 15-13's documentation changes since `92c536d`. The local checkout had only parent-owned STATE, ROADMAP and task-ledger changes when checking began; implementation files matched that commit. Subsequent completion, review and summary commits are records only.

The [experimental checklist](../../../RELEASE.md#experimental-package-preparation) is the acceptance boundary. This record is not strict certification: **not release-ready** under the optional strict profile. No version selection, tag or package publication was performed. Strict validators and historical campaign evidence remain unchanged.

## Local results

All commands below succeeded on macOS `aarch64-apple-darwin`, Rust 1.97.0 (`2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`), LLVM 22.1.6. Complete logs and source/compiler/status snapshots are retained under `target/phase15-hobby/attempt-15-14-01/`. The directory is ignored local evidence, not an uploaded or tracked artifact bundle.

| Order | Command | Result / log |
| --- | --- | --- |
| 1 | `cargo fmt --all` | Pass; `01-fmt.log` |
| 2 | `cargo clippy --all-targets --all-features -- -D warnings` | Pass; `02-clippy.log` |
| 3 | `cargo build --all-targets --all-features` | Pass; `03-build.log` |
| 4 | `cargo test --all-features` | Pass; `04-test.log` |
| 5 | `cargo test -p liquidfun --test particle_group_properties --test particle_groups --test particle_group_mutation` | 30 passed, including both persisted Windows seeds; `05-regressions.log` |
| 6 | `cargo test -p liquidfun-differential --test headless_catalog every_representative_family_executes_and_captures_without_a_display -- --exact` | 1 executed/passed, 4 filtered; `06-headless.log` |
| 7 | `cargo xtask package verify` | 238 entries built/tested outside the repository; `07-package.log` |
| 8 | `cargo test -p xtask --test docs_contract` | 43 passed; `08-contracts.log` |
| 9 | `cargo xtask docs check` | Public document contracts pass; `09-docs.log` |
| 10 | `just markdown-check` | Pass; `10-markdown.log` |
| 11 | `bun scripts/bright-builds-check.ts all` | Zero findings; `11-managed.log` |
| 12 | `cargo test` | Default-feature/prior-phase regression checks pass; `12-default-tests.log` |
| 13 | `git diff --check` | Pass; `13-diff-check.log` |

The native headless test executes eight representative rigid-body, joint, rope, particle, group, query, callback and mutation scenarios and asserts nonempty semantic checkpoints. This proves native execution/capture, not GUI behavior or exhaustive upstream parity. Replaying Windows seeds on macOS is not a new Windows test result.

Package verification validates the MIT manifest/license, matches packaged LICENSE to the root license, excludes private/native/renderer content and builds/tests the extracted consumer. LICENSE, THIRD_PARTY_NOTICES.md, reference/source-map.toml, dependency lockfile, compiler pin and package manifest are unchanged by this documentation implementation; existing attribution and notices are preserved. No new dependencies or derived source were introduced.

## Historical debug records and remaining limitations

- `safety-attempt-ci-ripgrep.md`: its outstanding follow-up is historical bookkeeping. Later source `2e7cfd457fae6d7944c6b6d2c849534415e3f232` passed the early fixture and all four Cargo jobs in run `34907682631`; retained `target/phase15-ci-fixture-confirmation/attempt-01/result/validation.json` records 2,281 workspace tests passed. This is historical evidence, not a current Linux result.
- `phase15-particle-forces-statistics.md`: its canonical follow-up records run `35140576574` at `91ac59b3068ec4c5929833b16413e478854d5fac`, passing canonical and sanitizer checks including the original failing scenario. That source repair is evidenced; the old strict campaign's bookkeeping does not block hobby completion. Pre-existing nonparticle debug-capture omissions were not repaired or certified by this phase.
- `macroquad-draw-rect-abort.md`: the diagnosed crash concerned dependency GUI tests; the record does not prove a liquidfun physics defect. Commit `0361335` subsequently retired Macroquad in favor of the current eframe/egui testbed. Current GUI behavior was not exercised here, and this phase does not claim GUI certification.

The old records are preserved. Non-macOS platforms remain best effort; APIs are experimental; feature/behavior parity is incremental. Linux, C++ campaigns, controlled performance, Miri/sanitizers, fuzzing and coverage were not rerun. Rust 1.92 remains the declared minimum and still needs verification before publication (or an explicit coordinated manifest/documentation revision). Passing Rust 1.97 is not proof of Rust 1.92 support. Durable API/MSRV promises remain deferred.

## Hosted result and independent review

[Cargo CI run 35169132504](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/35169132504) succeeded for exact `headSha` `75ead0edcbde68d01b804e2c466f2f4b2a4d30da` in `bright-builds-llc/liquidfun-rs`, workflow `.github/workflows/ci.yml`. `Default features (macos-15)` passed; `Linux quality and isolation` was skipped. Retained `ci-final.json` includes every job and step result; `ci-identity.json` records repository and workflow identity. No optional workflow was dispatched.

Separate AI reviewer `/root/review15_hobby_docs` inspected the complete implementation diff and local/CI evidence and independently acknowledged digest `841c63faa79d3e615cdbf949e0c2c5bd4584a4692bd1b1e6558575f18d1c4ea7` at `2026-09-17T01:10:03.370473+00:00`, with zero final findings. See [15-REVIEW.md](15-REVIEW.md) for the actual acknowledgment and all 28 bound path/content-hash pairs. This is AI review, not human approval or publication authority.

The digest is SHA-256 of sorted UTF-8 repo-relative path, NUL, lowercase file SHA-256, LF entries. `review-manifest.json` in the attempt directory binds nine implementation documents, their full diff, source/compiler/initial-status snapshots, command logs and final CI snapshots. This mutable completion record, GSD summaries, review acknowledgment and phase verification are excluded. Later record-only commits cite the tested implementation and do not claim their own HEAD was the CI source.

All six experimental checklist items are evidenced within these limits. Required local precommit checks are repeated for record commits; the historical strict campaign remains deferred rather than passed.
