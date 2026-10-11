---
phase: quick-261010-rif
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/liquidfun/src/particle/contact_scan.rs
  - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
autonomous: true
requirements: [PERF-08, PERF-09]
generated_by: gsd-plan-phase
lifecycle_mode: yolo
phase_lifecycle_id: 261010-rif
generated_at: "2026-10-11T00:50:12Z"
must_haves:
  truths:
    - "target/phase35/rif-protocol.txt was written before the first A2r timed leg: its written_utc is earlier than the first start line in A2r.uptime. It holds the amended D-11 verbatim, the orchestrator's multi-target interpretation (clauses a, b, c), the SHA-256s of spot-base-rif, spot-A2r and spot-before, the A2r source digest, the gate, void and PRE-WAIT rules, and the frozen SHA-256 of rif_judge.py."
    - "spot-base-rif is a fresh release build of clean HEAD 7c0d9b9d2, which includes A3r2. spot-base-ibo is not used. spot-A2r is a fresh release build of HEAD plus attempts/A2.patch, and its source digest is in A2r.source-sha."
    - "rif_judge.py's selftest on the old 35-03 A2 data reports particles at +3.090% / +2.037% in the targeted pairs, not floor-confirmed. Its negative selftest (particles after legs scaled to above +4% in both pairs) prints DECISION: revert."
    - "The decision comes only from fresh A2r sets run through abba-gated.sh: targeted ABBA on all three targets, then the full catalog, then isolated pairs for the listed non-target scenes. No old A2 timing file is a decision input. A2r.decision was written before any diagnostic or cargo run."
    - "rif_judge.py keeps A2r only if: (a) at least one of liquid-tumbler, stacked-drip and particles has its after median below its base min in both targeted pairs; (b) 25 of 25 fingerprints equal before-full.jsonl, with no targeted-pair mismatch; (c) no non-gaining scene has an after median more than 4% above its base median in BOTH of its pairs. For non-gaining targets these are the targeted pairs, and for other listed scenes the isolated pairs."
    - "If kept: A2r-cum (spot-before vs spot-A2r, all three targets) was timed before any cargo run. All six checks passed, the committed crates/ diff digest equals A2r.source-sha, and the change is committed as perf(35). If reverted: crates/ equals HEAD."
    - "35-PROFILES.md has a new A2r Attempts row after the A3r2 row, a bullet, and a run note. The three target records and the three Phase summary rows reflect the outcome. No existing Attempts row (A0 to A3r2) changed."
  artifacts:
    - path: target/phase35/rif-protocol.txt
      provides: "Pre-registered protocol (local, gitignored)"
    - path: target/phase35/rif_judge.py
      provides: "Frozen multi-target judge applying F = 4% (local, gitignored)"
    - path: target/phase35/A2r.decision
      provides: "Decision written before any diagnostic or cargo run (local, gitignored)"
    - path: .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
      provides: "A2r Attempts row, bullet, run note, three target records and three Phase summary rows"
      contains: "| A2r |"
  key_links:
    - from: target/phase35/A2r.decision
      to: target/phase35/rif_judge.py
      via: "python3 rif_judge.py A2r, whose SHA-256 must match rif-protocol.txt"
      pattern: "DECISION: (keep|revert)"
    - from: "perf(35) commit"
      to: target/phase35/A2r.source-sha
      via: "git diff of the commit over crates/, piped to shasum -a 256, must equal the timed digest"
      pattern: "perf\\(35\\)"
---

<objective>
Re-judge A2 under the amended D-11 rule (F = 4%) with a fresh, pre-registered run. A2 is the bounded insertion sort for the retained contact-proxy order from 35-03. If the frozen judge keeps it, apply it, run the full checks, commit it, and record it. If the judge reverts it, record that and leave the engine unchanged.

Purpose: In 35-03, A2 beat A1 on liquid-tumbler (−5.5% / −6.1%) and stacked-drip (−5.0% / −5.7%). It was reverted because particles (+3.1% / +2.0%) and washing-machine (about +2%) were above the base max in both pairs. Those excesses are below the calibrated no-op noise (M = 3.096%) and the adopted floor F = 4%. The amendment applies only to new decisions, so a fresh run on the current engine is required. The current engine is HEAD, which includes A3r2.

