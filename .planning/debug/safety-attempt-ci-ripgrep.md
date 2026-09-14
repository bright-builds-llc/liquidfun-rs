---
status: verifying
trigger: "Cargo CI producers_preserve_failures_and_bound_diagnostics fails with missing terminal.json"
created: 2026-09-14T21:56:44Z
updated: 2026-09-14T23:03:53Z
---

## Current Focus

hypothesis: Subsequent current-source failure includes an uncopied inventory helper; actual Linux retained run lacked enough output to identify its failing step.
test: Trace real fixture and retain all output; copy current producer's actual inventory helper dependency chain.
expecting: Full local traced attempt succeeds; next Linux exact preflight independently establishes runner behavior.
next_action: Parent runs combined gates and publishes; inspect early exact Linux attempt test and its failure artifact if needed.

## Symptoms

expected: Synthetic producer failures create bounded, immutable terminal diagnostics.
actual: Cargo CI cannot open miri-fail diagnostics/terminal.json.
errors: jq cannot open terminal.json; assertion at attempts.rs:26.
reproduction: Run attempts.sh with PATH excluding rg.
started: Candidate 00358a585f8629dbd7b4b4a486da6af0cfe94c7c Cargo CI.

## Eliminated

- hypothesis: Provider SHA/workspace overrides fixture source.
  evidence: Producers use git fixture identity and explicit positional candidate; workflow/run/attempt/job are explicitly set by attempts.sh.
- hypothesis: Terminal recording itself fails.
  evidence: Producer exits in check_contract before begin_attempt; outer.log identifies missing rg.

## Evidence

- checked: Retained Cargo CI log.
  found: 19 safety tests pass and attempt test fails at first Miri failure terminal lookup.
- checked: Workflow prerequisites.
  found: Cargo CI installs Rust and Python tooling but does not provision ripgrep.
- checked: No-rg PATH reproduction.
  found: Exact missing terminal symptom; target/phase15-safety-ci-repair/red-no-rg/miri-fail/outer.log reports Miri source scan requires rg.
  implication: Synthetic fixture's uncontrolled host tool dependency masks intended command failure.
- checked: Production scanner repair and simplification review.
  found: The sole ASCII regular expression uses only portable ERE constructs. Replaced rg invocation and prerequisite with grep -Ei; no fixture scanner adapter remains.
- checked: New scanner controls.
  found: Real uppercase CMAKE, ORACLE, THIRD_PARTY, SUBMODULE and missing file all reject; clean file succeeds. Injected missing/error/finding controls still reject. Eleven scanner and adjacent modes passed; scanner-controls.log retains results.
- checked: Full producer controls with no rg and inherited GitHub fields.
  found: All five variants passed using modern Bash in target/phase15-safety-ci-repair/green-bash5-no-rg/. Prior preliminary runs using macOS /bin/bash 3 reached an unrelated empty-array failure in release validation and remain retained in green-no-rg/ and green-production-no-rg/; those are failed attempts, not passing evidence.
- checked: Native Cargo suite and source checks.
  found: 15 native safety contract tests passed; Linux-gated attempt/scanner tests require direct shell controls on macOS, completed above. shfmt, ShellCheck with source paths and diff checks passed.
- checked: Actual subsequent Linux Cargo CI run 34905390484 at 4e9f1edefba1f0a9a06abdc8fa2c44eaab725c1f.
  found: Attempt assertion fails with empty stderr after 4.04 seconds; 20 adjacent tests pass. Retained log target/phase15-canonical/attempt-03/cargo-ci-failed.log does not identify the failed shell statement.
- checked: Current-source traced fixture reproduction.
  found: target/phase15-safety-ci-repair/traced-preflight-01/coverage-differential-success/outer.log reports missing scripts/phase15-inventory-check.sh and inventory command exit 127. Source had acquired this transitive dependency after the earlier passing fixture.
- checked: Fixture dependency repair.
  found: Copying the real inventory shell/Python helpers plus explicit unpromoted fixture README makes all five traced producer variants pass in traced-preflight-02/. No production edit or predicate relaxation. Actual Linux follow-up remains outstanding.
- checked: Diagnostic CI improvements.
  found: Rust subprocess uses bash -x with line-number PS4 and reports exit/root/stdout/stderr. Cargo CI runs the exact attempt test after cache restore before workspace validation and uploads failed attempt fixtures. actionlint, rustfmt, shfmt, diff checks and ShellCheck with existing subshell informational exclusions pass.

## Resolution

root_cause: Attempt fixture controls compiler/build tools but leaves ripgrep as an undeclared host prerequisite; Miri validates it before diagnostic recording starts.
fix: Replace incidental rg source scan dependency with grep -Ei using identical ASCII predicate; preserve all fail-closed branches.
verification: RED retained in target/phase15-safety-ci-repair/red-no-rg/; passing full no-rg producer controls in green-bash5-no-rg/; scanner-controls.log records 11 passing modes. Parent owns final gates and CI.
files_changed: [scripts/phase12-miri.sh, tools/xtask/tests/safety_evidence_contract/miri-control.sh, tools/xtask/tests/safety_evidence_contract/miri.rs]
