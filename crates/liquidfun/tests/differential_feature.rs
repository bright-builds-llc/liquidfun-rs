//! Feature-gated collision diagnostic contract tests.

#![cfg(feature = "differential-internals")]

use liquidfun::collision::differential::{
    ClipDiagnosticInput, DiagnosticFeature, DistanceCacheReplayOutcome, DistanceCacheSeed,
    DistanceCacheSeedPair, DistanceCacheSeedRejection, DistanceCacheSeedReset,
    DistanceProxyFingerprint, DistanceProxyKind, DistanceProxyVertexBits, GjkTermination,
    PairDiagnosticRecord, TimeOfImpactDiagnosticRecord, clip_segment_diagnostic,
    distance_diagnostic, distance_proxy_fingerprint, pair_diagnostic, replay_distance_cache,
    time_of_impact_diagnostic,
};
use liquidfun::collision::{
    ChainShape, CircleShape, ContactFeatureId, DistanceCache, DynamicTree, EdgeShape, FeatureKind,
    FilterData, PairOrientation, PolygonShape, Shape, TimeOfImpactState,
};
use liquidfun::math::{Transform, Vec2};

fn assert_owned<T: Send + Sync + 'static>(value: T) -> T {
    value
}

#[test]
fn root_exports_are_curated_and_constructible() {
    // Arrange / Act
    let circle = CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid");
    let edge = EdgeShape::new(Vec2::ZERO, Vec2::new(1.0, 0.0)).expect("edge should be valid");
    let polygon = PolygonShape::box_shape(1.0, 1.0).expect("polygon should be valid");
    let chain = ChainShape::open(&[Vec2::ZERO, Vec2::new(1.0, 0.0)], None, None)
        .expect("chain should be valid");
    let shape = Shape::from(circle);
    let cache = DistanceCache::empty();
    let tree = DynamicTree::<u32>::new().expect("tree identity should remain available");
    let filter = FilterData::default();

    // Assert
    assert_eq!(shape.child_count(), 1);
    assert_eq!(edge.child_count(), 1);
    assert_eq!(polygon.child_count(), 1);
    assert_eq!(chain.child_count(), 1);
    assert_eq!(cache.snapshot().count(), 0);
    assert_eq!(tree.proxy_count(), 0);
    assert!(filter.should_collide(filter));
}

#[test]
fn diagnostics_are_typed_owned_and_bounded() {
    // Arrange
    let feature = DiagnosticFeature::new(ContactFeatureId::new(
        1,
        2,
        FeatureKind::Face,
        FeatureKind::Vertex,
    ));
    let clip_input = ClipDiagnosticInput::new(
        [
            (Vec2::new(-1.0, 0.0), feature),
            (Vec2::new(1.0, 0.0), feature),
        ],
        Vec2::new(1.0, 0.0),
        0.0,
        3,
    )
    .expect("bounded clip input should be valid");

    // Act
    let clipped = assert_owned(clip_segment_diagnostic(clip_input));
    let distance = assert_owned(distance_diagnostic(&DistanceCache::empty(), 0));
    let pair: PairDiagnosticRecord =
        assert_owned(pair_diagnostic(PairOrientation::Primary, None, None));
    let toi: TimeOfImpactDiagnosticRecord =
        assert_owned(time_of_impact_diagnostic(TimeOfImpactState::Separated, 1.0));

    // Assert
    assert_eq!(clipped.points().len(), 2);
    assert_eq!(distance.termination(), GjkTermination::NearZeroDirection);
    assert_eq!(pair.orientation(), PairOrientation::Primary);
    assert_eq!(toi.state(), TimeOfImpactState::Separated);
}

fn circle(center: Vec2) -> Shape {
    CircleShape::new(center, 1.0)
        .expect("circle should be valid")
        .into()
}

fn fingerprint_with_radius(
    fingerprint: &DistanceProxyFingerprint,
    radius_bits: u32,
) -> DistanceProxyFingerprint {
    DistanceProxyFingerprint::new(
        fingerprint.kind(),
        fingerprint.child_index(),
        radius_bits,
        fingerprint.vertices().to_vec(),
    )
    .expect("bounded fingerprint should remain valid")
}

