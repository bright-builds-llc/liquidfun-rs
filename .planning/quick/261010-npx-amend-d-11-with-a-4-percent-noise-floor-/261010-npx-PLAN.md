---
phase: quick-261010-npx
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/liquidfun/src/particle/body_contact.rs
  - crates/liquidfun/src/particle/body_contact/chain_edge_tests.rs
  - crates/liquidfun/src/particle/body_contact/tests.rs
  - crates/liquidfun/src/world/particle_coupling.rs
  - crates/liquidfun/src/world/particle_coupling/moving_fixture_query_tests.rs
  - crates/liquidfun/src/world/particle_coupling/scratch.rs
  - crates/liquidfun/src/world/particle_coupling/scratch_tests.rs
  - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
autonomous: true
requirements: [PERF-08, PERF-09]
generated_by: gsd-plan-phase
lifecycle_mode: yolo
phase_lifecycle_id: 261010-npx
generated_at: "2026-10-10T22:06:36Z"
must_haves:
  truths:
    - "target/phase35/npx-protocol.txt was written before the first A3r2 timed leg. Its written_utc is earlier than the first start line in A3r2.uptime. It holds the amended D-11 rule verbatim, the SHA-256s of both binaries, the source digest, the gate and void rules, and the frozen SHA-256 of npx_judge.py."
    - "spot-A3r2 was built fresh from HEAD plus attempts/A3r.patch. Before the patch, git diff HEAD -- crates/ and git status --porcelain -- crates/ were both empty. The base is bin/spot-base-ibo (bd2b2eb5...). The comparison of the fresh binary with spot-A3r (f028bf58...) is recorded."
    - "The decision comes only from fresh A3r2 sets: targeted ABBA, then the full catalog, then isolated pairs, all run through abba-gated.sh. No old A3r timing file is a decision input. A3r2.decision was written before any diagnostic or cargo run."
    - "npx_judge.py applies the amended rule exactly. It keeps A3r2 only if liquid-tumbler's after median is below its base min in both targeted pairs, all 25 fingerprints match before-full.jsonl, and no listed scene has an after median more than 4% above its base median in BOTH isolated pairs. It also prints each isolated scene's single-pair maximum as information."
    - "If kept: A3r2-cum (spot-before vs spot-A3r2) was timed before any cargo run, all six checks passed, the committed crates/ diff digest equals A3r2.source-sha, and the code is committed as perf(35). If reverted: crates/ equals HEAD."
    - "35-PROFILES.md has a new A3r2 Attempts row and a run note. The liquid-tumbler target record and Phase summary row reflect the outcome. No existing Attempts row (A0 to A3br) changed."
  artifacts:
    - path: target/phase35/npx-protocol.txt
      provides: "Pre-registered amended-D-11 protocol (local, gitignored)"
    - path: target/phase35/npx_judge.py
      provides: "Frozen judge applying the F = 4% floor (local, gitignored)"
    - path: target/phase35/A3r2.decision
      provides: "Decision written before any diagnostic (local, gitignored)"
    - path: .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
      provides: "A3r2 Attempts row, run note, updated liquid-tumbler record and Phase summary row"
      contains: "| A3r2 |"
  key_links:
    - from: target/phase35/A3r2.decision
      to: target/phase35/npx_judge.py
      via: "python3 npx_judge.py A3r2, with the SHA-256 matching npx-protocol.txt"
      pattern: "DECISION: (keep|revert)"
    - from: "perf(35) commit"
      to: target/phase35/A3r2.source-sha
      via: "git diff HEAD~1 HEAD -- crates/ | shasum -a 256 equals the timed digest"
      pattern: "8c747d84"
---

<objective>
Re-judge the A3r chain-edge hoist under the amended D-11 rule, which adds the F = 4% noise floor. The run is fresh and pre-registered. If the judge keeps it, apply it, run the full checks, commit it, and record it. If the judge reverts it, record that honestly and leave the engine unchanged.

