//! Fail-closed paired execution and raw-report assembly.

use liquidfun::DiagnosticProfileSchema;
use liquidfun_test_protocol::performance::{
    BenchmarkHarnessFailureKind, BenchmarkPerformanceResult, BenchmarkRunOutcome,
    BenchmarkRunRequest, BenchmarkRunResult, PerformanceEngineRole, validate_benchmark_run_pair,
};

use super::{
    PairedBenchmarkOutcome, PairedBenchmarkPlan, PairedEngineOrder, PairedHarnessFailure,
    PairedPerformanceReport, PairedPhysicsMismatch, PairedRawSample, RustChildProfileDiagnostic,
};
use crate::performance::PairedBenchmarkAdapter;

struct AcceptedPerformance {
    result: BenchmarkRunResult,
    rust_child_diagnostics: Box<[RustChildProfileDiagnostic]>,
    process_generation: u64,
}

#[derive(Default)]
struct ResetIdentities {
    native: ResetIdentity,
    oracle: ResetIdentity,
}

#[derive(Default)]
struct ResetIdentity {
    process_generation: u64,
    reset_epoch: u64,
}

/// Executes one complete five-run same-host session with exact per-sample interleaving.
#[must_use]
pub fn run_paired_benchmark<N, O>(
    plan: &PairedBenchmarkPlan,
    native: &mut N,
    oracle: &mut O,
) -> PairedBenchmarkOutcome
where
    N: PairedBenchmarkAdapter,
    O: PairedBenchmarkAdapter,
{
    if let Err(outcome) = validate_adapter_roles(native, oracle) {
        return outcome;
    }
    let mut raw_samples = Vec::with_capacity(
        usize::from(plan.policy.baseline_runs())
            .saturating_mul(usize::from(plan.policy.samples_per_engine())),
    );
    let mut reset_identities = ResetIdentities::default();
    for baseline_run in 1..=plan.policy.baseline_runs() {
        for sample_ordinal in 1..=plan.policy.samples_per_engine() {
            let request = match paired_request(plan, baseline_run, sample_ordinal) {
                Ok(request) => request,
                Err(outcome) => return outcome,
            };
            let order = if sample_ordinal % 2 == 1 {
                PairedEngineOrder::NativeThenOracle
            } else {
                PairedEngineOrder::OracleThenNative
            };
            let pair = match execute_pair(
                native,
                oracle,
                &request,
                baseline_run,
                order,
                &mut reset_identities,
            ) {
                Ok(pair) => pair,
                Err(outcome) => return outcome,
            };
            let (native, oracle) = pair;
            let BenchmarkRunOutcome::Performance(native_performance) = native.result.outcome()
            else {
                return harness_failure(
                    baseline_run,
                    sample_ordinal,
                    PerformanceEngineRole::NativeRust,
                    BenchmarkHarnessFailureKind::AdapterFailure,
                );
            };
            let BenchmarkRunOutcome::Performance(oracle_performance) = oracle.result.outcome()
            else {
                return harness_failure(
                    baseline_run,
                    sample_ordinal,
                    PerformanceEngineRole::PinnedCppOracle,
                    BenchmarkHarnessFailureKind::AdapterFailure,
                );
            };
            if native_performance
                .semantic_checkpoint_identity()
                .checkpoint_id()
                != oracle_performance
                    .semantic_checkpoint_identity()
                    .checkpoint_id()
            {
                return harness_failure(
                    baseline_run,
                    sample_ordinal,
                    PerformanceEngineRole::PinnedCppOracle,
                    BenchmarkHarnessFailureKind::IdentityMismatch,
                );
            }
            if native_performance
                .semantic_checkpoint_identity()
                .checkpoint_sha256()
                != oracle_performance
                    .semantic_checkpoint_identity()
                    .checkpoint_sha256()
            {
                return PairedBenchmarkOutcome::PhysicsMismatch(PairedPhysicsMismatch {
                    baseline_run,
                    sample_ordinal,
                    engine_role: PerformanceEngineRole::PinnedCppOracle,
                    semantic_checkpoint_identity: oracle_performance
                        .semantic_checkpoint_identity()
                        .clone(),
                });
            }
            raw_samples.push(raw_sample(
                baseline_run,
                sample_ordinal,
                order,
                &native,
                native_performance,
                &oracle,
                oracle_performance,
            ));
        }
    }
    PairedBenchmarkOutcome::Performance(Box::new(PairedPerformanceReport {
        identity: plan.report_identity.clone(),
        compatibility_status: plan.compatibility_status,
        policy: plan.policy.clone(),
        profile_schema: DiagnosticProfileSchema::Phase12V1.as_str(),
        raw_samples: raw_samples.into_boxed_slice(),
    }))
}

fn paired_request(
    plan: &PairedBenchmarkPlan,
    baseline_run: u8,
    sample_ordinal: u16,
) -> Result<BenchmarkRunRequest, PairedBenchmarkOutcome> {
    plan.request(baseline_run, sample_ordinal)
        .map_err(|_error| {
            harness_failure(
                baseline_run,
                sample_ordinal,
                PerformanceEngineRole::NativeRust,
                BenchmarkHarnessFailureKind::AdapterFailure,
            )
        })
}

fn validate_adapter_roles<N, O>(native: &N, oracle: &O) -> Result<(), PairedBenchmarkOutcome>
where
    N: PairedBenchmarkAdapter,
    O: PairedBenchmarkAdapter,
{
    if native.engine_role() != PerformanceEngineRole::NativeRust {
        return Err(harness_failure(
            1,
            1,
            PerformanceEngineRole::NativeRust,
            BenchmarkHarnessFailureKind::IdentityMismatch,
        ));
    }
    if oracle.engine_role() != PerformanceEngineRole::PinnedCppOracle {
        return Err(harness_failure(
            1,
            1,
            PerformanceEngineRole::PinnedCppOracle,
            BenchmarkHarnessFailureKind::IdentityMismatch,
        ));
    }
    Ok(())
}

