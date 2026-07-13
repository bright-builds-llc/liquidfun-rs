//! Inherited Phase 6 checkpoint comparison.

use std::fmt::Debug;

use liquidfun_test_protocol::{
    FieldComparison, FloatBits, Phase6PolicyProfile, RigidWorldCheckpointResult,
    RigidWorldRequestRecord, RigidWorldResultRecord,
};

use crate::float_values_match_with_policy;

use super::{Location, RigidMismatchKind, RigidMismatchReport, mismatch};

pub(super) fn first_rigid_divergence(
    request: &RigidWorldRequestRecord,
    expected: &RigidWorldResultRecord,
    actual: &RigidWorldResultRecord,
    profile: &Phase6PolicyProfile,
) -> Option<RigidMismatchReport> {
    for (timeline_index, (expected_timeline, actual_timeline)) in expected
        .timelines()
        .iter()
        .zip(actual.timelines())
        .enumerate()
    {
        for (checkpoint_index, (expected_checkpoint, actual_checkpoint)) in expected_timeline
            .checkpoints
            .iter()
            .zip(actual_timeline.checkpoints.iter())
            .enumerate()
        {
            let location = Location {
                timeline_index,
                checkpoint_index,
            };
            if let Some(report) = compare_checkpoint(
                request,
                profile,
                location,
                expected_checkpoint,
                actual_checkpoint,
            ) {
                return Some(report);
            }
        }
    }
    None
}

macro_rules! exact_field {
    ($request:expr, $profile:expr, $location:expr, $path:expr, $left:expr, $right:expr) => {
        if let Some(report) = exact(
            $request,
            $profile,
            $location,
            $path,
            RigidMismatchKind::Exact,
            &$left,
            &$right,
        ) {
            return Some(report);
        }
    };
    ($request:expr, $profile:expr, $location:expr, $path:expr, $left:expr, $right:expr, $kind:expr) => {
        if let Some(report) = exact($request, $profile, $location, $path, $kind, &$left, &$right) {
            return Some(report);
        }
    };
}

macro_rules! float_field {
    ($request:expr, $profile:expr, $location:expr, $path:expr, $left:expr, $right:expr) => {
        if let Some(report) = float($request, $profile, $location, $path, $left, $right) {
            return Some(report);
        }
    };
}

