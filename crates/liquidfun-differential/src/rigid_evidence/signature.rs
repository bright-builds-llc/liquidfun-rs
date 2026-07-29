//! Stable first-divergence signature construction.

use liquidfun_test_protocol::{FloatBits, RigidWorldRequestRecord, Sha256Hex};
use sha2::{Digest, Sha256};

use super::{EvidenceContext, Location, RigidFailureSignature, RigidMismatchKind};

#[allow(
    clippy::too_many_arguments,
    reason = "the digest binds every replay and minimization classification field"
)]
pub(super) fn build_signature(
    request: &RigidWorldRequestRecord,
    profile_sha256: &Sha256Hex,
    context: EvidenceContext<'_>,
    path: &str,
    kind: RigidMismatchKind,
    expected: &str,
    actual: &str,
    maybe_bits: Option<(FloatBits, FloatBits)>,
) -> RigidFailureSignature {
    let timeline = &request.scenario().timelines()[context.location.timeline_index];
    let checkpoint = &timeline.checkpoints()[context.location.checkpoint_index];
    let action_id = context
        .maybe_action_id
        .unwrap_or_else(|| checkpoint.after_action_id().as_str());
    let stage = context.maybe_stage.unwrap_or_else(|| checkpoint.phase());
    let input = format!(
        "{}\0{:?}\0{}\0{}\0{}\0{:?}\0{}\0{:?}\0{}\0{}\0{}\0{:?}\0{:?}\0{:?}",
        request.request_id().as_str(),
        timeline.witness_family(),
        action_id,
        checkpoint.checkpoint_id().as_str(),
        path,
        kind,
        profile_sha256.as_str(),
        context.maybe_entity,
        stage,
        expected,
        actual,
        maybe_bits.map(|bits| bits.0),
        maybe_bits.map(|bits| bits.1),
        context.maybe_completion_context,
    );
    RigidFailureSignature {
        signature_sha256: Sha256Hex::from_digest(Sha256::digest(input.as_bytes()).into()),
        witness_family: timeline.witness_family(),
        action_id: action_id.into(),
        checkpoint_id: checkpoint.checkpoint_id().as_str().into(),
        semantic_path: path.into(),
        kind,
        stage: stage.into(),
        maybe_entity: context.maybe_entity.map(Into::into),
        expected: expected.into(),
        actual: actual.into(),
        maybe_expected_bits: maybe_bits.map(|bits| bits.0),
        maybe_actual_bits: maybe_bits.map(|bits| bits.1),
        profile_sha256: profile_sha256.clone(),
        maybe_completion_context: context.maybe_completion_context,
    }
}

pub(super) fn declaration_signature(
    request: &RigidWorldRequestRecord,
    profile_sha256: &Sha256Hex,
    location: Location,
    path: &str,
    expected: &str,
    actual: &str,
) -> RigidFailureSignature {
    build_signature(
        request,
        profile_sha256,
        EvidenceContext::checkpoint(location),
        path,
        RigidMismatchKind::Exact,
        expected,
        actual,
        None,
    )
}