Output (all local and gitignored unless noted):
- rif-protocol.txt and rif_judge.py
- spot-base-rif, spot-A2r and the A2r timing sets
- A2r.decision
- either a perf(35) commit or a restored tree
- the 35-PROFILES.md record, committed as docs(35)
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@./CLAUDE.md
@.planning/STATE.md

Template to mirror: .planning/quick/261010-npx-amend-d-11-with-a-4-percent-noise-floor-/261010-npx-PLAN.md and its SUMMARY. Read them only if a step below is unclear; this plan already gives every command.

Read 35-PROFILES.md (`.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`, 524 lines) only by line range, with sed -n:
- line 7: Phase summary paragraph
- lines 11, 13, 15: summary rows for liquid-tumbler, stacked-drip and particles
- line 32: A3r2 bullet. Line 33 is blank; line 34 starts "A3 and A3b stay reverted".
- line 286: Attempts header (11 columns)
- line 291: the A2 row, which is the format to follow
- line 301: the A3r2 row, the last Attempts row
- line 315: Quick 261010-npx run notes
- lines 319, 321, 323: target records
- line 381: Notes for Phase 36, item 2, about A2

Binding rule: 35-CONTEXT.md lines 41–42 (D-11 and its 2026-10-10 amendment). Copy them verbatim into the protocol with `sed -n 41,42p`.

<facts>
The planner checked these on 2026-10-11 at 00:50Z:
- **Repository state**
  - HEAD is `7c0d9b9d2`.
  - `git diff 154ace4bf HEAD -- crates/` is empty, so the engine is the committed A3r2.
  - `git status --porcelain` shows only the untracked `=`. Do not touch `=` or `.planning/config.json`.
- **The patch**
  - `git apply --check target/phase35/attempts/A2.patch` passes.
  - The patch's SHA-256 is `0fac8e716d69d887d85de0cc44c233badb578dd114dba82d407e944d260b10a2`.
  - It changes only `crates/liquidfun/src/particle/contact_scan.rs` (+45/−1, including a unit test `insertion_budget_exhaustion_matches_full_sort`) and creates no new file, so `git add -N` is not needed.
- **Binary hashes**
  - `bin/spot-before`: `5bfc7b614d0701b6c9ba0606dd3b1b0937c5fc931676dff55c294cb6467e0742`
  - `bin/spot-A3r2`: `f028bf5866e49bccb14070458697bd3cee5a9db60b1bbbf63940b7558a2381b9`. A clean-HEAD build is expected to be byte-identical to it.
  - `bin/spot-base-ibo` (`bd2b2eb5…`) predates A3r2 and must NOT be used.
  - The old `bin/spot-A2` (`a6e039ca…`) is on the A1 base and is not used.
- **Script hashes**
  - `abba-gated.sh`: `d7f7e05508c7f640f89e5995b283a55c1ccc03d8c545a8515f81d5fc35704917`
  - `keep_rule.py`: `e2dafec624d61751eba4be23c30eb41f62ca9d771117c5c3abf2ce2f8f059c38`
  - `npx_judge.py`: `7fdbd876d10210db0f36cf536ac8fa1081a2926a44d1626034150123e7dd7174`. Do not edit it; write a new file.
- **Build**
  - `cargo build --release -p liquidfun-wasm --bin playground-scene-spot` writes `target/release/playground-scene-spot`. Copy it with `cp`, never a link.
  - The wasm32 build needs `--lib`.
- **jsonl format**
  - Record fields: `scene`, `median_ms_per_step`, `min_ms_per_step`, `max_ms_per_step`, `timed_out`, `fingerprint`, `runs`.
- **Script output**
  - abba-gated.sh calls keep_rule.py, which prints `medians above before max (any scene without a gain): [...]` or `none`.
  - In full mode (`LEGS="b1 a1"`, no scenes) the target list is empty, so every scene above its base max is listed, targets included.
- **Old A2 data (35-03, base spot-A1)** in target/phase35:
  - Targeted: `A2-{b1,a1,a2,b2}.jsonl`, three scenes.
  - Full: `A2-base-full.jsonl` (base) and `A2-full.jsonl` (after). There is no keep.txt and no `-full-b1/-a1` naming.
  - Isolated: `A2-iso-{b1,a1,a2,b2}.jsonl` (fountain, impulse, theo-jansen, washing-machine).
