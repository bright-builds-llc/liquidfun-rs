---
phase: 14-repair-windows-particle-group-invariants
type: ci-test-isolation-repair
status: locally-verified
---

# Acceptance history fixture correction

## Observed failure and scope

Cargo CI run `34809888461` at `fc3af47291750af418befe98ad437dc40aa87bb3` passed all three platform jobs and the newly provisioned fuzz smoke, then failed the live-repository ancestry test because the shallow quality checkout lacked historical commit `981908ea87b6789b6b6e9aa136e65a369c5c736d`. The target recorded 27 passing tests and one failure. Its actual quality log and terminal metadata remain in `target/phase14-platform/fc3af47291750af418befe98ad437dc40aa87bb3/attempt-20260914-01/`.

A full-history local check exposed the real current-HEAD replay closure drift against the frozen promotion record, after a container-local Git ownership guard was resolved using an invocation-scoped safe.directory for the known `/workspace` bind mount. That closure drift remains the explicit Phase 15 deferral. Neither this repair nor a unit-test pass makes current HEAD valid promotion evidence.

## Contract-preserving correction

The old test asserted that arbitrary current repository HEAD satisfied exact-head acceptance. Four separate integration tests now construct a disposable Git P→R→Q fixture and exercise the unchanged production `validate_repository_identity_at` path:

1. Valid schema-v2 promotion passes receipt, ledger, content, trailer, ancestry and closure validation.
1. A schema-v1 receipt fails with exactly `AcceptanceErrorKind::Schema`.
1. A metadata-only descendant A passes without altering either producer closure.
1. A changed replay input committed after Q fails with exactly `AcceptanceErrorKind::Closure`.

The fixture uses synthetic bytes and identities only. It writes its own minimal materials manifest, replay input, seven promoted outputs, schema-v2 receipt and ledgers beneath a newly created owned temporary directory, then removes that directory. It never writes repository canonical receipts or producer outputs. Empty external-material declarations avoid any oracle/source requirement while the tracked manifest itself makes the witness closure nonempty; the replay closure also contains real committed synthetic input bytes.

One cfg(test) child-module registration in the private acceptance facade lets the fixture reuse existing private hash helpers rather than duplicate semantic receipt normalization or digest algorithms. The fixture implementation lives under tests. Production validator function bodies and non-test builds are unchanged. No error is suppressed, no assertion accepts a current-HEAD closure failure, no suite is skipped, and no new release/provenance schema is introduced. The proposed full-history quality checkout was removed because the controlled fixture has no dependency on repository history.

## Verification

The four new cases initially pass in `local-attempt-20260914-acceptance-fixture01/gate.log` under `target/phase14-platform/`. The final `local-attempt-20260914-isolation01/gate-02.log` records ordered fmt, workspace-wide Clippy with denied warnings, all-target/all-feature core build and 1,012 all-feature core tests, followed by acceptance 31/31, evidence 24/24 and Phase 9 provenance 2/2. Both `third_party/liquidfun` and repository `target` are hidden by empty container overlays, while Cargo's build cache remains separate at `/cargo-target`; the synthetic history cases do not depend on either omitted tree. The first workspace Clippy pass reports two test-helper string-format allocations; those were corrected using fallible `write!` calls. All failed local logs remain retained separately. Fresh remote complete-workflow proof is still required by Plan 14-04.