Purpose: A3r cut liquid-tumbler by 8.8% and 10.4% with bit-identical fingerprints. It was reverted on regressions of 1–7% in scenes without chains, which the A/A calibration showed are within no-op noise (M = 3.096%). The user adopted F = 4% on 2026-10-10. The amendment applies to new decisions only, so a fresh run is required.

Output: npx-protocol.txt, npx_judge.py, spot-A3r2 and the A3r2 timing sets with A3r2.decision (all local); either a perf(35) commit or a restored tree; and the 35-PROFILES.md record.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@./CLAUDE.md
@.planning/STATE.md
@.planning/phases/35-speed-up-the-slowest-scenes/35-CONTEXT.md

Read these only when needed. They are large; use grep or sed -n on the line ranges given:
- 35-PROFILES.md: line 11 (Phase summary, liquid-tumbler row), lines 30–31 (A3r and A3br bullets), line 286 (Attempts header), line 298 (A3r row), line 299 (A3br row, the last Attempts row), line 311 (ibo run notes), line 315 (liquid-tumbler target record), lines 507–520 (calibration what-if).
- target/phase35/mrp-protocol.txt (the Gate block, to copy verbatim) and target/phase35/A3r.decision (the decision-file format).

<facts>
Checked by the planner on 2026-10-10 at 22:06Z:
- HEAD is `1cbfbb32b`. `git diff cfe85b09f HEAD -- crates/` is empty, so the engine equals the A3r base `f7041fc75`. `git diff HEAD -- crates/` is empty. The untracked file `=` exists; do not touch it.
- `git apply --check target/phase35/attempts/A3r.patch` succeeds. The patch's own SHA-256 is `8c747d84ecf8822e881e9ef0ead0b8a2a158844d46d44ed15b553917e14fc2e6`, which equals the recorded A3r source digest. It changes 7 files and **creates one new file**, `crates/liquidfun/src/particle/body_contact/chain_edge_tests.rs`. `git diff HEAD` shows a new file only if it is intent-to-add, so run `git add -N` on that file after applying the patch.
- Binaries in target/phase35/bin:
  - `spot-base-ibo`: `bd2b2eb59057ecbcc6e31d10337c8d7552f14764737042d53f816adc2b1612f5`
  - `spot-A3r`: `f028bf5866e49bccb14070458697bd3cee5a9db60b1bbbf63940b7558a2381b9`
  - `spot-before`: `5bfc7b614d0701b6c9ba0606dd3b1b0937c5fc931676dff55c294cb6467e0742`
- Script hashes:
  - `abba-gated.sh`: `d7f7e055…`
  - `keep_rule.py`: `e2dafec6…`
  - `aa_calib.py`: `0393cf95…`
- Build: `cargo build --release -p liquidfun-wasm --bin playground-scene-spot` writes `target/release/playground-scene-spot`. In ibo, a release rebuild of the A3r tree was byte-identical to `spot-A3r`.
- jsonl record fields: `scene`, `median_ms_per_step`, `min_ms_per_step`, `max_ms_per_step`, `timed_out`, `fingerprint`, `runs`.
- abba-gated.sh calls `keep_rule.py` and prints `fingerprint mismatches:` and `medians above before max (any scene without a gain):` lines. In full mode (`LEGS="b1 a1"`, no scenes) the target list is empty, so every scene above its base max is listed.
- The calibration what-if for the old A3r data: jelly-drop +1.356% / +7.074% and water-wheel +2.248% / +2.255% against the base median; floor-confirmed none; would-be keep.
- The `cargo build -p liquidfun-wasm --target wasm32-unknown-unknown` build without `--lib` fails for a pre-existing reason. Use `--lib`.
- Trailer check: `git log -1 --format=%B | tail -1` reads a blank line. Use `git log -1 --format='%(trailers:only)'` instead.
</facts>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Write and freeze the judge, rebuild A3r fresh as spot-A3r2, and pre-register npx-protocol.txt</name>
  <files>target/phase35/npx_judge.py, target/phase35/bin/spot-A3r2, target/phase35/A3r2.source-sha, target/phase35/attempts/A3r2.patch, target/phase35/npx-protocol.txt, crates/ (patch applied, uncommitted)</files>
  <action>