- **Values computed from the old A2 data**
  - Targeted ratio% (pair 1 / pair 2):
    - liquid-tumbler −5.534 / −6.138, gain both
    - stacked-drip −5.032 / −5.770, gain both
    - particles +3.090 / +2.037, no gain. Its excess% is +2.872 / +0.867, so d11_as_written is true.
  - Full listing with no targets: fountain, impulse, particles, theo-jansen, washing-machine. Minus the targets this leaves exactly the old isolated set.
  - Fingerprints: A2-full vs before-full are 25/25 equal.
  - Isolated ratio% (pair 1 / pair 2), with excess% in brackets:
    - fountain +1.745 / +1.701 (+1.221 / −0.825)
    - impulse +1.193 / +0.835 (+0.823 / −0.760)
    - theo-jansen +0.109 / −0.844 (−1.682 / −3.055)
    - washing-machine +2.249 / +1.302 (+2.063 / +1.110), so d11_as_written is true
  - No scene is above +4% in both pairs.
- **Commit trailer check:** use `git log -1 --format='%(trailers:only)'`.
- **macOS launch stalls are fixed:** fresh binaries launch instantly. `cargo test --workspace --all-features` takes about 20 min.
</facts>

<interpretation>
The orchestrator decided how to apply the amended D-11 to this multi-target attempt. Record it in the protocol verbatim, before any timing. TARGETS = liquid-tumbler, stacked-drip, particles; F = 4%.
- (a) Target gain: at least one target has its after median below its base min in both targeted pairs. The targeted ABBA covers all three targets.
- (b) Fingerprints: 25 of 25 equal `before-full.jsonl`. A fingerprint mismatch between legs of a targeted pair also fails (b).
- (c) Regression floor: F applies to every scene that did not meet the gain rule, including non-gaining targets. This matches how keep_rule.py has treated non-gaining targets since the 35-02 revision. Such a scene is a confirmed regression only if its after median exceeds its base median by MORE THAN 4% in BOTH pairs:
  - for non-gaining targets, the two targeted pairs;
  - for other scenes, the isolated pairs. These run for each non-target scene that the full catalog run lists above its base max.

  Gaining targets are not regression-checked, and targets are never re-run in the isolated set. The single-pair maximum per scene and d11_as_written (excess% > 0 in both pairs) are reported as information only.
</interpretation>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Write and freeze rif_judge.py, build spot-base-rif and spot-A2r, and pre-register rif-protocol.txt</name>
  <files>target/phase35/rif_judge.py, target/phase35/rif-selftest/, target/phase35/bin/spot-base-rif, target/phase35/bin/spot-A2r, target/phase35/A2r.source-sha, target/phase35/attempts/A2r.patch, target/phase35/rif-warm.txt, target/phase35/rif-protocol.txt, crates/liquidfun/src/particle/contact_scan.rs (patch applied, uncommitted)</files>
  <action>
Paths are relative to the repository root unless a step says cwd target/phase35. Run no timed leg in this task.

