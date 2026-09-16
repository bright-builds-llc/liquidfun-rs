---
phase: 15-re-establish-candidate-evidence
plan: "07"
subsystem: evidence
tags: [canonical, promotion, independent-review, provenance, ci]
requires:
  - phase: 15-06
    provides: Bounded provider retention and explicit attestation readiness
provides:
  - Fresh canonical producer bundle and independently validated acquisition
  - Seven-path transactional promotion with identified independent AI review
  - Successful exact-head canonical Linux acceptance with retained identity
affects: [15-08, 15-09, 15-10, 15-11, 15-12]
tech-stack:
  added: []
  patterns: [explicit acquisition identity, bounded provider queries, independently acknowledged promotion]
requirements-completed: []
requirements-supported: [PLAT-01, DOCS-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-09-14T16-41-28
generated_at: "2026-09-16T17:57:47Z"
status: complete
completed: 2026-09-16
---

# Phase 15 Plan 07: Canonical Promotion Summary

**Fresh canonical evidence was independently acquired, independently AI-reviewed, promoted at Q and accepted by exact-head canonical Linux verification. Both plan tasks are complete; final candidate and milestone evidence remain outstanding.**

## Actual Evidence Identities

| Role | Observed identity |
| --- | --- |
| Actual producer P | `4e9f1edefba1f0a9a06abdc8fa2c44eaab725c1f` |
| Immutable bundle B | `6b126eafbe8ea26373b3994670ebce67519033868fb89df777b95c7a1defb502` |
| Canonical producer run / attempt | `34905391313` / `1` |
| Producer artifact | `10372097499`, `phase13-staged-34905391313-4e9f1edefba1f0a9a06abdc8fa2c44eaab725c1f` |
| Raw provider ZIP SHA-256 | `a385b74aa709443ef6384545b7ed0793fb203f69919da383b18e35806e957cdc` |
| Promotion base R | `8ff73150e6608eff4bcc9759618c45fa4b220eea` |
| Promotion commit Q | `7794304a5901a376c82d260c5b5dcc6835603673`, pushed to main |
| Independent reviewer | `codex-canonical-packet-pre-review`, separate AI agent `/root/canonical_packet_pre_review` |
| Acknowledged review digest | `07c79c668231d2e35566278b47d747432f079d4ec247b2e456d1916e2fd2dc2b` |
| Actual review time | `2026-09-16T17:55:13Z` |
| Exact-Q acceptance run | `35131323038`, attempt `1`, success |

Canonical recorded environment is Linux x86_64, Rust 1.97.0, Clang 22.1.8, CMake 4.3.3, Ninja 1.13.2 and `oracle-debug`, with upstream `7f20402173fd143a3988c921bc384459c6a858f2`.

Production and acquisition records are `target/phase15-canonical/production.json` and `acquisition.json`; actual raw provider records, ZIP and bundle are retained under `attempt-03/collection/`. The typed acquire-check returned exit 0 and recomputed the raw ZIP digest. Earlier wording in those immutable records describing human review as pending reflects their creation time; the later independent AI policy and acknowledgment are separate records, not rewrites of the historical acquisition.

## Implementation and Publication Commits

1. `00358a5` — Qualify fresh canonical promotion acquisitions: replace historical runtime producer/provider constants with explicit validated acquisition identity, preserve historical receipt validity and retain bounded archive/member validation.
1. `84405cc` — Resolve four pre-freeze review warnings and native CI gaps: bounded provider supervision, explicit GitHub host routing, isolated regression fixture attempt identity and attestation-backed inventory projection/native routes. Includes Windows process capture support and the Miri scanner correction reviewed in the recorded follow-up.
1. `3f954fe` — Record resolved pre-freeze review and CI recovery.
1. `71c0cb6` — Keep read-only Python invocation from generating checkout bytecode caches and make the Phase 9 workflow identity-marker assertion insensitive to formatting while preserving ordering checks.
1. `4e9f1ed` — Record full-workspace verification and final source review; selected as actual P for the retained canonical bundle.
1. `2e7cfd4` — Include the inventory adapter in isolated producer fixtures, retain useful fixture diagnostics and run the failure-prone fixture early in CI.
1. `8ff7315` — Implement the owner's explicit 2026-09-16 independent-AI reviewer policy, preserve identity/digest binding and remove the validator's Codex-name exclusion.
1. `7794304` — Atomically promote the independently reviewed canonical bundle with exact P/B/R trailers. Q's verified first parent is R.

The policy change is based on explicit owner authorization, not an invented human approval. An identified separate AI reviewer actually inspected and acknowledged the exact packet. The implementing agent did not self-approve. Historical human receipts retain their original identity and remain valid.

## Review and Promoted Semantics

The current packet and all review records are under `target/phase15-canonical/review-base-8ff7315/`. The independent reviewer passed 31 checks, inspected the complete semantic diff and all seven replacement files, independently regenerated the diff against R, and recomputed review, path-set, content-set, provider ZIP and bundle digests. All 123 witness and 766 replay closure entries were checked against R Git objects and pinned upstream objects. P is an ancestor of R; intervening changed paths are outside both recorded producer closures. B therefore retains its actual P identity rather than being relabeled as newly executed at R.

The seven-path transaction has four changed files and three byte-identical files:

- Replay evidence refreshes the oracle identity digest; D0 repeats, D1 result, sealed input and diagnostic content are unchanged.
- Manifest and promotion receipt refresh provenance, reviewer, P/B/R/provider and digest bindings.
- Witness provenance refreshes actual generation time, P and materials digest while preserving physical witness hash and compiler/upstream identity.
- Catalog Rust bytes, physical witness JSON and source-map bytes remain unchanged.

The recorded replay diagnosis of debug primitive count 0 versus 22 remains historical capture-schema context, not newly introduced physics drift. No expected physics bits were regenerated from the implementation under test.

`review-ack check`, `promotion-ready` and `promote` passed through the existing typed commands. Their logs are retained with the review packet. Q carries exact P/B/R commit trailers and uses the unchanged seven-file integrity boundary even though only four files differ in Git. The separate exact-head acceptance evidence is recorded below.

## Evidence from Executed Checks

- `target/phase15-plan07/main-prerequisite-gate/passed.json`: ordered fmt, Clippy, build and tests, additional private xtask Clippy, promotion/evidence/acceptance contract suites, Bright Builds, configured Markdown and diff checks all returned exit 0.
- `target/phase15-plan07/combined-repair-gate/passed.json`: ordered gates and 237 tests across nine focused targets, plus private xtask Clippy, Markdown, actionlint and managed checks passed. The independent reviewer matched all 43 repair hashes to the gated inventory and committed blobs.
- `target/phase15-plan07/read-only-and-workflow-repair/passed.json`: ordered gates, workspace all-target/all-feature Clippy, full-workspace tests and ancillary checks passed. The recorded full-workspace result is 2,274 passed, zero failed and one ignored fixture-regeneration utility.
- `target/phase15-plan07/fixture-dependency-gate/passed.json`: ordered gates, private xtask Clippy, safety contract target and ancillary checks passed for the fixture repair.
- `target/phase15-ci-fixture-confirmation/attempt-01/result/validation.json`: Cargo CI run `34907682631` at `2e7cfd457fae6d7944c6b6d2c849534415e3f232`, attempt 1, passed all four Linux/Windows/macOS jobs. Linux workspace evidence records 2,281 passed, zero failed and one ignored; the early fixture step passed. This is that exact source's CI evidence, not acceptance at Q.
- `target/phase15-review-policy/attempt-01/` and `result.json`: policy gates passed and the promotion contract passed all 49 tests, including actual reviewer-ID validation, independent AI IDs, invalid identity controls, historical human receipt and exact acknowledgment reviewer/digest safeguards. The independent policy review reported no findings.
- Current review directory `fmt.log`, `clippy.log`, `build.log`, `test.log` and `standards.log` retain promotion verification; Bright Builds reports 1,000 files and zero findings. The 31-check independent semantic review is in `validation.json` and `independent-review.md`.
- Post-Q preflight at `target/phase15-source-readiness/preflight-01/` passes local release-constructor, docs and corpus snapshot/closure checks, with 388 items and zero unresolved. This is local source-readiness evidence and is distinct from canonical acceptance.

Local verification used the dedicated macOS target `target/phase15-native-verification`, `CARGO_INCREMENTAL=0`, development/test debug information disabled and configured GNU Bash/find/coreutils. This avoids the unhealthy historical cache and reduces disk use; it is supplemental local evidence, not canonical Linux or Windows acceptance.

## Preserved Failures, Repairs and Evidence Limits

1. The first successful producer run `34899206044` at `00358a585f8629dbd7b4b4a486da6af0cfe94c7c` remains in `attempt-01/collection/`. The second run `34903089795` at `3f954fe8183b665d4d1fd15dae467cce5e9e4da3` remains in `attempt-02/collection/`. Neither is relabeled as the final P; attempt 03 is the selected bundle.
1. Independent pre-freeze review identified four original warnings; `15-REVIEW.md` records their resolution and cumulative review of 135 distinct paths, with zero unresolved findings at its reviewed source. It does not claim every path was re-reviewed from scratch at the final head.
1. Push CI at original P encountered the missing Miri terminal-record fixture failure and expected stale Phase 9 provenance failure. Later run `34903089766` failed the formatting-sensitive Phase 9 workflow identity assertion. Their retained logs and repairs are preserved; they are not passing acceptance records.
1. Actual native Windows process controls ran at source `3f954fe8183b665d4d1fd15dae467cce5e9e4da3`, run `34903184049`, attempt 1, job `104173806565`: nine controls passed. The overall Oracle run and job failed provenance validation. `target/phase15-native-portability/attempt-01/collection/validation.json` explicitly bounds this to the successful process-control step. The later `-B` flags had source review, not proof from that earlier Windows execution.
1. Earlier review packets, including `review-base-2e7cfd4`, are superseded for promotion; their old intended human reviewer is not the actual acknowledgment. Current independent AI review applies only to the digest in the identity table.
1. D0/D1 were reviewed as immutable provider-backed recorded outcomes. The independent reviewer did not rerun the expensive physics producer or rebuild a fresh oracle merely to reconstruct its runtime identity. Canonical acceptance remains independently required.

## Exact-Head Acceptance and Remaining Phase Work

Run `35131323038`, attempt 1, job `104912767133`, passed at exact Q `7794304a5901a376c82d260c5b5dcc6835603673`. The independently collected provider archive digest matched; its terminal identity binds the exact P/B/R/Q, reviewed content digests and pinned upstream. All seven ordered steps succeeded: identity, provenance, reviewed replay, diagnosis, regression, oracle build and live replay. Metadata, jobs, raw ZIP, logs, terminal identity and independent validations are retained under `target/phase15-canonical/acceptance-attempt-01/`.

Controlled-performance infrastructure remains unresolved; the last recorded prerequisite inventory showed zero repository runners and no listed repository identity secret, with organization access unconfirmed. No infrastructure identity or credentials have been invented. Final-C producers, all required retained release evidence, direct C-to-A attestation, later D documentation projection and milestone acceptance remain separate phase work. This plan's promotion is not the final release candidate freeze, a release authorization or completion of PLAT-01/DOCS-09.

## Self-Check: PASSED

This completed plan continues `gsd-yolo-discuss-plan-execute-commit-and-push 15` in yolo lifecycle `15-2026-09-14T16-41-28`. Implementation and promotion commits resolve; the retained independent review and canonical acceptance records have been inspected. Source readiness and final release attestation remain Plans 08–12 work. Requirement completion is intentionally unchanged until full candidate evidence passes.