All paths below are relative to the repository root unless the step says cwd target/phase35. Do not run any timed leg in this task.

1. **Judge script** `target/phase35/npx_judge.py`. Use only the standard library. Usage is `python3 npx_judge.py ID`, run from `target/phase35`. It reads only plain files in the cwd and never reads `*.void-*` directories. The constants are `TARGET = "liquid-tumbler"` and `F_PCT = 4.0`. Definitions follow aa_calib.py, where a is the after leg and b the base leg of the same pair:
   - ratio% = (a_median / b_median − 1) × 100
   - excess% = (a_median / b_max − 1) × 100
   - Compare unrounded values and print them to 3 decimals.

   It prints these sections in order:
   1. **Targeted.** Read `ID-{b1,a1,a2,b2}.jsonl`. Pair 1 is b1/a1 and pair 2 is b2/a2. For liquid-tumbler, print base median, min and max, the after median, ratio%, and `gain` (a_median < b_min). Also print any fingerprint mismatch between b and a in each pair.
   1. **Full.** Read `ID-full-{b1,a1}.jsonl`. Each must have 25 records with `timed_out` false. Listed = every scene with a_median > b_max, sorted by name; this is keep_rule's screening with an empty target list. Parse the `medians above before max` line in `ID-full.keep.txt`. If it differs from the computed listing, print `LISTING MISMATCH`, which is an input error.
   1. **Fingerprints.** Compare `before-full.jsonl` with `ID-full-a1.jsonl`. All 25 scenes must be present in both and equal; print `fingerprints: 25/25 equal` or the mismatching scenes.
   1. **Isolated.** If the listing is empty, print `isolated: none listed` and read no isolated file. Otherwise read `ID-iso-{b1,a1,a2,b2}.jsonl`. Their scene set must equal the listing. For each scene, print one row with:
      - ratio% and excess% for pair 1 and pair 2
      - `floor_confirmed`: ratio% > F_PCT in BOTH pairs
      - `d11_as_written`: excess% > 0 in both pairs (information only)
      - `single_pair_max`: the larger of the two ratio% values (information only)

      Then print the confirmed list.
   1. **Decision.** Print `DECISION: keep` only if all of these hold:
      - gain in both targeted pairs
      - no targeted fingerprint mismatch
      - fingerprints 25/25 equal
      - no floor-confirmed scene

      Otherwise print `DECISION: revert (<each failed clause>)`.

   On any input error (missing file, wrong count, `timed_out` true, scene-set mismatch, `LISTING MISMATCH`), print `INPUT ERROR: <reason>`, print no DECISION line, and exit 2. Exit 0 when a decision is printed.

   **Selftest.** Run `cd target/phase35 && python3 npx_judge.py A3r > npx-selftest.txt`. This is tooling validation on published data, not a decision input. It must show:
   - gain both (−8.832% / −10.436%)
   - fingerprints 25/25 equal
   - the 11-scene listing, with no `LISTING MISMATCH`
   - jelly-drop +1.356 / +7.074 and water-wheel +2.248 / +2.255
   - `d11_as_written` true for exactly jelly-drop and water-wheel
   - no floor-confirmed scene
   - `DECISION: keep`

   If anything differs, fix the script until it reproduces these, then freeze it: `shasum -a 256 npx_judge.py`. Do not edit the script after hashing.

2. **Base identity.**
   - Confirm `git diff HEAD -- crates/` is empty and `git status --porcelain -- crates/` is empty. If either is not empty, stop and report.
   - Build the release spot binary on clean HEAD. Its SHA-256 must start with `bd2b2eb59057ecbc`, the same as `bin/spot-base-ibo`. If it does not, stop and report: the base would not represent HEAD.

