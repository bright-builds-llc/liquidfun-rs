---
phase: quick-261010-mrp
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
autonomous: true
requirements: [PERF-08, PERF-09]
generated_by: gsd-plan-phase
lifecycle_mode: yolo
phase_lifecycle_id: 261010-mrp
generated_at: "2026-10-10T21:25:35Z"
must_haves:
  truths:
    - "target/phase35/mrp-protocol.txt (A/A design, metrics a-g, the F formula, binary SHA-256s and the SHA-256 of aa_calib.py) was written before the first A/A timed leg; its written_utc is earlier than the first start line in AA1.uptime."
    - "B is bin/spot-base-ibo and A is bin/spot-AA, a separate copy with an identical SHA-256 (bd2b2eb5...). The copy was launched once, untimed, before any timed leg."
    - "Three A/A rounds (R1, R2, R3) ran the 35-PROFILES §Method sequence unchanged through abba-gated.sh: targeted ABBA on liquid-tumbler (--runs 5), full catalog b1 then a1 (--runs 3), and isolated ABBA (--runs 5) on every scene the full run listed. The same gate and void rules applied, with at most 2 re-runs per set."
    - "No timed leg started or ended while cargo or rustc ran. Every leg has uptime lines in target/phase35/AA*.uptime. No cargo or rustc command was run by the executor during this plan."
    - "35-PROFILES.md ends with a new '## D-11 calibration (quick 261010-mrp)' section with the protocol summary, per-round tables, the false-alarm count, the noise distribution, the F formula and value, and the what-if table for A3, A3b, A3r and A3br. The section is labelled a proposal for the user. D-11 and every prior decision stay unchanged."
    - "git diff HEAD -- crates/ is empty, and the commit touches only 35-PROFILES.md, with additions only."
  artifacts:
    - path: target/phase35/mrp-protocol.txt
      provides: "Pre-registered A/A protocol (local, gitignored)"
    - path: target/phase35/aa_calib.py
      provides: "Frozen analysis script (stdlib only; SHA-256 recorded in the protocol)"
    - path: target/phase35/mrp-results.txt
      provides: "aa_calib.py output: metrics a-g, F, and the what-if table"
    - path: .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
      provides: "Committed calibration record"
      contains: "## D-11 calibration (quick 261010-mrp)"
  key_links:
    - from: target/phase35/abba-gated.sh
      to: "bin/spot-base-ibo and bin/spot-AA"
      via: "B legs run spot-base-ibo; A legs run spot-AA"
      pattern: "abba-gated.sh AA[123]"
    - from: target/phase35/aa_calib.py
      to: "AA<n>-*.jsonl and A3*/A3b*/A3r*/A3br* iso and targeted jsonl"
      via: "reads only counted (non-void) files"
      pattern: "median_ms_per_step"
---

<objective>
Calibrate the Phase 35 D-11 keep rule. Compare the final Phase 35 binary with an identical copy of itself (an A/A test) through the exact §Method protocol used to judge A3r in quick 261010-ibo. Do three rounds. Measure how often D-11 as written reports a confirmed regression when nothing changed, and how large per-scene noise is. Derive a noise-floor proposal F from a formula fixed in advance. Show what each earlier chain-edge decision (A3, A3b, A3r, A3br) would have been under F.

Purpose: The chain-edge hoist made liquid-tumbler 7–10% faster with identical fingerprints. It was reverted four times, each time because a different chain-free scene was 1–7% over its base max in both isolated pairs. The user chose "Calibrate first". They want evidence on false alarms, and they will decide the floor themselves before A3r is re-judged.

Output: measurement records under target/phase35/ (local), and one committed addition to 35-PROFILES.md. This plan is measurement and recording only. It does NOT change engine or test code, D-11, any decision, or any existing 35-PROFILES row.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@./CLAUDE.md
@.planning/phases/35-speed-up-the-slowest-scenes/35-CONTEXT.md

Read 35-PROFILES.md only in §Method through §A/B procedure (lines 43-90), the A3, A3b, A3r and A3br rows of §Attempts, and the "Quick 261010-ibo run notes" paragraph (line 311). Lines are long, so use `cut -c1-800` or grep. Do not load the whole file into context.

Read these local files in full: `target/phase35/ibo-protocol.txt`, `target/phase35/abba-gated.sh`, `target/phase35/keep_rule.py`, `target/phase35/A3r.decision`, `target/phase35/A3br.decision`.

