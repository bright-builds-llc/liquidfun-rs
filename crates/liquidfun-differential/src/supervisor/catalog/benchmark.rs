//! One-record benchmark exchange over the shared bounded catalog supervisor.

use std::{io::Write, time::Instant};

use liquidfun_test_protocol::{
    HarnessFailureKind, HarnessLimits,
    performance::{
        BenchmarkHarnessFailureKind, BenchmarkRunRequest, BenchmarkRunResult,
        PerformanceEngineRole, decode_benchmark_run_result_jsonl,
        encode_benchmark_run_request_jsonl, validate_benchmark_run_pair,
    },
};

use super::super::{
    IoEvent, ReadyChild, enforce_total_output, receive_with_output_precedence,
    reconcile_request_output,
};

pub(super) fn run_benchmark_request(
    ready: &mut ReadyChild,
    request: &BenchmarkRunRequest,
    expected_reset_epoch: u64,
    limits: &HarnessLimits,
) -> Result<BenchmarkRunResult, BenchmarkHarnessFailureKind> {
    let baseline = ready.output_boundary;
    ready.last_request_baseline = baseline;
    let request_bytes = encode_benchmark_run_request_jsonl(request, limits)
        .map_err(|_error| BenchmarkHarnessFailureKind::MalformedRecord)?;
    let stdin = ready
        .io
        .maybe_stdin
        .as_mut()
        .ok_or(BenchmarkHarnessFailureKind::ChildNonZeroExit)?;
    stdin
        .write_all(&request_bytes)
        .and_then(|()| stdin.flush())
        .map_err(|_error| BenchmarkHarnessFailureKind::ChildNonZeroExit)?;
    let deadline = Instant::now() + limits.request_timeout();
    loop {
        let event = receive_with_output_precedence(
            &ready.io.workers,
            deadline,
            HarnessFailureKind::RequestTimeout,
            baseline,
            limits,
        )
        .map_err(map_harness_kind)?;
        match event {
            IoEvent::StdoutRecord(bytes) => {
                let result = decode_benchmark_run_result_jsonl(&bytes, limits)
                    .map_err(|_error| BenchmarkHarnessFailureKind::MalformedRecord)?;
                validate_benchmark_run_pair(request, &result)
                    .map_err(|_error| BenchmarkHarnessFailureKind::IdentityMismatch)?;
                if result.engine_role() != PerformanceEngineRole::PinnedCppOracle {
                    return Err(BenchmarkHarnessFailureKind::IdentityMismatch);
                }
                if result.reset_epoch() != expected_reset_epoch {
                    return Err(BenchmarkHarnessFailureKind::AdapterResetFailure);
                }
                ready.output_boundary =
                    reconcile_request_output(&ready.io.workers, deadline, baseline, limits, None)
                        .map_err(|failure| map_harness_kind(failure.kind))?;
                return Ok(result);
            }
            IoEvent::OutputProgress(total) => {
                enforce_total_output(total, baseline, limits).map_err(map_harness_kind)?;
            }
            IoEvent::SanitizerDetected => {
                return Err(BenchmarkHarnessFailureKind::SanitizerReport);
            }
            IoEvent::StdoutRecordTooLarge => {
                return Err(BenchmarkHarnessFailureKind::OutputLimitExceeded);
            }
            IoEvent::StdoutPartial | IoEvent::StdoutEof | IoEvent::ReadFailure => {
                return Err(BenchmarkHarnessFailureKind::ChildNonZeroExit);
            }
        }
    }
}

pub(super) const fn map_harness_kind(kind: HarnessFailureKind) -> BenchmarkHarnessFailureKind {
    match kind {
        HarnessFailureKind::StartupTimeout | HarnessFailureKind::RequestTimeout => {
            BenchmarkHarnessFailureKind::RequestTimeout
        }
        HarnessFailureKind::ChildNonZeroExit => BenchmarkHarnessFailureKind::ChildNonZeroExit,
        HarnessFailureKind::ChildSignaled => BenchmarkHarnessFailureKind::ChildSignaled,
        HarnessFailureKind::SanitizerReport => BenchmarkHarnessFailureKind::SanitizerReport,
        HarnessFailureKind::RecordTooLarge
        | HarnessFailureKind::TraceTooLarge
        | HarnessFailureKind::TotalOutputExceeded => {
            BenchmarkHarnessFailureKind::OutputLimitExceeded
        }
        HarnessFailureKind::WrongProvenance
        | HarnessFailureKind::RequestIdMismatch
        | HarnessFailureKind::TraceIdentityMismatch => {
            BenchmarkHarnessFailureKind::IdentityMismatch
        }
        HarnessFailureKind::AdapterResetFailure => BenchmarkHarnessFailureKind::AdapterResetFailure,
        HarnessFailureKind::CppAdapterFailure => BenchmarkHarnessFailureKind::AdapterFailure,
        _ => BenchmarkHarnessFailureKind::MalformedRecord,
    }
}