3. **Fresh A3r2 build.**
   - Run `git apply target/phase35/attempts/A3r.patch && git add -N crates/liquidfun/src/particle/body_contact/chain_edge_tests.rs`.
   - Write `git diff HEAD -- crates/ | shasum -a 256 > target/phase35/A3r2.source-sha` and `git diff HEAD -- crates/ > target/phase35/attempts/A3r2.patch`. The digest is expected to equal `8c747d84…`; record whether it does.
   - Build the release spot binary and `cp target/release/playground-scene-spot target/phase35/bin/spot-A3r2`. Do not link it.
   - Record its SHA-256 and inode, and whether it is byte-identical to `spot-A3r` (`cmp`). Either result is fine; the fresh binary is the one timed.
   - Leave the patch applied. Run no cargo command after this build until A3r2.decision exists.

4. **Warm launch.** Wait until `pgrep -x cargo || pgrep -x rustc` matches nothing. Then launch `bin/spot-A3r2`, `bin/spot-base-ibo` and `bin/spot-before` once each, untimed, with `--warmup 1 --steps 1 --runs 1 --scene particles`. Each must exit 0. Record the launch times.

5. **Pre-register** `target/phase35/npx-protocol.txt` before any timed leg. Contents:
   - `written_utc`, git HEAD, `uptime`, and whether cargo or rustc is running at writing.
   - The binaries (base `bin/spot-base-ibo`, after `bin/spot-A3r2`, cumulative base `bin/spot-before`), each with SHA-256 and inode, plus the spot-A3r comparison and the source digest.
   - The judge's SHA-256 and the selftest result.
   - The amended D-11 rule verbatim: CONTEXT.md line 41 plus the line 42 amendment, and the three binding clauses:
     1. Target gain: liquid-tumbler's after median below its base min in both targeted pairs.
     2. Fingerprints: all 25 match `before-full.jsonl`.
     3. No confirmed regression: for each scene the full catalog run lists above base max, run isolated ABBA pairs. A scene is a confirmed regression only if its after median exceeds its base median by MORE THAN 4% in BOTH isolated pairs.
   - The statement that the single-pair maximum is reported but is not a decision input.
   - The Gate block from `mrp-protocol.txt`, copied verbatim with its mrp line on moving `keep.txt`, and a pre-wait rule: before a third (NO_VOID=1) run, wait for 300 s with no cargo or rustc, capped at 3,000 s, and log `PRE-WAIT` in the set's `.uptime`.
   - The run order from Task 2 with its exact commands.
   - The statement that no old A3r timing file is a decision input.
   - The statement that A3r2.decision is written before any diagnostic or cargo run, and that diagnostics are named `A3r2-diag-*` and never change the decision.
   - The post-timing bug rule: if npx_judge.py has a parsing bug after timing, fix only that bug, append an addendum with the reason and the new SHA-256, and never change F or the definitions.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs/target/phase35 && grep -q "DECISION: keep" npx-selftest.txt && ! grep -q "MISMATCH" npx-selftest.txt && test -x bin/spot-A3r2 && test -s A3r2.source-sha && test -s attempts/A3r2.patch && grep -q "$(shasum -a 256 npx_judge.py | cut -c1-64)" npx-protocol.txt && grep -q "$(shasum -a 256 bin/spot-A3r2 | cut -c1-64)" npx-protocol.txt && grep -q "MORE THAN 4%" npx-protocol.txt && ! ls A3r2-b1.jsonl A3r2.uptime 2>/dev/null</automated>
  </verify>
  <done>The judge is frozen and its selftest reproduces the published A3r what-if. The base is confirmed equal to HEAD. spot-A3r2 is built from HEAD plus the patch, with its digest recorded. All three binaries have been warm-launched. npx-protocol.txt exists, and no A3r2 timing file exists yet.</done>
</task>