1. **Judge** `target/phase35/rif_judge.py`. Use only the standard library, and start from a copy of `npx_judge.py`; leave npx_judge.py unchanged.
   - **Usage:** `python3 rif_judge.py ID`, run from the directory that holds the files. It reads only plain files in the cwd and never reads `*.void-*`.
   - **Constants:** `TARGETS = ("liquid-tumbler", "stacked-drip", "particles")`, `F_PCT = 4.0`, `CATALOG_SIZE = 25`.
   - **Definitions** (a = after leg, b = base leg of the same pair; pair 1 = b1/a1, pair 2 = b2/a2):
     - ratio% = (a_median / b_median − 1) × 100
     - excess% = (a_median / b_max − 1) × 100
     - gain = a_median < b_min
     - Compare unrounded values and print them to 3 decimals.
   - **Sections, printed in order:**
     1. **Targeted.** Read `ID-{b1,a1,a2,b2}.jsonl`; all three targets must be present in each leg. For each target and pair, print base median/min/max, after median, ratio%, excess% and gain. A target gains only if it gains in both pairs. Print `gaining targets: ...`. For each non-gaining target, print `floor_confirmed` (ratio% > F_PCT in BOTH targeted pairs), `d11_as_written` (excess% > 0 in both) and `single_pair_max` (max of the two ratio%). Then print each pair's fingerprint mismatches between b and a.
     1. **Full.** Read `ID-full-{b1,a1}.jsonl`. Each needs 25 records, `timed_out` false and equal scene sets. Listed = every scene with a_median > b_max, sorted. Parse the single `medians above before max (any scene without a gain):` line in `ID-full.keep.txt`. It must equal the listing, or the judge prints `LISTING MISMATCH` (an input error). Print `isolated set` = listed minus TARGETS, sorted.
     1. **Fingerprints.** `before-full.jsonl` vs `ID-full-a1.jsonl`: all 25 present in both and equal. Print `fingerprints: 25/25 equal` or the mismatching scenes.
     1. **Isolated.** If the isolated set is empty, print `isolated: none listed` and read no isolated file. Otherwise read `ID-iso-{b1,a1,a2,b2}.jsonl`; each scene set must equal the isolated set. Per scene, print ratio% and excess% for both pairs, `floor_confirmed`, `d11_as_written` and `single_pair_max`.
     1. **Decision.** Print the floor-confirmed list across non-gaining targets and isolated scenes. Print `DECISION: keep` only if all hold:
        - (a) at least one gaining target
        - (b) no targeted-pair mismatch and 25/25 equal
        - (c) nothing floor-confirmed

        Otherwise print `DECISION: revert (<each failed clause>)`.
   - **Errors:** missing file, wrong count, `timed_out` true, duplicate scene, scene-set mismatch or `LISTING MISMATCH` prints `INPUT ERROR: <reason>`, no DECISION line, and exit 2. Exit 0 after a decision.

2. **Selftests** (tooling validation on old data, not decision inputs). Create `target/phase35/rif-selftest/` and copy files in, renaming:
   - `A2-{b1,a1,a2,b2}.jsonl` → `A2old-{same}.jsonl`
   - `A2-base-full.jsonl` → `A2old-full-b1.jsonl`
   - `A2-full.jsonl` → `A2old-full-a1.jsonl`
   - `A2-iso-{leg}.jsonl` → `A2old-iso-{leg}.jsonl`
   - `before-full.jsonl` unchanged

   Then, in that directory:
   1. Run `python3 ../keep_rule.py A2old-full-b1.jsonl A2old-full-a1.jsonl "" > A2old-full.keep.txt`.
   1. Run `python3 ../rif_judge.py A2old > ../rif-selftest.txt`. It must show:
      - liquid-tumbler −5.534 / −6.138 and stacked-drip −5.032 / −5.770, both gaining
      - particles +3.090 / +2.037: not gaining, floor_confirmed False, d11_as_written True
      - the listing fountain, impulse, particles, theo-jansen, washing-machine, with no LISTING MISMATCH
      - isolated set fountain, impulse, theo-jansen, washing-machine
      - fingerprints 25/25 equal
      - isolated ratios matching the facts; d11_as_written True only for washing-machine
      - floor-confirmed none
      - `DECISION: keep`
   1. **Negative selftest.** Write `A2neg-*` copies of the A2old files, with particles' `median_ms_per_step` in `A2neg-a1.jsonl` and `A2neg-a2.jsonl` multiplied by 1.05 using a small Python snippet. Regenerate `A2neg-full.keep.txt` the same way. Running `python3 ../rif_judge.py A2neg > ../rif-selftest-neg.txt` must show particles floor_confirmed True and `DECISION: revert (... particles)`.
   1. Check that `python3 ../rif_judge.py nosuch` exits 2 with `INPUT ERROR`.

   Fix the script until all of these hold. Then freeze it: `shasum -a 256 target/phase35/rif_judge.py`. Do not edit it after hashing.

3. **Base build.**
   - Confirm `git diff HEAD -- crates/` and `git status --porcelain -- crates/` are both empty; if not, stop and report.
   - Run `cargo build --release -p liquidfun-wasm --bin playground-scene-spot`, then `cp target/release/playground-scene-spot target/phase35/bin/spot-base-rif`.
   - Record its SHA-256 and inode, and `cmp` it with `bin/spot-A3r2`; byte-identical is expected.
   - If its SHA-256 starts with `bd2b2eb5` (the pre-A3r2 engine), stop and report. Otherwise, a non-identical result is recorded but is not a blocker, because the fresh build is the base.

