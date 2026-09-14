# Cargo-only performance fixture repair

Recorded 2026-09-14. This concerns private CLI test isolation, not measured performance or canonical evidence.

## Failure

Cargo CI run `34800684869`, source `e58e028a31ffe845c5465e8d72dd5d933c9d80f5`, Linux quality job `103842638038` failed three `performance_cli` tests. Eight other tests in that target passed.

- `paired_invokes_provider_once_per_sealed_case_and_preserves_raw_values`
- `paired_rejects_harness_failure_without_writing_completion_identity`
- `paired_check_validates_sealed_inputs_without_running_measurements`

All three reached `oracle_release` because `/workspace/target/reference` (the corresponding runner repository path) was absent. The fixture isolated output paths but retained the actual repository root; the check-mode test used the production environment. The production resolver deliberately runs before the fake provider and before returning from check mode. A local oracle therefore masked the incomplete fixture.

The exact CI log is retained in `target/phase14-platform/e58e028a31ffe845c5465e8d72dd5d933c9d80f5/attempt-20260914-initial-repair/quality.log`.

## Applied fixture correction

Only `tools/xtask/tests/performance_cli.rs` changes. Each test claims a fresh fixture root, copies the two reviewed policy/manifest files, and creates its own regular executable sentinel at the resolver's reviewed release path. The sentinel is intentionally not a real executable; unexpected execution fails. Check mode uses this fixture as well.

Two negative tests remove only their own sentinel and verify that both paired mode and check mode still return `oracle_release`, make zero provider calls, and write no completion identity. Production validation, provider behavior, accepted evidence, and Cargo-only CI isolation remain intact. Old test directories are not deleted or reused.

## Verification status

Formatting and diff checks passed. The exact no-native-source Docker test attempt was prepared with empty target/upstream overlays, but Docker failed before starting the container with an image-store I/O error. Log: `target/phase14-ci-performance/attempt-20260914-fixture/focused.log`. The fixture correction is not yet claimed to pass its focused tests or the full quality job, and remains uncommitted until the required gates pass.

## Post-restart verification

After the user-approved Docker Desktop restart, the fixture target passed all 13 tests with an empty target directory and empty native-checkout overlay. Both missing-oracle negative cases passed. Full-workspace Clippy also passed, and the required ordered core format/lint/build/test gate passed 1012 tests. Logs: target/phase14-ci-performance/attempt-20260914-after-restart/focused.log and target/phase14-local-verification/attempt-20260914-resumed-gate/gate.log. Remote quality acceptance remains pending.