<locked_rules>
- Measurement and recording only. Make no edits under `crates/`, `web/`, `xtask/` or any test. `git diff HEAD -- crates/` must stay empty throughout.
- Do not modify `abba-gated.sh` or `keep_rule.py`. The A/A run must use the protocol exactly as A3r was judged.
- Never time while any cargo or rustc process runs, in any repository. `abba-gated.sh` enforces this per leg. Do not run cargo, rustc, clippy, `just`, or bright-builds checks at any point in this plan: no code changes, so no build is needed.
- The protocol file and the analysis script are written and hashed before the first timed leg. No set is re-run because of its numbers. A set is voided only by the result-blind gate (cargo or rustc running, or 1-minute load above 10.0 at the end of a leg). At most 2 re-runs are allowed; the third run uses `NO_VOID=1` and counts regardless.
- Never overwrite failed or voided records. `abba-gated.sh` moves voided jsonl into `PREFIX.void-<epoch>/`. Also move that attempt's keep output into the same directory.
- Never run mdformat on `.planning/**`. Stage explicit paths only; never `git add -A` or `git add .`. Never touch the untracked empty file `=` in the repo root or `.planning/config.json`.
- Work on the main tree, not a worktree, because `target/phase35/` holds the artifacts.
- The F value and the what-if table are a PROPOSAL for the user. Do not edit D-11 in 35-CONTEXT.md. Do not edit existing Attempts rows, decisions, target records or the Phase summary. Do not re-judge or re-apply A3r or A3br.
- Commit messages end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Long sets: run each `abba-gated.sh` invocation with the Bash tool's `run_in_background: true`, then wait for the completion notification. The gate alone can wait up to 1800 s per leg, which exceeds the foreground timeout. Do not use a foreground `sleep`.
</locked_rules>

<facts_checked_at_planning>
- HEAD `079de026d`. `git diff --stat HEAD -- crates/` is empty.
- `bin/spot-base-ibo` and `bin/spot-final` are hard links (inode 545134829, 3 links), SHA-256 `bd2b2eb59057ecbcc6e31d10337c8d7552f14764737042d53f816adc2b1612f5`. This is the final Phase 35 build.
- `before-full.jsonl` has 25 lines. Each jsonl line has the keys `scene`, `runs`, `median_ms_per_step`, `min_ms_per_step`, `max_ms_per_step` and `fingerprint`.
- Historical isolated sets for the what-if: `A3-iso-{b1,a1,a2,b2}.jsonl` (7 scenes, base spot-base-04), `A3b-iso-*` (12 scenes, base spot-base-04), `A3r-iso-*` (11 scenes, base spot-base-ibo), `A3br-iso-*` (12 scenes, base spot-base-ibo). Targeted sets: `A3-{b1,a1,a2,b2}.jsonl`, `A3b-*`, `A3r-*`, `A3br-*`. All four attempts had 25/25 fingerprints equal and a target gain in both pairs. Each was reverted only on the regression clause.
- In ibo, sets took about 1–2 min of timing each, and 5 sets were voided because other repositories ran cargo. There are no git hooks.
- Host at planning: 1-minute load 3.4, no cargo or rustc running.
</facts_checked_at_planning>

<interfaces>
abba-gated.sh usage (cwd target/phase35): `./abba-gated.sh PREFIX BASE AFTER RUNS SCENE...`
- Default legs `b1 a1 a2 b2` write `PREFIX-<leg>.jsonl`, append to `PREFIX.uptime`, then run `keep_rule.py` twice (b1/a1, b2/a2) with the scenes as targets.
- `LEGS="b1 a1"` and no scenes: full catalog, with one `keep_rule.py` call and empty targets, so every scene is screened.
- Exit 3 means void. The jsonl files were moved to `PREFIX.void-<epoch>/` and `VOID ...` was appended to `PREFIX.uptime`.

keep_rule.py output lines: `<scene>: before median X min Y max Z | after median W -> gain=Bool`, `fingerprint mismatches: ...`, `medians above before max (any scene without a gain): [...]`.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create the A/A binary, write the frozen analysis script, and pre-register the protocol</name>
  <files>target/phase35/bin/spot-AA, target/phase35/aa_calib.py, target/phase35/mrp-protocol.txt (all local, gitignored)</files>
  <action>
All paths are relative to `/Users/peterryszkiewicz/Repos/liquidfun-rs/target/phase35`.

