---
phase: 09-particle-storage-lifecycle-and-coupling
fixed_at: 2026-07-18T19:57:52Z
review_path: .planning/phases/09-particle-storage-lifecycle-and-coupling/09-REVIEW.md
iteration: 3
findings_in_scope: 6
fixed: 5
skipped: 1
status: partial
---

# Phase 09: Code Review Fix Report

**Fixed at:** 2026-07-18T19:57:52Z
**Source review:** `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-REVIEW.md`
**Iteration:** 3
**Loop outcome:** Automatic review/fix loop cap reached

**Cumulative summary:**

- Findings reviewed across iterations: 6
- Fixed: 5
- Remaining: 1
- Status: partial

## Fixed Issues

### Iteration 1 — CR-01: Evidence cleanup follows symlinked output roots outside `target/`

**Files modified:** `scripts/phase9-evidence.sh`, `tools/xtask/src/phase9_evidence.rs`, `tools/xtask/tests/phase9_evidence_cli.rs`, `crates/liquidfun-differential/tests/phase9_corpus.rs`
**Commit:** 155ffc9
**Applied fix:** Canonicalized the repository and `target/` roots, rejected symlinks in every existing path component before output creation, cleanup, reads, or archive access, and revalidated created output paths beneath the canonical target root. Added final-output, ancestor-output, and exact-ref archive-ancestor regressions that preserve an external marker.

### Iteration 1 — WR-01: Exact-ref validation does not prove most witness bindings against their bound observations

**Files modified:** `crates/liquidfun-differential/src/rigid_world.rs`, `crates/liquidfun-differential/src/rigid_world/phase9.rs`, `crates/liquidfun-differential/src/rigid_world/phase9/evidence.rs`, `crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json`, `crates/liquidfun-differential/tests/phase9_corpus.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/phase9.rs`, `tools/xtask/src/phase9_evidence.rs`, `tools/xtask/tests/phase9_evidence_cli.rs`
**Commit:** fac6573
**Status:** fixed: requires human verification
**Applied fix:** Added one typed resolver/evaluator shared by corpus execution and evidence validation. It verifies the reviewed action ID, checkpoint interval, particle-observation ordinal and variant, and result-local semantic assertions against both decoded results. The validator recomputes the complete native/oracle comparison rather than trusting the stored match payload. Seven stale bindings were corrected, and digest-recomputed regressions cover action, checkpoint, observation, lifetime, contact, listener, filter, and divergent-result mutations.

### Iteration 1 — IN-01: Obsolete constructor code remains commented out

**Files modified:** `crates/liquidfun/src/particle/storage.rs`
**Commit:** 13aa7fd
**Applied fix:** Removed the obsolete block-commented constructor call while leaving the active `from_owned_lanes` construction path unchanged.

### Iteration 2 — WR-01: Five declared result-evidence assertions still collapse to a particle-presence marker