fn execute_pair<N, O>(
    native: &mut N,
    oracle: &mut O,
    request: &BenchmarkRunRequest,
    baseline_run: u8,
    order: PairedEngineOrder,
    reset_identities: &mut ResetIdentities,
) -> Result<(AcceptedPerformance, AcceptedPerformance), PairedBenchmarkOutcome>
where
    N: PairedBenchmarkAdapter,
    O: PairedBenchmarkAdapter,
{
    match order {
        PairedEngineOrder::NativeThenOracle => {
            let native =
                execute_adapter(native, request, baseline_run, &mut reset_identities.native)?;
            let oracle =
                execute_adapter(oracle, request, baseline_run, &mut reset_identities.oracle)?;
            Ok((native, oracle))
        }
        PairedEngineOrder::OracleThenNative => {
            let oracle =
                execute_adapter(oracle, request, baseline_run, &mut reset_identities.oracle)?;
            let native =
                execute_adapter(native, request, baseline_run, &mut reset_identities.native)?;
            Ok((native, oracle))
        }
    }
}

fn execute_adapter<A: PairedBenchmarkAdapter>(
    adapter: &mut A,
    request: &BenchmarkRunRequest,
    baseline_run: u8,
    reset_identity: &mut ResetIdentity,
) -> Result<AcceptedPerformance, PairedBenchmarkOutcome> {
    let engine_role = adapter.engine_role();
    let output = adapter.execute(request, baseline_run).map_err(|kind| {
        harness_failure(
            baseline_run,
            request.identity().sample_ordinal(),
            engine_role,
            kind,
        )
    })?;
    let result = output.result();
    if result.engine_role() != engine_role || validate_benchmark_run_pair(request, result).is_err()
    {
        return Err(harness_failure(
            baseline_run,
            request.identity().sample_ordinal(),
            engine_role,
            BenchmarkHarnessFailureKind::IdentityMismatch,
        ));
    }
    if !advance_reset_identity(
        reset_identity,
        output.process_generation(),
        result.reset_epoch(),
    ) {
        return Err(harness_failure(
            baseline_run,
            request.identity().sample_ordinal(),
            engine_role,
            BenchmarkHarnessFailureKind::AdapterResetFailure,
        ));
    }
    match result.outcome() {
        BenchmarkRunOutcome::Performance(_) => Ok(AcceptedPerformance {
            result: result.clone(),
            rust_child_diagnostics: output.rust_child_diagnostics().into(),
            process_generation: output.process_generation(),
        }),
        BenchmarkRunOutcome::PhysicsMismatch(mismatch) => Err(
            PairedBenchmarkOutcome::PhysicsMismatch(PairedPhysicsMismatch {
                baseline_run,
                sample_ordinal: request.identity().sample_ordinal(),
                engine_role,
                semantic_checkpoint_identity: mismatch.semantic_checkpoint_identity().clone(),
            }),
        ),
        BenchmarkRunOutcome::HarnessFailure(failure) => Err(harness_failure(
            baseline_run,
            request.identity().sample_ordinal(),
            engine_role,
            failure.kind(),
        )),
    }
}

fn advance_reset_identity(
    previous: &mut ResetIdentity,
    process_generation: u64,
    reset_epoch: u64,
) -> bool {
    let valid = if previous.process_generation == 0 {
        process_generation == 1 && reset_epoch == 1
    } else if process_generation == previous.process_generation {
        previous
            .reset_epoch
            .checked_add(1)
            .is_some_and(|expected| reset_epoch == expected)
    } else {
        previous
            .process_generation
            .checked_add(1)
            .is_some_and(|expected| process_generation == expected)
            && reset_epoch == 1
    };
    if valid {
        previous.process_generation = process_generation;
        previous.reset_epoch = reset_epoch;
    }
    valid
}

fn raw_sample(
    baseline_run: u8,
    sample_ordinal: u16,
    engine_order: PairedEngineOrder,
    native: &AcceptedPerformance,
    native_performance: &BenchmarkPerformanceResult,
    oracle: &AcceptedPerformance,
    oracle_performance: &BenchmarkPerformanceResult,
) -> PairedRawSample {
    PairedRawSample {
        baseline_run,
        sample_ordinal,
        engine_order,
        native_nanoseconds: native_performance.unprofiled_nanoseconds(),
        oracle_nanoseconds: oracle_performance.unprofiled_nanoseconds(),
        native_process_generation: native.process_generation,
        oracle_process_generation: oracle.process_generation,
        native_reset_epoch: native.result.reset_epoch(),
        oracle_reset_epoch: oracle.result.reset_epoch(),
        semantic_checkpoint_identity: native_performance.semantic_checkpoint_identity().clone(),
        native_common_parent_diagnostics: native_performance
            .maybe_common_parent_diagnostics()
            .unwrap_or_default()
            .into(),
        oracle_common_parent_diagnostics: oracle_performance
            .maybe_common_parent_diagnostics()
            .unwrap_or_default()
            .into(),
        rust_child_diagnostics: native.rust_child_diagnostics.clone(),
    }
}

const fn harness_failure(
    baseline_run: u8,
    sample_ordinal: u16,
    engine_role: PerformanceEngineRole,
    kind: BenchmarkHarnessFailureKind,
) -> PairedBenchmarkOutcome {
    PairedBenchmarkOutcome::HarnessFailure(PairedHarnessFailure {
        baseline_run,
        sample_ordinal,
        engine_role,
        kind,
    })
}