<task type="auto">
  <name>Task 2: Run the fresh gated protocol and write A3r2.decision</name>
  <files>target/phase35/A3r2*.jsonl, target/phase35/A3r2*.uptime, target/phase35/A3r2*.keep.txt, target/phase35/A3r2.decision</files>
  <action>
Use cwd `target/phase35` for every step. Run no cargo, rustc, clippy, just or bright-builds command during this task. The gate may wait up to 1,800 s per leg, so start each set with the Bash tool's `run_in_background: true` and wait for its `exit=` line. Never time two sets at once.

Run each set to completion before starting the next:
1. **Targeted.** `./abba-gated.sh A3r2 bin/spot-base-ibo bin/spot-A3r2 5 liquid-tumbler > A3r2.keep.txt 2>&1; echo exit=$? >> A3r2.keep.txt`
1. **Full catalog.** `LEGS="b1 a1" ./abba-gated.sh A3r2-full bin/spot-base-ibo bin/spot-A3r2 3 > A3r2-full.keep.txt 2>&1; echo exit=$? >> A3r2-full.keep.txt`. Both `A3r2-full-b1.jsonl` and `A3r2-full-a1.jsonl` must have 25 lines.
1. **Isolated.** Take every scene on the `medians above before max` line of `A3r2-full.keep.txt`, in printed order; liquid-tumbler is included if listed. Run `./abba-gated.sh A3r2-iso bin/spot-base-ibo bin/spot-A3r2 5 <scenes> > A3r2-iso.keep.txt 2>&1; echo exit=$? >> A3r2-iso.keep.txt`. If the line reads `none`, skip this set and record "none listed".

**Voids** follow the protocol:
- On `exit=3`, move that set's `.keep.txt` into its newest `.void-<epoch>/` directory. Do not read the voided numbers. Re-run the same command.
- At most 2 re-runs. Before the third run, do the logged 300 s PRE-WAIT, then run it with `NO_VOID=1`; it counts regardless.
- No valid set is re-run.
- Any other non-zero exit is a tooling error. Stop and diagnose without reading timings for a decision.

**Decision.** Immediately after the last set, write `A3r2.decision`, and do it before any diagnostic or cargo run. The file contains:
- a header with the UTC time it was written, the base and after binaries with their SHA-256 prefixes, the source digest, and the counted sets' UTC spans, 1-minute load ranges, and void, GATE-WAITED and GATE-TIMEOUT counts (from the `.uptime` files)
- the full output of `python3 npx_judge.py A3r2`
- for each clause, the amended D-11 clause it applies

Before writing, check that `shasum -a 256 npx_judge.py` still equals the protocol value. If the judge exits 2, apply the protocol's bug rule (fix only the parse bug, add an addendum, re-hash), then re-run it. Never re-time to fix a judge problem.

If the decision is **revert**, restore the tree now:
```
git apply -R target/phase35/attempts/A3r2.patch && git reset -q -- crates/
```
`git status --porcelain -- crates/` and `git diff HEAD -- crates/` must then both be empty.

If the decision is **keep**, run the cumulative pair now, before any cargo command. Warm-launch `spot-before` again if more than an hour has passed. Run `./abba-gated.sh A3r2-cum bin/spot-before bin/spot-A3r2 5 liquid-tumbler > A3r2-cum.keep.txt 2>&1; echo exit=$? >> A3r2-cum.keep.txt`. The same void rules apply. This pair is evidence only and not a decision input.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs/target/phase35 && test -s A3r2.decision && grep -Eq "DECISION: (keep|revert)" A3r2.decision && grep -q "exit=0" A3r2.keep.txt && grep -q "exit=0" A3r2-full.keep.txt && test "$(wc -l < A3r2-full-a1.jsonl | tr -d ' ')" = 25 && test "$(wc -l < A3r2-full-b1.jsonl | tr -d ' ')" = 25 && (grep -q "DECISION: revert" A3r2.decision && test -z "$(git -C ../.. status --porcelain -- crates/)" || grep -q "exit=0" A3r2-cum.keep.txt)</automated>
  </verify>
  <done>The fresh targeted, full and (if needed) isolated sets are counted under the gate. A3r2.decision holds the frozen judge's output and was written before any diagnostic or cargo run. If the decision is revert, crates/ equals HEAD. If keep, the cumulative spot-before vs spot-A3r2 pair exists.</done>
