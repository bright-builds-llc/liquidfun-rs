//! Strict catalog request/response sequence validation.

use std::{io::Write, time::Instant};

use liquidfun_test_protocol::{
    BuildEvidenceTier, CanonicalCheckpoint, CatalogRunRequest, CheckpointPosition, EvidenceTier,
    HarnessLimits, ProtocolVersion, RequestId, Sha256Hex, decode_canonical_checkpoint_jsonl,
    encode_catalog_run_request_jsonl,
};
use serde::Deserialize;

use crate::{CatalogFailureKind, CatalogRunCapture};

use super::super::{
    IoEvent, ReadyChild, enforce_total_output, receive_with_output_precedence,
    reconcile_request_output,
};

pub(super) fn run_catalog_request(
    ready: &mut ReadyChild,
    request: &CatalogRunRequest,
    limits: &HarnessLimits,
) -> Result<(CatalogRunCapture, Vec<u8>, CatalogEnd), CatalogFailureKind> {
    let baseline = ready.output_boundary;
    ready.last_request_baseline = baseline;
    let request_bytes = encode_catalog_run_request_jsonl(request, limits)
        .map_err(|_error| CatalogFailureKind::Protocol)?;
    let stdin = ready
        .io
        .maybe_stdin
        .as_mut()
        .ok_or(CatalogFailureKind::ChildProcess)?;
    stdin
        .write_all(&request_bytes)
        .and_then(|()| stdin.flush())
        .map_err(|_error| CatalogFailureKind::ChildProcess)?;
    let deadline = Instant::now() + limits.request_timeout();
    let mut checkpoints = Vec::with_capacity(request.resolved().checkpoints().len());
    let mut response_bytes = Vec::from(ready.handshake_jsonl.as_ref());
    let mut response_total = 0_usize;
    loop {
        let event = receive_with_output_precedence(
            &ready.io.workers,
            deadline,
            liquidfun_test_protocol::HarnessFailureKind::RequestTimeout,
            baseline,
            limits,
        )
        .map_err(|kind| map_harness_kind(kind, CatalogFailureKind::Timeout))?;
        match event {
            IoEvent::StdoutRecord(bytes) => {
                response_total = response_total.saturating_add(bytes.len());
                if response_total > limits.complete_trace_bytes() {
                    return Err(CatalogFailureKind::ResourceLimit);
                }
                response_bytes.extend_from_slice(&bytes);
                match record_kind(&bytes)? {
                    CatalogRecordKind::Checkpoint => {
                        let checkpoint = decode_canonical_checkpoint_jsonl(&bytes, limits)
                            .map_err(|_error| CatalogFailureKind::MalformedRecord)?;
                        validate_checkpoint(request, checkpoints.len(), &checkpoint)?;
                        checkpoints.push(checkpoint);
                    }
                    CatalogRecordKind::End => {
                        let end = decode_end(&bytes, limits)?;
                        validate_end(request, ready.requests, checkpoints.len(), &end)?;
                        ready.output_boundary = reconcile_request_output(
                            &ready.io.workers,
                            deadline,
                            baseline,
                            limits,
                            None,
                        )
                        .map_err(|failure| {
                            map_harness_kind(failure.kind, CatalogFailureKind::MalformedRecord)
                        })?;
                        let capture = CatalogRunCapture::from_parts(request, checkpoints)
                            .map_err(|error| error.kind())?;
                        return Ok((capture, response_bytes, end));
                    }
                }
            }
            IoEvent::OutputProgress(total) => enforce_total_output(total, baseline, limits)
                .map_err(|_kind| CatalogFailureKind::ResourceLimit)?,
            IoEvent::SanitizerDetected => return Err(CatalogFailureKind::ChildProcess),
            IoEvent::StdoutRecordTooLarge => return Err(CatalogFailureKind::ResourceLimit),
            IoEvent::StdoutPartial | IoEvent::StdoutEof | IoEvent::ReadFailure => {
                return Err(CatalogFailureKind::MalformedRecord);
            }
        }
    }
}

