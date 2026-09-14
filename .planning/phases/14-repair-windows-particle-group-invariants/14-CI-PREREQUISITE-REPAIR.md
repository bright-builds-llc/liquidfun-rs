# Phase 14 quality-lane prerequisite repair

Candidate ce37c5632507df2be9baf61305967a57a68567d4 passed all three default-feature platform jobs, but Cargo CI run 34808501136, quality job 103864994314, failed two of 66 phase13_1_gap_verification tests. The failed log and metadata remain in the candidate attempt directory.

## Preserved contracts

The sole Phase 13.1 deferral remains exact-head Phase 13 acceptance after later producer changes, addressed in Phase 15. The test now reads the structured frontmatter deferral and requires exactly one truth and exactly one Phase 15 destination, rather than obsolete report-body prose. The verified Phase 13.1 report itself is unchanged.

The lifecycle fuzz test deliberately performs a real cargo +nightly-2026-07-15 fuzz build and requires the tracked lockfile and Git status to remain unchanged. The quality lane now installs the same nightly/rust-src and cargo-fuzz 0.13.2 --locked pins already used in fuzz.yml. No test is skipped, no oracle is added to Cargo-only CI, and no producer or Phase 15 acceptance criterion changes.

## Verification

Actionlint passed. The exact deferral test passed locally, full-workspace Clippy passed, and the ordered format/lint/build/test gate passed 1,012 core tests. Log: target/phase14-local-verification/attempt-20260914-quality-prerequisites/gate.log. The real fuzz-build smoke and final quality outcome remain for the fresh CI candidate; these local checks do not claim that CI acceptance.