4. **After build.**
   - Run `git apply target/phase35/attempts/A2.patch`.
   - Run `git diff HEAD -- crates/ | shasum -a 256 > target/phase35/A2r.source-sha` and `git diff HEAD -- crates/ > target/phase35/attempts/A2r.patch`, and record whether A2r.patch is byte-identical to A2.patch.
   - Build the release spot binary again and `cp` it to `target/phase35/bin/spot-A2r`. Record its SHA-256 and inode.
   - Leave the patch applied. Run no cargo command from here until A2r.decision exists (and, if kept, A2r-cum is counted).

5. **Warm launch.**
   - Wait until `pgrep -x cargo || pgrep -x rustc` matches nothing. The editor's rust-analyzer may run a `cargo check` after the patch, so wait for it.
   - From cwd target/phase35, launch `bin/spot-A2r`, `bin/spot-base-rif` and `bin/spot-before` once each, untimed: `--warmup 1 --steps 1 --runs 1 --scene particles`. Each must exit 0.
   - Log the UTC time, exit code and duration of each launch to `rif-warm.txt`.

6. **Pre-register** `target/phase35/rif-protocol.txt` before any timed leg. Use npx-protocol.txt as the layout model. Include:
   - **Run identity:** `written_utc`, git HEAD, `uptime`, and whether cargo or rustc is running.
   - **Binaries:** base `bin/spot-base-rif`, after `bin/spot-A2r` and cumulative base `bin/spot-before`, each with full SHA-256 and inode, plus the spot-A3r2 comparison.
   - **Source:** the source digest and the A2.patch comparison.
   - **Judge:** its SHA-256 and the selftest and negative-selftest results.
   - **Rule:** 35-CONTEXT.md lines 41–42 verbatim, followed by the `<interpretation>` block above verbatim, labeled as the orchestrator's decision.
   - **Definitions** as in step 1.
   - **Gate block:** copy npx-protocol.txt lines 45–61 verbatim (find them with `grep -n`), i.e. the gate, void, mrp and PRE-WAIT rules and the tooling-error rule, with the current script hashes.
   - **Run order:** the exact Task 2 commands.
   - **Inputs:** no old A2 timing file (A2-*.jsonl, A2old/A2neg selftest copies) is a decision input.
   - **Decision timing:** A2r.decision is written before any diagnostic or cargo run. Diagnostics are named `A2r-diag-*` and never change the decision.
   - **Post-timing bug rule:** if rif_judge.py has a parsing bug after timing, fix only that bug, append an addendum with the reason and the new SHA-256, never change F, TARGETS or definitions, and never re-time to fix a judge problem.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs/target/phase35 && grep -q "DECISION: keep" rif-selftest.txt && grep -q "+3.090" rif-selftest.txt && ! grep -q "MISMATCH" rif-selftest.txt && grep -q "DECISION: revert" rif-selftest-neg.txt && test -x bin/spot-base-rif && test -x bin/spot-A2r && test -s A2r.source-sha && test -s attempts/A2r.patch && grep -q "$(shasum -a 256 rif_judge.py | cut -c1-64)" rif-protocol.txt && grep -q "$(shasum -a 256 bin/spot-A2r | cut -c1-64)" rif-protocol.txt && grep -q "$(shasum -a 256 bin/spot-base-rif | cut -c1-64)" rif-protocol.txt && grep -q "MORE THAN 4%" rif-protocol.txt && ! ls A2r-b1.jsonl A2r.uptime 2>/dev/null</automated>
  </verify>
  <done>The judge is frozen; its selftest reproduces the old A2 what-if and its negative selftest reverts. spot-base-rif is a fresh clean-HEAD build that includes A3r2. spot-A2r is built from HEAD plus A2.patch, with its digest recorded. All three binaries are warm-launched. rif-protocol.txt exists, and no A2r timing file exists yet.</done>
</task>