fn validate_checkpoint(
    request: &CatalogRunRequest,
    index: usize,
    checkpoint: &CanonicalCheckpoint,
) -> Result<(), CatalogFailureKind> {
    let declaration = request
        .resolved()
        .checkpoints()
        .get(index)
        .ok_or(CatalogFailureKind::MalformedRecord)?;
    if checkpoint.request_id() != request.request_id()
        || checkpoint.resolved_sha256() != request.resolved().identity().content_sha256()
        || checkpoint.checkpoint_id() != declaration.checkpoint_id()
        || checkpoint.position()
            != &(CheckpointPosition::LogicalStep {
                ordinal: declaration.logical_step(),
            })
    {
        return Err(CatalogFailureKind::Protocol);
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CatalogEnd {
    protocol_version: ProtocolVersion,
    record_kind: EndKind,
    request_id: RequestId,
    resolved_sha256: Sha256Hex,
    checkpoint_count: u32,
    pub(super) reset_epoch: u64,
    pub(super) reset_verified: bool,
}

#[derive(Deserialize)]
enum EndKind {
    #[serde(rename = "catalog_run_end")]
    CatalogRunEnd,
}

fn decode_end(bytes: &[u8], limits: &HarnessLimits) -> Result<CatalogEnd, CatalogFailureKind> {
    if bytes.len() > limits.output_record_bytes() {
        return Err(CatalogFailureKind::ResourceLimit);
    }
    serde_json::from_slice(bytes).map_err(|_error| CatalogFailureKind::MalformedRecord)
}

fn validate_end(
    request: &CatalogRunRequest,
    completed_requests: usize,
    checkpoint_count: usize,
    end: &CatalogEnd,
) -> Result<(), CatalogFailureKind> {
    let expected_epoch = u64::try_from(completed_requests.saturating_add(1))
        .map_err(|_error| CatalogFailureKind::ResetFailure)?;
    if end.protocol_version != ProtocolVersion::CURRENT
        || !matches!(end.record_kind, EndKind::CatalogRunEnd)
        || end.request_id != *request.request_id()
        || end.resolved_sha256 != *request.resolved().identity().content_sha256()
        || usize::try_from(end.checkpoint_count).ok() != Some(checkpoint_count)
        || checkpoint_count != request.resolved().checkpoints().len()
        || end.reset_epoch != expected_epoch
        || !end.reset_verified
    {
        return Err(CatalogFailureKind::ResetFailure);
    }
    Ok(())
}

enum CatalogRecordKind {
    Checkpoint,
    End,
}

fn record_kind(bytes: &[u8]) -> Result<CatalogRecordKind, CatalogFailureKind> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_error| CatalogFailureKind::MalformedRecord)?;
    match value.get("record_kind").and_then(serde_json::Value::as_str) {
        Some("canonical_checkpoint") => Ok(CatalogRecordKind::Checkpoint),
        Some("catalog_run_end") => Ok(CatalogRecordKind::End),
        _ => Err(CatalogFailureKind::MalformedRecord),
    }
}

pub(super) fn tier_satisfies(actual: BuildEvidenceTier, required: EvidenceTier) -> bool {
    matches!(
        (actual, required),
        (BuildEvidenceTier::D1Canonical, _)
            | (
                BuildEvidenceTier::D2Supported,
                EvidenceTier::D2Supported | EvidenceTier::D3Exploratory
            )
            | (
                BuildEvidenceTier::D3Exploratory,
                EvidenceTier::D3Exploratory
            )
    )
}

fn map_harness_kind(
    kind: liquidfun_test_protocol::HarnessFailureKind,
    fallback: CatalogFailureKind,
) -> CatalogFailureKind {
    use liquidfun_test_protocol::HarnessFailureKind;
    match kind {
        HarnessFailureKind::StartupTimeout | HarnessFailureKind::RequestTimeout => {
            CatalogFailureKind::Timeout
        }
        HarnessFailureKind::RecordTooLarge
        | HarnessFailureKind::TraceTooLarge
        | HarnessFailureKind::TotalOutputExceeded => CatalogFailureKind::ResourceLimit,
        HarnessFailureKind::AdapterResetFailure => CatalogFailureKind::ResetFailure,
        HarnessFailureKind::WrongProvenance | HarnessFailureKind::TraceIdentityMismatch => {
            CatalogFailureKind::Provenance
        }
        _ => fallback,
    }
}