**Files modified:** `.github/workflows/oracle.yml`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/phase9.rs`, `crates/liquidfun-differential/src/rigid_world.rs`, `crates/liquidfun-differential/src/rigid_world/phase9.rs`, `crates/liquidfun-differential/src/rigid_world/phase9/evidence.rs`, `crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json`, `crates/liquidfun-differential/tests/phase9_corpus.rs`, `tools/xtask/src/phase9_evidence.rs`, `tools/xtask/tests/phase9_evidence_cli.rs`
**Commit:** 1e621f4
**Status:** fixed: requires human verification
**Applied fix:** Reclassified the five relationships as case-level evidence and added closed typed proof records bound to branch, request, native-result, and oracle-result digests. The corpus persists replay, repeated-D0, debug, release, minimized, and copied result payloads. The shared evaluator reloads the payloads and recomputes replay equality, exact D0 bytes, debug/release agreement, mismatch signatures, semantic paths, and first divergence. Evidence manifests now use schema 3 and case-record schema 2, and identity/archive file sets include proof payloads.

### Iteration 2 — WR-02: Superseded Phase 09 authority is still advertised as current platform evidence

**Files modified:** `reference/compatibility.json`, `COMPATIBILITY.md`, `TESTING.md`, `tools/xtask/src/inventory/validation.rs`, `tools/xtask/tests/inventory_cli.rs`
**Commit:** cb7397c
**Applied fix:** Demoted `platform_validated` to `not_evidenced` with no references for the four scoped particle rows and regenerated the compatibility report. Marked run `29652578231`, artifacts `8431920189` and `8431922578`, commit `22b31c0…`, and their archive, manifest, and semantic digests as superseded pre-WR-01 authority. The inventory validator rejects the old authority set, while `TESTING.md` retains it only as forensic history.

## Skipped / Remaining Issues

### Iteration 3 — WR-01: Cross-run proof references can alias the same persisted result

**Files:** `crates/liquidfun-differential/src/rigid_world/phase9/evidence.rs:140-287`, `tools/xtask/src/phase9_evidence.rs:612-641`, `tools/xtask/src/phase9_evidence.rs:656-679`, `tools/xtask/tests/phase9_evidence_cli.rs:673-703`
**Reason:** The automatic review/fix loop reached its iteration cap before another source-change cycle. No iteration-3 source edit or commit was attempted.
**Original issue:** The validator recomputes the contents and predicates for every referenced proof payload but does not validate proof-path topology. A digest-recomputed manifest can alias an independent pair, reuse baseline result paths, or collapse minimized/copied evidence onto one persisted result while still satisfying the content predicates and deduplicated exact-file set.
**Recommended next fix:** Define a canonical topology beneath `cases/<case-id>/proofs/`. Reject baseline request/result/comparison paths; require replay-native/replay-oracle, debug/release, and minimized/copied paths to be pairwise distinct; and explicitly permit only reviewed reuse across proof families. Add identity- and digest-recomputed regressions for baseline substitution, every independent-pair alias, and a first-divergence-only semantic-path mutation.

## Verification Evidence

Before every committed finding, the required Rust gate passed in order:

1. `cargo fmt --all`
1. `cargo clippy --all-targets --all-features -- -D warnings`
1. `cargo build --all-targets --all-features`
1. `cargo test --all-features`

Additional completed verification:

- `cargo test -p liquidfun-differential --test phase9_corpus`: 26 passed, 1 explicit fixture-regeneration test ignored
- `cargo test -p liquidfun-differential --test particle_oracle`: 13 passed
- `cargo test -p liquidfun-differential --test particle_protocol`: 25 passed
- `cargo test -p xtask --test phase9_evidence_cli`: 14 passed, including digest-recomputed regressions for all five cross-run proof kinds
- `cargo test -p xtask --test inventory_cli`: 21 passed
- Fresh local canonical and sanitizer evidence generation completed with provenance, inventory, content, and read-only checks
- Local paired evidence validation passed with 7 cases and all 58 semantic bindings
- Canonical and sanitizer manifests were byte-identical with SHA-256 `fa5e2ad1d25f074416eb86df1fd3c71404064a2f608f5f0cea0340e7612896b6`
- The schema-v3 semantic-manifest SHA-256 was `224fcfcb52ca842303510433387ca16541995c3515d7b0ca398dc334b4354be4`
- `cargo xtask inventory generate` ran twice with identical `COMPATIBILITY.md` and `reference/compatibility.json` hashes
- `cargo xtask inventory check` verified 177 compatibility rows
- `cargo xtask provenance check` passed for upstream oracle `7f20402173fd143a3988c921bc384459c6a858f2`
- `just markdown-check`, `bash -n scripts/phase9-evidence.sh`, and `git diff --check` passed in the completed fix cycles

No external authority run was dispatched and no compatibility promotion was performed. The four platform rows remain `not_evidenced`; a fresh exact-ref canonical/sanitizer pair is still required before promotion.

***

_Fixed through: 2026-07-18T19:57:52Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 3 — automatic loop cap reached_