<task type="auto">
  <name>Task 2: Run the fresh gated protocol and write A2r.decision</name>
  <files>target/phase35/A2r*.jsonl, target/phase35/A2r*.uptime, target/phase35/A2r*.keep.txt, target/phase35/A2r.decision</files>
  <action>
Use cwd `target/phase35` throughout. Run no cargo, rustc, clippy, just or bright-builds command. The gate may wait up to 1,800 s per leg, so start each set with `run_in_background: true` and wait for its `exit=` line. Never time two sets at once, and run each set to completion before the next.

1. **Targeted:**
   ```
   ./abba-gated.sh A2r bin/spot-base-rif bin/spot-A2r 5 liquid-tumbler stacked-drip particles > A2r.keep.txt 2>&1; echo exit=$? >> A2r.keep.txt
   ```
1. **Full catalog:**
   ```
   LEGS="b1 a1" ./abba-gated.sh A2r-full bin/spot-base-rif bin/spot-A2r 3 > A2r-full.keep.txt 2>&1; echo exit=$? >> A2r-full.keep.txt
   ```
   Both `A2r-full-b1.jsonl` and `A2r-full-a1.jsonl` must have 25 lines.
1. **Isolated:** take the scenes on the `medians above before max` line of `A2r-full.keep.txt`, minus liquid-tumbler, stacked-drip and particles, in printed order. Run:
   ```
   ./abba-gated.sh A2r-iso bin/spot-base-rif bin/spot-A2r 5 <scenes> > A2r-iso.keep.txt 2>&1; echo exit=$? >> A2r-iso.keep.txt
   ```
   If no scene remains, skip this set and record "none listed".

**Voids:**
- On `exit=3`, move that set's `.keep.txt` into its newest `.void-<epoch>/` without reading the numbers, then re-run the same command.
- At most 2 re-runs. Before a third run, do the logged 300 s PRE-WAIT (cap 3,000 s), then run with `NO_VOID=1`; that run counts regardless.
- No valid set is re-run.
- Any other non-zero exit is a tooling error: stop and diagnose without reading timings for a decision.

**Decision.** Immediately after the last set, check that `shasum -a 256 rif_judge.py` equals the protocol value. Then write `A2r.decision`, before any diagnostic or cargo run. It contains:
- a header: written UTC; base and after binaries with SHA-256 prefixes; source digest; judge hash; and the counted sets' UTC spans, 1-minute load ranges, and void, GATE-WAITED, GATE-TIMEOUT and PRE-WAIT counts, taken from the `.uptime` files
- the full output of `python3 rif_judge.py A2r`
- for each clause (a), (b), (c), the amended D-11 text it applies

If the judge exits 2, apply the post-timing bug rule and re-run the judge; never re-time.

**If revert:** run `git apply -R target/phase35/attempts/A2r.patch` from the repository root. Then `git status --porcelain -- crates/` and `git diff HEAD -- crates/` must both be empty.

**If keep:** before any cargo command, run the cumulative pair (evidence only, not a decision input). Re-run the warm launch of `spot-before` first if more than an hour has passed. The same void rules apply.
```
./abba-gated.sh A2r-cum bin/spot-before bin/spot-A2r 5 liquid-tumbler stacked-drip particles > A2r-cum.keep.txt 2>&1; echo exit=$? >> A2r-cum.keep.txt
```
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs/target/phase35 && test -s A2r.decision && grep -Eq "DECISION: (keep|revert)" A2r.decision && grep -q "exit=0" A2r.keep.txt && grep -q "exit=0" A2r-full.keep.txt && test "$(wc -l < A2r-full-a1.jsonl | tr -d ' ')" = 25 && test "$(wc -l < A2r-full-b1.jsonl | tr -d ' ')" = 25 && (grep -q "DECISION: revert" A2r.decision && test -z "$(git -C ../.. status --porcelain -- crates/)" || grep -q "exit=0" A2r-cum.keep.txt)</automated>
  </verify>
  <done>The fresh targeted, full and (if needed) isolated sets are counted under the gate. A2r.decision holds the frozen judge's output and was written before any diagnostic or cargo run. If reverted, crates/ equals HEAD. If kept, the cumulative spot-before vs spot-A2r set exists.</done>
</task>

<task type="auto">
  <name>Task 3: Check and commit if kept, then record A2r in 35-PROFILES.md</name>
  <files>crates/liquidfun/src/particle/contact_scan.rs (only if kept), .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md</files>
  <action>