</task>

<task type="auto">
  <name>Task 3: Check and commit if kept, then record A3r2 in 35-PROFILES.md</name>
  <files>crates/ (7 files from A3r2.patch, only if kept), .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md</files>
  <action>
**If kept**, follow these steps in order. All cargo runs happen after every timed leg.

1. **Run the checks** on the applied tree, with exit codes logged to `target/phase35/A3r2-checks.log`. Run long commands in the background. The checks are:
   1. `cargo fmt --all --check`
   1. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   1. `cargo build --workspace --all-targets --all-features`
   1. `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown`
   1. `bun scripts/bright-builds-check.ts all`
   1. `cargo test --workspace --all-features`

   If any check fails, check whether the same failure occurs on HEAD; `target/phase35/checks-08.results` shows all of these at 0. A failure that needs a source change makes the tree differ from the timed digest. In that case, do not commit. Restore the tree as in Task 2 and record A3r2 as "kept by the judge, not committed: <check> failed". Do not re-time in this plan.
1. **Digest gate.** Before committing, `git diff HEAD -- crates/ | shasum -a 256` must equal `A3r2.source-sha`.
1. **Commit the code.** Stage the 7 patch paths explicitly; never use `git add -A`, and do not touch `=` or `.planning/config.json`. Commit as `perf(35): hoist chain child edges (A3r2, kept under amended D-11)`. The body gives liquid-tumbler's targeted ratios, 25/25 fingerprints, the floor-confirmed result, and the source digest, and ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
1. **Post-commit checks.** `git diff HEAD~1 HEAD -- crates/ | shasum -a 256` must equal `A3r2.source-sha`. If the text differs only in format, compare it with `attempts/A3r2.patch` and record the comparison. `git log -1 --format='%(trailers:only)'` must show the Co-Authored-By line.

**In every outcome**, edit 35-PROFILES.md (never mdformat `.planning/**`):
1. Append an `A3r2` row directly after the A3br row (currently line 299), in the same 11 columns:
   - Plan: `quick 261010-npx`
   - Change: "fresh rebuild of A3r (`attempts/A3r.patch`) judged under amended D-11 (F = 4%)", plus the binary SHA prefix and whether it is byte-identical to `spot-A3r`
   - Base: `spot-base-ibo` medians (min-max) for pairs 1 and 2
   - After: medians for pairs 1 and 2
   - Fingerprints: 25/25 with the file name
   - Regression: the full listing, each isolated scene's ratio% for pairs 1 and 2, the floor-confirmed list, and the informational single-pair maxima
   - Decision: kept or reverted
   - Commit: the hash, or none, with the patch file
   - Reason: which clauses held or failed
1. Add a bullet after the A3br bullet (line 31) with the outcome.
1. Add a "Quick 261010-npx run notes" paragraph after the ibo run notes (line 311). Include:
   - HEAD, binaries and hashes, and the source digest
   - the protocol and judge hashes and the time the protocol was written
   - the counted set spans and loads, with the `.uptime` file names
   - voids, waits and pre-waits
   - that `A3r2.decision` preceded any diagnostic or cargo run
   - if kept: the cumulative pair and the six check results
1. Append the A3r2 outcome to the liquid-tumbler target record (line 315). If kept, include the cumulative `spot-before` vs `spot-A3r2` medians, the change for pairs 1 and 2, and the commit.
1. In the Phase summary liquid-tumbler row (line 11), append `A3r2` to Attempts.
   - If kept, also append `A3r2` to Kept, and append `; after A3r2 (quick 261010-npx): X / Y` to Final median using the cumulative pair. Add one sentence to the Phase summary paragraph saying the engine now also includes A3r2 (commit hash).