1. Run `uptime` and `pgrep -x cargo; pgrep -x rustc` to record the starting state. Run `git diff --stat HEAD -- crates/` and confirm it is empty.
1. Create A with `cp bin/spot-base-ibo bin/spot-AA`. Use a copy, not a hard link. Every real A/B compared two separate files, and a hard link shares the inode with B. Run `shasum -a 256 bin/spot-base-ibo bin/spot-AA` and stop if the two hashes differ or differ from `bd2b2eb5…`. Run `ls -li` to confirm the inodes differ. Launch A once, untimed: `bin/spot-AA --warmup 1 --steps 1 --runs 1 --scene particles` (§Method: never time a first launch). If that launch stalls at `_dyld_start` under syspolicyd, wait for it to finish before any timing.
1. Write `aa_calib.py` (python3 stdlib only, `json`, `math`, `sys`, `pathlib`). It reads only counted files from the current directory and never reads `*.void-*` directories. Define `ratio% = (a_median / b_median - 1) * 100` and `excess% = (a_median / b_max - 1) * 100`, where a is the A or after leg and b is the B or base leg of the same pair. For targeted and isolated sets, pair 1 is b1/a1 and pair 2 is b2/a2. The full catalog has one pair, b1/a1. The script prints the following to stdout:
   - Per round k in 1..3 (prefixes `AA<k>`, `AA<k>-full`, `AA<k>-iso`):
     - (a) scenes with full `excess% > 0`. This must equal keep_rule's full listing; print both and flag any difference.
     - (b) confirmed scenes: isolated `excess% > 0` in both pairs.
     - (c) a table of each isolated scene with ratio% and excess% for pair 1 and pair 2.
     - The liquid-tumbler targeted ratio% for each pair, and whether `gain` (a_median < b_min) held in both pairs. A gain in both pairs counts as a false gain.
     - Fingerprint equality of `AA<k>-full-a1.jsonl` against `before-full.jsonl` (all 25).
     - If `AA<k>-iso-*` is absent, print "none listed".
   - (d) the false-alarm count: the number of rounds with at least one confirmed scene. This judges the D-11 regression clause alone. In A/A the target-gain clause fails by design, so it is reported separately as the false-gain count.
   - (e) noise distributions, one row per pair type (targeted, full, isolated) over all rounds: n, min, p50, p90 and max of ratio%, the same for excess%, and the count with excess% > 0. Use nearest-rank quantiles. Also print a per-scene table for all 25 scenes over full and isolated pairs: n pairs, max ratio%, max excess%, and the median base spread% `(b_max - b_min) / b_median * 100` across those pairs' B legs.
   - The floor:
     - M = the largest ratio% over every scene in every isolated pair (pair 1 and pair 2) of every counted round.
     - F = max(2, floor(M) + 1), the smallest whole percent strictly above M, with a minimum of 2.
     - If no round produced isolated pairs, print "M undefined" and F = 2.
     - Print M to 3 decimals. Print F on its own line, starting the line, in the form `F = <n>%`.
   - (g) a leave-one-round-out check, for information only: for each round k, compute F_k from the other two rounds' isolated pairs with the same formula. Report whether round k has a scene with ratio% > F_k in both pairs.
   - The what-if table, for information only, for attempts A3, A3b, A3r and A3br, using `<ID>-iso-*` and the targeted `<ID>-{b1,a1,a2,b2}.jsonl`:
     - per isolated scene, ratio% and excess% for both pairs
     - D-11-as-written confirmed scenes (excess% > 0 in both pairs), recomputed. These must reproduce the recorded confirmations: A3 soup-stirrer, A3b fountain, A3r jelly-drop and water-wheel, A3br wave-tank. Print MISMATCH if they do not.
     - floor-confirmed scenes (ratio% > F in both pairs)
     - target gain in both pairs
     - the what-if decision: keep if target gain in both pairs, no floor-confirmed scene, and fingerprints equal. All four recorded 25/25 equal (quote the decision files and 35-PROFILES rows), so treat fingerprints as equal.

   Accept the round list as argv (default `1 2 3`) so the script can run on partial data. Add a `--selftest` mode that treats the A3r targeted/full/iso files as a stand-in round, to check parsing only. Run `python3 aa_calib.py --selftest` once. It must reproduce the A3r confirmations jelly-drop and water-wheel and exit 0. This uses only data already published in 35-PROFILES. It does not touch A/A results, and F is fixed by formula, so it cannot bias anything.