**If kept**, follow these steps in order. Every cargo run happens after all timing.

1. **Checks.** Copy `target/phase35/A3r2-checks.sh` to `A2r-checks.sh`, replace `A3r2` with `A2r` in it, and run it in the background. It writes `A2r-checks.log` and `A2r-checks.results` and runs, sequentially:
   1. `cargo fmt --all --check`
   1. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   1. `cargo build --workspace --all-targets --all-features`
   1. `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown`
   1. `bun scripts/bright-builds-check.ts all`
   1. `cargo test --workspace --all-features`

   Every exit code must be 0. A3r2-checks.results shows all six at 0 on the previous HEAD. If a check fails and the fix needs a source change, the tree no longer matches the timed digest. In that case, do not commit: restore the tree as in Task 2, record A2r as "kept by the judge, not committed: <check> failed", and do not re-time in this plan.
1. **Digest gate.** `git diff HEAD -- crates/ | shasum -a 256` must equal `A2r.source-sha`.
1. **Commit.** Stage only `crates/liquidfun/src/particle/contact_scan.rs`; never use `git add -A`. Commit as `perf(35): bounded insertion sort for the retained proxy order (A2r, kept under amended D-11)`. The body gives:
   - each target's targeted ratios and gaining status
   - 25/25 fingerprints
   - the floor-confirmed result
   - the source digest

   End the body with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
1. **Post-commit checks.** `git diff HEAD~1 HEAD -- crates/ | shasum -a 256` must equal `A2r.source-sha`, and `git log -1 --format='%(trailers:only)'` must show the trailer.

**In every outcome**, edit 35-PROFILES.md. Never run mdformat on `.planning/**`.

1. **Attempts row.** Insert an `A2r` row directly after the A3r2 row (line 301), with the same 11 columns, following the A2 row (291) and A3r2 row (301):
   - Plan: `quick 261010-rif`.
   - Change: "fresh rebuild of A2 (`attempts/A2.patch`) on HEAD `7c0d9b9d2` (includes A3r2) judged under amended D-11 (F = 4%)", plus the `spot-A2r` SHA prefix and source digest prefix.
   - Targets: the three scenes.
   - Base: `spot-base-rif` median (min-max) per target for pairs 1 and 2.
   - After: medians per target.
   - Fingerprints: 25/25 and the file name.
   - Regression: the gaining targets; each non-gaining target's targeted ratio% for pairs 1 and 2; the full listing; each isolated scene's ratio% for pairs 1 and 2; the floor-confirmed list; and the single-pair maxima and d11_as_written as information only.
   - Decision: kept or reverted.
   - Commit: the hash, or "none" with the patch file.
   - Reason: the clauses that held or failed. If kept, include the cumulative `spot-before` vs `spot-A2r` figures per target.
1. **Bullet.** Add a bullet for the A2r outcome after the A3r2 bullet (line 32).
1. **Run notes.** Add a "Quick 261010-rif run notes" paragraph after the npx run notes (line 315). It records:
   - HEAD and the binaries with hashes, inodes and the spot-A3r2 comparison
   - the source digest
   - the protocol written time and the judge hash
   - the selftest results
   - the warm launches
   - the counted set spans and loads, with the `.uptime` file names
   - voids, waits and pre-waits
   - that A2r.decision preceded any diagnostic or cargo run
   - if kept: the cumulative pair and the six check results with times
1. **Target records.** Append the A2r outcome, with that target's ratios, to the liquid-tumbler, stacked-drip and particles records (lines 319, 321, 323). If kept, add the cumulative figures and the commit.
1. **Phase summary rows.** In rows 11, 13 and 15, append `A2r` to Attempts. If kept, also:
   - append `A2r` to Kept
   - append `; after A2r (quick 261010-rif): X / Y` to Final median, using the cumulative pair
   - append the after-A2r cumulative percentages to Gain beyond noise; "yes" still requires the cumulative after median to be below the before min in both pairs
   - add one sentence to the line 7 paragraph saying the engine now also includes A2r (commit hash)