#[test]
fn cache_replay_count_one_is_used_without_metric_flush() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = circle(Vec2::new(5.0, 0.0));
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let seed = DistanceCacheSeed::new(
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed"),
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed"),
        vec![DistanceCacheSeedPair::new(0, 0)],
        f32::MAX,
    )
    .expect("bounded seed should be valid");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    let DistanceCacheReplayOutcome::Used { result } = outcome else {
        panic!("one-point source cache must be used");
    };
    assert_eq!(result.distance().to_bits(), 5.0_f32.to_bits());
}

#[test]
fn cache_replay_rejection_precedence_is_fail_closed() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = circle(Vec2::new(5.0, 0.0));
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let fingerprint_a =
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed");
    let fingerprint_b =
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed");
    let mismatched_a = fingerprint_with_radius(&fingerprint_a, 99.0_f32.to_bits());
    let mismatched_b = fingerprint_with_radius(&fingerprint_b, 98.0_f32.to_bits());
    let seed = DistanceCacheSeed::new(mismatched_a, mismatched_b, Vec::new(), f32::NAN)
        .expect("bounded invalid seed should be representable");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    assert!(matches!(
        outcome,
        DistanceCacheReplayOutcome::Rejected {
            reason: DistanceCacheSeedRejection::ProxyAFingerprintMismatch,
        }
    ));
}

#[test]
fn cache_replay_multi_point_reset_uses_source_ratio_precedence() {
    // Arrange
    let shape_a: Shape = EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0))
        .expect("edge should be valid")
        .into();
    let shape_b = shape_a.clone();
    let child_a = shape_a.child_index(0).expect("edge child should exist");
    let child_b = shape_b.child_index(0).expect("edge child should exist");
    let seed = DistanceCacheSeed::new(
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed"),
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed"),
        vec![
            DistanceCacheSeedPair::new(0, 0),
            DistanceCacheSeedPair::new(1, 1),
        ],
        1.0,
    )
    .expect("bounded seed should be valid");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    assert!(matches!(
        outcome,
        DistanceCacheReplayOutcome::Reset {
            reason: DistanceCacheSeedReset::MetricRatio,
            ..
        }
    ));
}

#[test]
fn cache_replay_rejects_duplicate_pair_before_non_finite_metric() {
    // Arrange
    let shape_a: Shape = EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0))
        .expect("edge should be valid")
        .into();
    let shape_b = shape_a.clone();
    let child_a = shape_a.child_index(0).expect("edge child should exist");
    let child_b = shape_b.child_index(0).expect("edge child should exist");
    let seed = DistanceCacheSeed::new(
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed"),
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed"),
        vec![
            DistanceCacheSeedPair::new(0, 0),
            DistanceCacheSeedPair::new(0, 0),
        ],
        f32::NAN,
    )
    .expect("bounded invalid seed should be representable");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    assert!(matches!(
        outcome,
        DistanceCacheReplayOutcome::Rejected {
            reason: DistanceCacheSeedRejection::DuplicateSupportPair,
        }
    ));
}

#[test]
fn cache_replay_proxy_b_mismatch_precedes_invalid_count() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = circle(Vec2::new(5.0, 0.0));
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let fingerprint_a =
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed");
    let fingerprint_b =
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed");
    let seed = DistanceCacheSeed::new(
        fingerprint_a,
        fingerprint_with_radius(&fingerprint_b, 99.0_f32.to_bits()),
        Vec::new(),
        0.0,
    )
    .expect("bounded invalid seed should be representable");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    assert!(matches!(
        outcome,
        DistanceCacheReplayOutcome::Rejected {
            reason: DistanceCacheSeedRejection::ProxyBFingerprintMismatch,
        }
    ));
}

#[test]
fn cache_replay_invalid_count_precedes_bad_indices() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = circle(Vec2::new(5.0, 0.0));
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let seed = DistanceCacheSeed::new(
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed"),
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed"),
        vec![
            DistanceCacheSeedPair::new(9, 9),
            DistanceCacheSeedPair::new(8, 8),
            DistanceCacheSeedPair::new(7, 7),
            DistanceCacheSeedPair::new(6, 6),
        ],
        0.0,
    )
    .expect("bounded invalid seed should be representable");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    assert!(matches!(
        outcome,
        DistanceCacheReplayOutcome::Rejected {
            reason: DistanceCacheSeedRejection::SupportCountOutOfRange,
        }
    ));
}