1. Compute `shasum -a 256 aa_calib.py`. Then write `mrp-protocol.txt` before any timed leg. Include:
   - title `quick 261010-mrp, pre-registered before the first A/A timed leg`
   - `written_utc` from `date -u +%FT%TZ`, git HEAD, and the `uptime` line
   - B `bin/spot-base-ibo` and A `bin/spot-AA` with both SHA-256s and inodes, plus the sentence "A is a byte-identical copy; any difference between A and B is measurement noise. A/A does not exercise code-layout changes."
   - the run sequence in task 2 below, verbatim in substance, with prefixes AA1/AA2/AA3
   - the gate and void rules copied from ibo-protocol.txt unchanged
   - the D-11 and §Method steps 3-5 quotations copied from ibo-protocol.txt
   - the definitions of ratio%, excess%, and metrics (a) through (g)
   - the F formula with its exact wording: "a confirmed regression requires the scene's after median to exceed its base median by more than F% in BOTH isolated pairs; F = max(2, floor(M) + 1) where M is the largest A/A isolated-pair ratio% across all counted rounds". Also the screening step stays unchanged: the full catalog still lists scenes above base max, and only listed scenes get isolated pairs.
   - the what-if definition
   - `aa_calib.py` SHA-256
   - the rule "If aa_calib.py has a parsing bug after timing, fix only the bug, append an addendum with the reason and the new SHA-256, and never change the definitions or the formula."
   - the statement "F and the what-ifs are a proposal for the user; D-11 and all recorded decisions are unchanged by this plan."
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs/target/phase35 && test "$(shasum -a 256 bin/spot-AA | cut -d' ' -f1)" = bd2b2eb59057ecbcc6e31d10337c8d7552f14764737042d53f816adc2b1612f5 && test "$(stat -f %i bin/spot-AA)" != "$(stat -f %i bin/spot-base-ibo)" && python3 aa_calib.py --selftest && grep -q "$(shasum -a 256 aa_calib.py | cut -d' ' -f1)" mrp-protocol.txt && grep -q 'floor(M) + 1' mrp-protocol.txt && ! ls AA1*.jsonl 2>/dev/null && test -z "$(git -C ../.. diff HEAD -- crates/)"</automated>
  </verify>
  <done>spot-AA is a separate file, byte-identical to spot-base-ibo, and has been launched once untimed. aa_calib.py passes its selftest, reproducing the A3r confirmations. mrp-protocol.txt holds the design, metrics, F formula and script hash, and no AA timing file exists yet.</done>
</task>

<task type="auto">
  <name>Task 2: Run three gated A/A rounds and produce the results</name>
  <files>target/phase35/AA{1,2,3}*.jsonl, AA*.uptime, AA*.keep.txt, AA{1,2,3}.round, mrp-results.txt (all local, gitignored)</files>
  <action>
cwd `target/phase35`. For k = 1, 2, 3 in order, run these sets. Run each with Bash `run_in_background: true`, and wait for the completion notification before the next set.

1. Targeted: `./abba-gated.sh AA<k> bin/spot-base-ibo bin/spot-AA 5 liquid-tumbler > AA<k>.keep.txt 2>&1; echo exit=$? >> AA<k>.keep.txt`
1. Full catalog: `LEGS="b1 a1" ./abba-gated.sh AA<k>-full bin/spot-base-ibo bin/spot-AA 3 > AA<k>-full.keep.txt 2>&1; echo exit=$? >> AA<k>-full.keep.txt`. Check that both `AA<k>-full-b1.jsonl` and `AA<k>-full-a1.jsonl` have 25 lines.
1. Isolated: read the list from the `medians above before max` line of `AA<k>-full.keep.txt`. If the list is `none`, skip this set and record "none listed". Otherwise: `./abba-gated.sh AA<k>-iso bin/spot-base-ibo bin/spot-AA 5 <listed scenes in the printed order> > AA<k>-iso.keep.txt 2>&1; echo exit=$? >> AA<k>-iso.keep.txt`. Include liquid-tumbler if it is listed.

Void handling, the same as ibo: if `exit=3`, move `<prefix>.keep.txt` into the newest `<prefix>.void-<epoch>/` directory. Re-run the same set unchanged, at most twice. The third run uses `NO_VOID=1` and counts regardless. Never re-run a valid set. Never look at a voided set's numbers before deciding to re-run; the decision depends only on the exit code.

