//! Feature-gated collision diagnostic contract tests.

#![cfg(feature = "differential-internals")]

use liquidfun::collision::differential::{
    ClipDiagnosticInput, DiagnosticFeature, GjkTermination, PairDiagnosticRecord,
    TimeOfImpactDiagnosticRecord, clip_segment_diagnostic, distance_diagnostic, pair_diagnostic,
    time_of_impact_diagnostic,
};
use liquidfun::collision::{
    ChainShape, CircleShape, ContactFeatureId, DistanceCache, DynamicTree, EdgeShape, FeatureKind,
    FilterData, PairOrientation, PolygonShape, Shape, TimeOfImpactState,
};
use liquidfun::math::Vec2;

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