1. Optionally append one sentence to Notes for Phase 36 item 1 giving the outcome.
1. Leave every existing Attempts row (A0 to A3br) and the calibration section byte-unchanged.

Commit only 35-PROFILES.md as `docs(35): record A3r2 re-judgement under amended D-11 (quick 261010-npx)`, ending with the same Co-Authored-By line.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs && P=.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md && grep -q "^| A3r2 | quick 261010-npx" $P && grep -q "Quick 261010-npx run notes" $P && git diff HEAD~1 HEAD -- $P | grep -E '^-[^-]' | grep -vE '^-(\| liquid-tumbler \||- liquid-tumbler:|Final commit|  1\. The chain child edge)' | wc -l | grep -qx ' *0' && (grep -q "DECISION: keep" target/phase35/A3r2.decision && git log --format=%s -2 | grep -q "^perf(35)" && test "$(git diff HEAD~2 HEAD~1 -- crates/ | shasum -a 256)" = "$(cat target/phase35/A3r2.source-sha)" || test -z "$(git status --porcelain -- crates/)") && git log -1 --format='%(trailers:only)' | grep -q "Co-Authored-By: Claude Opus 5.5"</automated>
  </verify>
  <done>Kept: the six checks passed, the perf(35) commit's crates/ digest equals the timed digest, and the cumulative figure is recorded. Not kept: crates/ equals HEAD and the reason is recorded. Either way, the A3r2 row, bullet, run note, target record and summary row are committed in docs(35), with no prior Attempts row changed.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Shared host → timing data | Other repositories' cargo jobs and load can bias the timings |
| Executor → keep decision | The executor sees results and could bias re-runs or the rule |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-npx-01 | Tampering (evidence integrity) | A3r2.decision, npx_judge.py | mitigate | Protocol and judge hash written before the first leg; hash re-checked before deciding; decision written before any diagnostic; voiding is result-blind (exit code only); old A3r timings never inputs |
| T-npx-02 | Repudiation | perf(35) commit | mitigate | Committed `git diff HEAD~1 HEAD -- crates/` digest must equal `A3r2.source-sha`; the patch is saved as `attempts/A3r2.patch` |
| T-npx-03 | Denial of service (measurement) | abba-gated.sh legs | mitigate | Per-leg cargo/rustc and load gate, void at load above 10, at most 2 re-runs with a logged PRE-WAIT, no own cargo during timing |
| T-npx-04 | Tampering (behavior) | engine change | mitigate | 25/25 fingerprints against `before-full.jsonl` plus the full `cargo test` before commit |
</threat_model>

<verification>
- npx-protocol.txt was written before the first start line in `A3r2.uptime`, and the judge hash in the protocol equals the file's hash at decision time.
- `A3r2.decision` exists with one DECISION line from the frozen judge. The decision's mtime is earlier than any `A3r2-diag-*` file and than `A3r2-checks.log`.
- The repository ends in exactly one state: a perf(35) commit whose digest matches the timed digest, or a crates/ tree equal to HEAD.
- 35-PROFILES.md has the A3r2 row, and no prior Attempts row changed.
</verification>

<success_criteria>
- A3r2 was judged on fresh data under the amended D-11 rule with F = 4%, applied exactly as written.
- The outcome, whether kept with a commit and cumulative figure or reverted with a reason, is truthfully recorded in 35-PROFILES.md.
- No timed leg ran while cargo or rustc ran. `=` and `.planning/config.json` were not touched.
</success_criteria>

<output>
After completion, create `.planning/quick/261010-npx-amend-d-11-with-a-4-percent-noise-floor-/261010-npx-SUMMARY.md`. Its frontmatter must include `generated_by: gsd-executor`, `lifecycle_mode: yolo`, `phase_lifecycle_id: 261010-npx`, and `generated_at` from `node "$HOME/.claude/get-shit-done/bin/gsd-tools.cjs" current-timestamp full`.
</output>