After each round's last set, write `AA<k>.round` before starting the next round. It holds:
- the UTC time span
- the 1-minute load range from the round's `.uptime` files
- the void, GATE-WAITED and GATE-TIMEOUT counts
- (a) the full-run listed scenes
- (b) the scenes listed by both isolated keep_rule lines, or "none listed"
- whether D-11's regression clause would have reverted this no-op

Do not run cargo or rustc between rounds. Rounds run back to back, with each leg gated.

After R3, run `python3 aa_calib.py 1 2 3 > mrp-results.txt` and check:
- the keep_rule cross-checks show no difference
- the four what-if D-11-as-written recomputations print no MISMATCH
- every round's full-run fingerprints are 25/25 equal to before-full.jsonl

A fingerprint difference in an A/A run would be a determinism defect. If one appears, stop and report it as a blocker, and do not proceed to Task 3. If aa_calib.py has a pure parsing bug, apply the addendum rule from mrp-protocol.txt.

Finally, confirm that `git diff HEAD -- crates/` is still empty.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs/target/phase35 && for k in 1 2 3; do test -f AA$k.round && test "$(wc -l < AA$k-full-b1.jsonl)" -eq 25 && test "$(wc -l < AA$k-full-a1.jsonl)" -eq 25 && for f in AA$k-b1 AA$k-a1 AA$k-a2 AA$k-b2; do test -s $f.jsonl; done || exit 1; done && grep -q '^F' mrp-results.txt && ! grep -q MISMATCH mrp-results.txt && test -z "$(git -C ../.. diff HEAD -- crates/)"</automated>
  </verify>
  <done>Three counted A/A rounds exist with targeted, full (25/25 lines) and isolated (or "none listed") sets. Each round has a .round record written before the next round started. mrp-results.txt holds metrics a-g, M, F and the what-if table, with no MISMATCH. Fingerprints are 25/25 equal in every round, and crates/ is unchanged.</done>
</task>

<task type="auto">
  <name>Task 3: Record the calibration in 35-PROFILES.md and commit it</name>
  <files>.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md</files>
  <action>
Append a new section at the very end of 35-PROFILES.md, after `## Notes for Phase 36`. Use the heading `## D-11 calibration (quick 261010-mrp)`. Do not edit, reorder or reformat any existing line. Write in plain, short sentences. Take numbers only from `mrp-results.txt`, the `.round` files and the `.uptime` files. Use the subsections below:

1. **Protocol.**
   - The A/A design: B `spot-base-ibo`, A `spot-AA` (a copy, same SHA-256 `bd2b2eb5…`, different inode).
   - The pre-registration: the `mrp-protocol.txt` written_utc, which is before the first leg, and the `aa_calib.py` SHA-256 prefix. Note any addendum.
   - The sequence: §Method unchanged through `abba-gated.sh` with the ibo gate and void rules, 3 rounds.
   - Definitions of ratio% and excess%, and the F formula quoted exactly from the protocol.
   - A note that an A/A test measures timing noise only and cannot show code-layout effects of a real change.
1. **Run notes.** One paragraph:
   - HEAD
   - each round's UTC span and 1-minute load range
   - void, GATE-WAITED and GATE-TIMEOUT counts, with reasons
   - that no cargo or rustc ran at the start or end of any counted leg
   - the uptime file names
   - that no build ran in this repository
1. **Per-round results.** One table with columns: Round | Full run listed above base max (a) | Isolated-pair confirmed (b) | D-11 would revert a no-op | liquid-tumbler targeted ratio% p1 / p2 (false gain?) | Fingerprints. Then one table per round of the isolated scenes, with columns: Scene | ratio% p1 | ratio% p2 | excess% p1 | excess% p2 | confirmed.
1. **False-alarm count (d).** "D-11's regression clause as written would have reverted N of 3 no-op rounds." Also report the false-gain count.
1. **Noise (e).** The pair-type distribution table, and the 25-scene per-scene table.
1. **Proposed floor.** M (3 decimals) and the scene and pair it came from, F, and the leave-one-round-out result (g). State plainly that F is a proposal for the user to accept, change or reject. D-11 is unchanged until the user decides.
1. **What-if (informational).** One table with columns: Attempt | Base | Target gain both pairs | Fingerprints | D-11-as-written confirmed (recorded decision) | max isolated ratio% (scene, pair) | Confirmed under F | Would-be decision under F. One row each for A3, A3b, A3r and A3br. Add the sentence: "These what-ifs change no decision. A3, A3b, A3r and A3br remain reverted; any re-judgement of A3r needs a fresh run after the user agrees a floor."