1. **Notes for Phase 36.** Append one sentence giving the outcome to item 2, the A2 idea at line 381.
1. **Leave unchanged:** every existing Attempts row (A0 to A3r2), the §Final verification tables and the calibration section, byte for byte.

Commit only 35-PROFILES.md as `docs(35): record A2r re-judgement under amended D-11 (quick 261010-rif)`, ending with the same Co-Authored-By line.
  </action>
  <verify>
    <automated>cd /Users/peterryszkiewicz/Repos/liquidfun-rs && P=.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md && grep -q "^| A2r | quick 261010-rif" $P && grep -q "Quick 261010-rif run notes" $P && test "$(git diff HEAD~1 HEAD -- $P | grep -E '^-[^-]' | grep -vcE '^-(Final commit|\| (liquid-tumbler|stacked-drip|particles) \||- (liquid-tumbler|stacked-drip|particles):|  1\. The bounded insertion sort)')" = 0 && (grep -q "DECISION: keep" target/phase35/A2r.decision && git log --format=%s -2 | grep -q "^perf(35)" && test "$(git diff HEAD~2 HEAD~1 -- crates/ | shasum -a 256)" = "$(cat target/phase35/A2r.source-sha)" || test -z "$(git status --porcelain -- crates/)") && git log -1 --format='%(trailers:only)' | grep -q "Co-Authored-By: Claude Opus 5.5"</automated>
  </verify>
  <done>Kept: all six checks passed, the perf(35) commit's crates/ digest equals the timed digest, and the cumulative figures are recorded. Reverted or not committed: crates/ equals HEAD and the reason is recorded. Either way, the A2r row, bullet, run note, three target records and three summary rows are committed in docs(35), and no prior Attempts row changed.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Shared host → timing data | Other repositories' cargo jobs and load can bias the timings |
| Executor → keep decision | The executor sees results and could bias re-runs, the rule or the multi-target interpretation |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-rif-01 | Tampering (evidence integrity) | A2r.decision, rif_judge.py | mitigate | Protocol (with the orchestrator's interpretation verbatim) and judge hash written before the first leg. Hash re-checked before deciding. Decision written before any diagnostic. Voiding uses the exit code only. Old A2 timings are never inputs. A positive and a negative selftest show the judge can both keep and revert. |
| T-rif-02 | Repudiation | perf(35) commit | mitigate | The commit's crates/ digest must equal `A2r.source-sha`. The patch is saved as `attempts/A2r.patch`. |
| T-rif-03 | Denial of service (measurement) | abba-gated.sh legs | mitigate | Per-leg cargo/rustc and load gate. A set is void if load is above 10. At most 2 re-runs, with a logged PRE-WAIT. No cargo of ours runs during timing. |
| T-rif-04 | Tampering (behavior) | contact_scan.rs sort | mitigate | 25/25 fingerprints against `before-full.jsonl`, the patch's budget-exhaustion unit test, and the full `cargo test` before commit. |
</threat_model>

<verification>
- rif-protocol.txt was written before the first start line in `A2r.uptime`, and its judge hash equals the file's hash at decision time.
- `A2r.decision` holds exactly one DECISION line from the frozen judge. Its mtime is earlier than any `A2r-diag-*` file and than `A2r-checks.log`.
- The repository ends in exactly one of two states: a perf(35) commit whose digest matches the timed digest, or a crates/ tree equal to HEAD.
- 35-PROFILES.md has the A2r row, and no prior Attempts row changed.
</verification>

<success_criteria>
- A2r was judged on fresh data against the current engine, which includes A3r2. The judge applied the amended D-11 (F = 4%) under the pre-registered multi-target interpretation.
- The outcome is recorded truthfully in 35-PROFILES.md: either kept, with the commit and cumulative figures, or reverted, with the reason.
- No timed leg ran while cargo or rustc ran. `=` and `.planning/config.json` were not touched.
</success_criteria>

<output>
After completion, create `.planning/quick/261010-rif-re-judge-a2-bounded-insertion-sort-under/261010-rif-SUMMARY.md`. Its frontmatter must include `generated_by: gsd-executor`, `lifecycle_mode: yolo`, `phase_lifecycle_id: 261010-rif`, and `generated_at` from `node "$HOME/.claude/get-shit-done/bin/gsd-tools.cjs" current-timestamp full`.
</output>