#[allow(
    clippy::too_many_lines,
    reason = "the inherited comparator preserves the closed Phase 6 first-divergence order"
)]
pub(super) fn compare_checkpoint(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    location: Location,
    expected: &RigidWorldCheckpointResult,
    actual: &RigidWorldCheckpointResult,
) -> Option<RigidMismatchReport> {
    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.counts",
        expected.counts,
        actual.counts
    );
    let expected_body_ids = expected
        .bodies
        .iter()
        .map(|body| &body.body_id)
        .collect::<Vec<_>>();
    let actual_body_ids = actual
        .bodies
        .iter()
        .map(|body| &body.body_id)
        .collect::<Vec<_>>();
    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.bodies.declaration_order",
        expected_body_ids,
        actual_body_ids,
        RigidMismatchKind::Order
    );
    for (left, right) in expected.bodies.iter().zip(actual.bodies.iter()) {
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.body.id",
            left.body_id,
            right.body_id
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.body.kind",
            left.body_kind,
            right.body_kind
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.body.active",
            left.active,
            right.active
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.transform.position.x",
            left.transform.position.x_bits,
            right.transform.position.x_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.transform.position.y",
            left.transform.position.y_bits,
            right.transform.position.y_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.transform.angle",
            left.transform.angle_bits,
            right.transform.angle_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.linear_velocity.x",
            left.linear_velocity.x_bits,
            right.linear_velocity.x_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.linear_velocity.y",
            left.linear_velocity.y_bits,
            right.linear_velocity.y_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.angular_velocity",
            left.angular_velocity_bits,
            right.angular_velocity_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.mass",
            left.mass_bits,
            right.mass_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.local_center.x",
            left.local_center.x_bits,
            right.local_center.x_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.local_center.y",
            left.local_center.y_bits,
            right.local_center.y_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.inertia",
            left.inertia_bits,
            right.inertia_bits
        );
    }

    let expected_fixture_ids = expected
        .fixtures
        .iter()
        .map(|fixture| &fixture.fixture_id)
        .collect::<Vec<_>>();
    let actual_fixture_ids = actual
        .fixtures
        .iter()
        .map(|fixture| &fixture.fixture_id)
        .collect::<Vec<_>>();
    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.fixtures.declaration_order",
        expected_fixture_ids,
        actual_fixture_ids,
        RigidMismatchKind::Order
    );
    for (left, right) in expected.fixtures.iter().zip(actual.fixtures.iter()) {
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.id",
            left.fixture_id,
            right.fixture_id
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.owner_body_id",
            left.owner_body_id,
            right.owner_body_id
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.sensor",
            left.sensor,
            right.sensor
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.filter.category_bits",
            left.filter.category_bits(),
            right.filter.category_bits()
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.filter.mask_bits",
            left.filter.mask_bits(),
            right.filter.mask_bits()
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.filter.group_index",
            left.filter.group_index(),
            right.filter.group_index()
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.density",
            left.density_bits,
            right.density_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.friction",
            left.friction_bits,
            right.friction_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.restitution",
            left.restitution_bits,
            right.restitution_bits
        );
    }

    let expected_contacts = expected
        .contacts
        .iter()
        .map(|contact| &contact.identity)
        .collect::<Vec<_>>();
    let actual_contacts = actual
        .contacts
        .iter()
        .map(|contact| &contact.identity)
        .collect::<Vec<_>>();
    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.contacts.manager_order",
        expected_contacts,
        actual_contacts,
        RigidMismatchKind::Order
    );
    for (left, right) in expected.contacts.iter().zip(actual.contacts.iter()) {
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.contact.touching",
            left.touching,
            right.touching
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.contact.enabled",
            left.enabled,
            right.enabled
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.contact.sensor",
            left.sensor,
            right.sensor
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.contact.mixed_friction",
            left.mixed_friction_bits,
            right.mixed_friction_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.contact.mixed_restitution",
            left.mixed_restitution_bits,
            right.mixed_restitution_bits
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.contact.manifold.presence",
            left.maybe_manifold.is_some(),
            right.maybe_manifold.is_some()
        );
        if let (Some(left), Some(right)) = (&left.maybe_manifold, &right.maybe_manifold) {
            exact_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.kind",
                left.manifold_kind,
                right.manifold_kind
            );
            let expected_features = left
                .points
                .iter()
                .map(|point| point.feature)
                .collect::<Vec<_>>();
            let actual_features = right
                .points
                .iter()
                .map(|point| point.feature)
                .collect::<Vec<_>>();
            exact_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.points.order",
                expected_features,
                actual_features,
                RigidMismatchKind::Order
            );
            float_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.local_normal.x",
                left.local_normal.x_bits,
                right.local_normal.x_bits
            );
            float_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.local_normal.y",
                left.local_normal.y_bits,
                right.local_normal.y_bits
            );
            float_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.local_point.x",
                left.local_point.x_bits,
                right.local_point.x_bits
            );
            float_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.local_point.y",
                left.local_point.y_bits,
                right.local_point.y_bits
            );
            for (left, right) in left.points.iter().zip(right.points.iter()) {
                exact_field!(
                    request,
                    profile,
                    location,
                    "rigid_world.contact.manifold.point.feature",
                    left.feature,
                    right.feature
                );
                float_field!(
                    request,
                    profile,
                    location,
                    "rigid_world.contact.manifold.point.position.x",
                    left.point.x_bits,
                    right.point.x_bits
                );
                float_field!(
                    request,
                    profile,
                    location,
                    "rigid_world.contact.manifold.point.position.y",
                    left.point.y_bits,
                    right.point.y_bits
                );
                float_field!(
                    request,
                    profile,
                    location,
                    "rigid_world.contact.manifold.point.normal_impulse",
                    left.normal_impulse_bits,
                    right.normal_impulse_bits
                );
                float_field!(
                    request,
                    profile,
                    location,
                    "rigid_world.contact.manifold.point.tangent_impulse",
                    left.tangent_impulse_bits,
                    right.tangent_impulse_bits
                );
            }
        }
    }

    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.events.report_order",
        expected.events,
        actual.events,
        RigidMismatchKind::Order
    );
    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.destructions.report_order",
        expected.destructions,
        actual.destructions,
        RigidMismatchKind::Order
    );
    None
}

fn exact<T: Debug + PartialEq>(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    location: Location,
    path: &'static str,
    kind: RigidMismatchKind,
    expected: &T,
    actual: &T,
) -> Option<RigidMismatchReport> {
    let policy = profile
        .field(path)
        .expect("validated Phase 6 profile contains every exact path");
    debug_assert_eq!(policy.comparison(), FieldComparison::ExactDiscrete);
    (expected != actual).then(|| {
        mismatch(
            request,
            profile,
            location,
            path,
            kind,
            format!("{expected:?}"),
            format!("{actual:?}"),
            None,
        )
    })
}

fn float(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Option<RigidMismatchReport> {
    let policy = profile
        .field(path)
        .expect("validated Phase 6 profile contains every float path");
    (!float_values_match_with_policy(expected, actual, policy)).then(|| {
        mismatch(
            request,
            profile,
            location,
            path,
            RigidMismatchKind::Numeric,
            format!("0x{:08x}", expected.bits()),
            format!("0x{:08x}", actual.bits()),
            Some((expected, actual)),
        )
    })
}