Then verify:
- `git diff --numstat HEAD -- .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md` shows 0 deleted lines
- `git diff HEAD -- crates/` is empty
- `git status --short` shows no change to `=` or `.planning/config.json`

Do not run mdformat. No code changed, so the cargo pre-commit checks do not apply. Do not run cargo now either, so that later timing is not disturbed.

Commit with `git add .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md` and `git commit -m "docs(35): record D-11 A/A calibration (quick 261010-mrp)" -m "<one-paragraph result: false-alarm count, M, proposed F; proposal only, no decision changed>" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"`. Commit only that file.

Write `.planning/quick/261010-mrp-calibrate-d-11-keep-rule-with-a-a-runs-o/261010-mrp-SUMMARY.md` with these frontmatter fields:
- `generated_by: gsd-executor`
- `lifecycle_mode: yolo`
- `phase_lifecycle_id: 261010-mrp`
- `generated_at`: the output of `node "$HOME/.claude/get-shit-done/bin/gsd-tools.cjs" current-timestamp full`

Include the usual summary fields. Do not include the SUMMARY in the 35-PROFILES commit; the quick workflow commits it.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs && grep -q '^## D-11 calibration (quick 261010-mrp)' .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md && test "$(git show --numstat --format= HEAD -- .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md | cut -f2)" = 0 && test "$(git show --name-only --format= HEAD)" = .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md && git log -1 --format=%B | tail -1 | grep -q 'Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>' && test -z "$(git diff HEAD -- crates/)" && grep -q 'phase_lifecycle_id: 261010-mrp' .planning/quick/261010-mrp-calibrate-d-11-keep-rule-with-a-a-runs-o/261010-mrp-SUMMARY.md</automated>
  </verify>
  <done>35-PROFILES.md ends with the calibration section, holding the protocol, run notes, per-round tables, false-alarm count, noise tables, the proposed F and the what-if table, all marked as a proposal. The commit touches only that file, with additions only, and ends with the Co-Authored-By line. The SUMMARY has the lifecycle frontmatter, and crates/ is unchanged.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| none external | Local timing of an existing binary and a planning-doc edit. No network, no input parsing, no credentials, no dependencies. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-mrp-01 | Tampering (evidence integrity) | F derivation | mitigate | mrp-protocol.txt fixes the metrics and the F formula, and aa_calib.py's SHA-256 is recorded before the first leg. Bug fixes need a logged addendum, and the definitions never change. |
| T-mrp-02 | Tampering (selective re-runs) | A/A sets | mitigate | Voiding is decided by abba-gated.sh from load and process data only, with at most 2 re-runs. Voided records are moved aside, never overwritten. The re-run decision uses the exit code only. |
| T-mrp-03 | Repudiation | decisions in 35-PROFILES | mitigate | The new section is appended only (0 deleted lines checked). It is labelled a proposal, and no decision or D-11 text changes. |
| T-mrp-04 | Tampering (measurement) | concurrent builds | mitigate | The per-leg gate excludes cargo and rustc and voids on load above 10. The executor runs no cargo in this plan. |
</threat_model>

<verification>
- `target/phase35/mrp-protocol.txt` written_utc precedes the first `start` line in `AA1.uptime`.
- `shasum -a 256 target/phase35/bin/spot-AA target/phase35/bin/spot-base-ibo` match (`bd2b2eb5…`).
- `target/phase35/mrp-results.txt` contains metrics (a)-(g), M, F and the four-row what-if table, with no MISMATCH.
- `git show --stat HEAD` lists only 35-PROFILES.md, with insertions only. `git diff HEAD -- crates/` is empty.
</verification>

<success_criteria>
- Three A/A rounds ran the unchanged §Method protocol behind the quiet-host gate, with every leg's uptime recorded.
- The false-alarm count, the per-scene noise and the pre-registered F are reported in 35-PROFILES.md, along with what A3, A3b, A3r and A3br would have been under F.
- Nothing about D-11, prior decisions or engine code changed. The user decides the floor.
</success_criteria>

<output>
After completion, create `.planning/quick/261010-mrp-calibrate-d-11-keep-rule-with-a-a-runs-o/261010-mrp-SUMMARY.md` with the lifecycle frontmatter listed in Task 3.
</output>