#[test]
fn cache_replay_index_a_precedes_index_b_in_each_pair() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = circle(Vec2::new(5.0, 0.0));
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let seed = DistanceCacheSeed::new(
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed"),
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed"),
        vec![DistanceCacheSeedPair::new(1, 1)],
        0.0,
    )
    .expect("bounded invalid seed should be representable");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    assert!(matches!(
        outcome,
        DistanceCacheReplayOutcome::Rejected {
            reason: DistanceCacheSeedRejection::SupportIndexAOutOfRange,
        }
    ));
}

#[test]
fn cache_replay_reports_index_b_after_valid_index_a() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = circle(Vec2::new(5.0, 0.0));
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let seed = DistanceCacheSeed::new(
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed"),
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed"),
        vec![DistanceCacheSeedPair::new(0, 1)],
        0.0,
    )
    .expect("bounded invalid seed should be representable");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    assert!(matches!(
        outcome,
        DistanceCacheReplayOutcome::Rejected {
            reason: DistanceCacheSeedRejection::SupportIndexBOutOfRange,
        }
    ));
}

#[test]
fn cache_replay_non_finite_metric_precedes_count_one_use() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = circle(Vec2::new(5.0, 0.0));
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let seed = DistanceCacheSeed::new(
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed"),
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed"),
        vec![DistanceCacheSeedPair::new(0, 0)],
        f32::INFINITY,
    )
    .expect("bounded invalid seed should be representable");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    assert!(matches!(
        outcome,
        DistanceCacheReplayOutcome::Rejected {
            reason: DistanceCacheSeedRejection::NonFiniteMetric,
        }
    ));
}

#[test]
fn cache_replay_multi_point_epsilon_reset_follows_ratio_check() {
    // Arrange
    let shape_a: Shape = EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0))
        .expect("edge should be valid")
        .into();
    let shape_b = shape_a.clone();
    let child_a = shape_a.child_index(0).expect("edge child should exist");
    let child_b = shape_b.child_index(0).expect("edge child should exist");
    let seed = DistanceCacheSeed::new(
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed"),
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed"),
        vec![
            DistanceCacheSeedPair::new(0, 0),
            DistanceCacheSeedPair::new(1, 1),
        ],
        0.0,
    )
    .expect("bounded seed should be valid");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    assert!(matches!(
        outcome,
        DistanceCacheReplayOutcome::Reset {
            reason: DistanceCacheSeedReset::MetricTooSmall,
            ..
        }
    ));
}

#[test]
fn cache_replay_matching_multi_point_seed_is_used() {
    // Arrange
    let shape_a: Shape = EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0))
        .expect("edge should be valid")
        .into();
    let shape_b = shape_a.clone();
    let child_a = shape_a.child_index(0).expect("edge child should exist");
    let child_b = shape_b.child_index(0).expect("edge child should exist");
    let seed = DistanceCacheSeed::new(
        distance_proxy_fingerprint(&shape_a, child_a).expect("fingerprint should succeed"),
        distance_proxy_fingerprint(&shape_b, child_b).expect("fingerprint should succeed"),
        vec![
            DistanceCacheSeedPair::new(0, 1),
            DistanceCacheSeedPair::new(1, 0),
        ],
        4.0,
    )
    .expect("bounded seed should be valid");

    // Act
    let outcome = replay_distance_cache(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        seed,
    )
    .expect("checked replay should execute");

    // Assert
    assert!(matches!(outcome, DistanceCacheReplayOutcome::Used { .. }));
}

#[test]
fn cache_seed_types_are_owned_and_semantic() {
    // Arrange
    let fingerprint = DistanceProxyFingerprint::new(
        DistanceProxyKind::Circle,
        0,
        1.0_f32.to_bits(),
        vec![DistanceProxyVertexBits::new(
            0.0_f32.to_bits(),
            0.0_f32.to_bits(),
        )],
    )
    .expect("bounded fingerprint should be valid");

    // Act
    let seed = assert_owned(
        DistanceCacheSeed::new(
            fingerprint.clone(),
            fingerprint,
            vec![DistanceCacheSeedPair::new(0, 0)],
            0.0,
        )
        .expect("bounded seed should be valid"),
    );

    // Assert
    assert_eq!(seed.support_pairs().len(), 1);
}
