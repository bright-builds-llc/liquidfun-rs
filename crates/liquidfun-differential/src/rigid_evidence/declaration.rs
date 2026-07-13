//! Result-to-request declaration validation before physics comparison.

use std::fmt::Debug;

use liquidfun_test_protocol::{
    RigidWorldRequestRecord, RigidWorldResultRecord, Sha256Hex,
    rigid_world_checkpoint_live_identities, validate_rigid_world_result_against_request,
};

use super::{
    Location, RigidComparisonFailure, RigidDeclarationReport, RigidEngineSide, RigidHarnessReport,
    declaration_signature,
};

#[allow(
    clippy::too_many_lines,
    reason = "declaration validation follows the closed protocol hierarchy in fail-fast order"
)]
pub(super) fn validate_rigid_declarations_with_identity(
    request: &RigidWorldRequestRecord,
    result: &RigidWorldResultRecord,
    profile_sha256: &Sha256Hex,
    side: RigidEngineSide,
) -> Result<(), RigidComparisonFailure> {
    if result.request_id() != request.request_id() {
        return declaration_root(
            request,
            profile_sha256,
            side,
            "rigid_world.result.request_id",
            request.request_id().as_str(),
            result.request_id().as_str(),
        );
    }
    if result.scenario_id() != request.scenario().scenario_id() {
        return declaration_root(
            request,
            profile_sha256,
            side,
            "rigid_world.result.scenario_id",
            request.scenario().scenario_id().as_str(),
            result.scenario_id().as_str(),
        );
    }
    if result.timelines().len() != request.scenario().timelines().len() {
        return declaration_root(
            request,
            profile_sha256,
            side,
            "rigid_world.timelines.order",
            request.scenario().timelines().len(),
            result.timelines().len(),
        );
    }

    for (timeline_index, (declared, actual)) in request
        .scenario()
        .timelines()
        .iter()
        .zip(result.timelines())
        .enumerate()
    {
        if declared.witness_family() != actual.witness_family {
            return declaration(
                request,
                profile_sha256,
                side,
                timeline_index,
                0,
                "rigid_world.timeline.witness_family",
                declared.witness_family(),
                actual.witness_family,
            );
        }
        if declared.checkpoints().len() != actual.checkpoints.len() {
            return declaration(
                request,
                profile_sha256,
                side,
                timeline_index,
                first_missing_index(declared.checkpoints().len(), actual.checkpoints.len()),
                "rigid_world.checkpoints.order",
                declared.checkpoints().len(),
                actual.checkpoints.len(),
            );
        }
        for (checkpoint_index, (expected, actual)) in declared
            .checkpoints()
            .iter()
            .zip(actual.checkpoints.iter())
            .enumerate()
        {
            if expected.checkpoint_id() != &actual.checkpoint_id {
                return declaration(
                    request,
                    profile_sha256,
                    side,
                    timeline_index,
                    checkpoint_index,
                    "rigid_world.checkpoint.id",
                    expected.checkpoint_id(),
                    &actual.checkpoint_id,
                );
            }
            if expected.phase() != actual.phase.as_ref() {
                return declaration(
                    request,
                    profile_sha256,
                    side,
                    timeline_index,
                    checkpoint_index,
                    "rigid_world.checkpoint.phase",
                    expected.phase(),
                    actual.phase.as_ref(),
                );
            }
            if expected.counts() != actual.counts {
                return declaration(
                    request,
                    profile_sha256,
                    side,
                    timeline_index,
                    checkpoint_index,
                    "rigid_world.checkpoint.counts",
                    expected.counts(),
                    actual.counts,
                );
            }
            let body_ids = actual
                .bodies
                .iter()
                .map(|body| &body.body_id)
                .collect::<Vec<_>>();
            let live_identities =
                rigid_world_checkpoint_live_identities(declared, checkpoint_index)
                    .expect("validated checkpoint action exists");
            if live_identities.body_ids() != body_ids {
                return declaration(
                    request,
                    profile_sha256,
                    side,
                    timeline_index,
                    checkpoint_index,
                    "rigid_world.checkpoint.bodies.declaration_order",
                    live_identities.body_ids(),
                    body_ids.as_slice(),
                );
            }
            let fixture_ids = actual
                .fixtures
                .iter()
                .map(|fixture| &fixture.fixture_id)
                .collect::<Vec<_>>();
            if live_identities.fixture_ids() != fixture_ids {
                return declaration(
                    request,
                    profile_sha256,
                    side,
                    timeline_index,
                    checkpoint_index,
                    "rigid_world.checkpoint.fixtures.declaration_order",
                    live_identities.fixture_ids(),
                    fixture_ids.as_slice(),
                );
            }
        }
    }

    validate_rigid_world_result_against_request(request, result).map_err(|error| {
        RigidComparisonFailure::Harness(RigidHarnessReport {
            reason: "validate_result_declaration".into(),
            expected: "request declarations".into(),
            actual: error.to_string().into_boxed_str(),
        })
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "a declaration report requires complete stable evidence coordinates"
)]
fn declaration<T: Debug>(
    request: &RigidWorldRequestRecord,
    profile_sha256: &Sha256Hex,
    side: RigidEngineSide,
    timeline_index: usize,
    checkpoint_index: usize,
    path: &'static str,
    expected: T,
    actual: T,
) -> Result<(), RigidComparisonFailure> {
    let timeline = &request.scenario().timelines()[timeline_index];
    let bounded_index = checkpoint_index.min(timeline.checkpoints().len().saturating_sub(1));
    let location = Location {
        timeline_index,
        checkpoint_index: bounded_index,
    };
    Err(RigidComparisonFailure::Declaration(Box::new(
        RigidDeclarationReport {
            signature: declaration_signature(
                request,
                profile_sha256,
                location,
                path,
                &format!("{expected:?}"),
                &format!("{actual:?}"),
            ),
            engine_side: side,
            expected: format!("{expected:?}").into_boxed_str(),
            actual: format!("{actual:?}").into_boxed_str(),
        },
    )))
}

fn declaration_root<T: Debug>(
    request: &RigidWorldRequestRecord,
    profile_sha256: &Sha256Hex,
    side: RigidEngineSide,
    path: &'static str,
    expected: T,
    actual: T,
) -> Result<(), RigidComparisonFailure> {
    declaration(request, profile_sha256, side, 0, 0, path, expected, actual)
}

fn first_missing_index(expected: usize, actual: usize) -> usize {
    expected.min(actual)
}
