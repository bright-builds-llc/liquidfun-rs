use std::{io::Write, time::Instant};

use liquidfun_test_protocol::{
    HarnessFailureKind, HarnessLimits, LastValidRecord, RecordLimit, ScenarioRequestRecord,
    TraceRecord, TraceValidator, decode_trace_record_jsonl, encode_jsonl,
};

use super::{
    CapturedOracleTrace, IoEvent, ReadyChild, RequestFailure, classify_trace_decode,
    enforce_total_output, receive_with_output_precedence, reconcile_request_output,
};

#[allow(
    clippy::too_many_lines,
    reason = "the linear receive loop makes one in-flight protocol state machine auditable"
)]
pub(super) fn run_request(
    ready: &mut ReadyChild,
    request: &ScenarioRequestRecord,
    limits: &HarnessLimits,
) -> Result<CapturedOracleTrace, RequestFailure> {
    let baseline = ready.output_boundary;
    ready.last_request_baseline = baseline;
    let bytes = encode_jsonl(request, limits, RecordLimit::Input).map_err(|_| RequestFailure {
        kind: HarnessFailureKind::CppAdapterFailure,
        maybe_last_record: Some(LastValidRecord::Handshake),
    })?;
    let Some(stdin) = ready.io.maybe_stdin.as_mut() else {
        return Err(RequestFailure {
            kind: HarnessFailureKind::UnexpectedEof,
            maybe_last_record: Some(LastValidRecord::Handshake),
        });
    };
    stdin
        .write_all(&bytes)
        .and_then(|()| stdin.flush())
        .map_err(|_| RequestFailure {
            kind: HarnessFailureKind::UnexpectedEof,
            maybe_last_record: Some(LastValidRecord::Handshake),
        })?;

    let deadline = Instant::now() + limits.request_timeout();
    let mut records = Vec::new();
    let mut jsonl = Vec::from(ready.handshake_jsonl.as_ref());
    let mut trace_bytes = 0_usize;
    let mut stream_state = 0_u8;
    let mut maybe_last_record = Some(LastValidRecord::Handshake);
    loop {
        if ready.io.workers.sanitizer_detected() {
            return Err(RequestFailure {
                kind: HarnessFailureKind::SanitizerReport,
                maybe_last_record,
            });
        }
        let event = receive_with_output_precedence(
            &ready.io.workers,
            deadline,
            HarnessFailureKind::RequestTimeout,
            baseline,
            limits,
        )
        .map_err(|kind| RequestFailure {
            kind,
            maybe_last_record,
        })?;
        match event {
            IoEvent::StdoutRecord(bytes) => {
                trace_bytes = trace_bytes.saturating_add(bytes.len());
                if trace_bytes > limits.complete_trace_bytes() {
                    return Err(RequestFailure {
                        kind: HarnessFailureKind::TraceTooLarge,
                        maybe_last_record,
                    });
                }
                jsonl.extend_from_slice(&bytes);
                let record =
                    decode_trace_record_jsonl(&bytes, limits).map_err(|error| RequestFailure {
                        kind: classify_trace_decode(&error),
                        maybe_last_record,
                    })?;
                let last = match &record {
                    TraceRecord::Begin(_) if stream_state == 0 => {
                        stream_state = 1;
                        LastValidRecord::TraceBegin
                    }
                    TraceRecord::Checkpoint(_) if stream_state == 1 => LastValidRecord::Checkpoint,
                    TraceRecord::End(_) if stream_state == 1 => {
                        stream_state = 2;
                        LastValidRecord::TraceEnd
                    }
                    _ => {
                        return Err(RequestFailure {
                            kind: HarnessFailureKind::SequenceViolation,
                            maybe_last_record,
                        });
                    }
                };
                maybe_last_record = Some(last);
                records.push(record);
                if stream_state == 2 {
                    let trace = TraceValidator::validate(
                        request,
                        &ready.identity,
                        u64::try_from(ready.requests.saturating_add(1)).map_err(|_| {
                            RequestFailure {
                                kind: HarnessFailureKind::AdapterResetFailure,
                                maybe_last_record,
                            }
                        })?,
                        records,
                        limits,
                    )
                    .map_err(|error| RequestFailure {
                        kind: error.kind(),
                        maybe_last_record,
                    })?;
                    ready.output_boundary = reconcile_request_output(
                        &ready.io.workers,
                        deadline,
                        baseline,
                        limits,
                        maybe_last_record,
                    )?;
                    return Ok(CapturedOracleTrace {
                        trace,
                        jsonl: jsonl.into_boxed_slice(),
                    });
                }
            }
            IoEvent::OutputProgress(total) => {
                enforce_total_output(total, baseline, limits).map_err(|kind| RequestFailure {
                    kind,
                    maybe_last_record,
                })?;
            }
            IoEvent::SanitizerDetected => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::SanitizerReport,
                    maybe_last_record,
                });
            }
            IoEvent::StdoutRecordTooLarge => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::RecordTooLarge,
                    maybe_last_record,
                });
            }
            IoEvent::StdoutPartial => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::PartialRecord,
                    maybe_last_record,
                });
            }
            IoEvent::StdoutEof | IoEvent::ReadFailure => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::UnexpectedEof,
                    maybe_last_record,
                });
            }
        }
    }
}
