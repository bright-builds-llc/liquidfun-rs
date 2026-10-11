//! Bit-identity witnesses for the chain child edge built once per child.

use crate::collision::{ChainShape, PolygonShape};
use crate::identity::{HandleIdentity, Identity};
use crate::particle::storage::ParticleStorage;

use super::tests::{legacy, storage};
use super::*;

fn rotated_transform() -> Transform {
    Transform::from_position_angle(Vec2::new(0.05, -0.03), 0.3)
}

fn rotated_positions(local: &[Vec2]) -> Vec<Vec2> {
    local
        .iter()
        .map(|point| rotated_transform().apply(*point))
        .collect()
}

fn single_source(storage: &ParticleStorage, shape: Shape) -> Vec<FixtureContactSource> {
    let world = storage.system().identity().world();
    vec![FixtureContactSource {
        body: BodyId::from_identity(Identity::new(world, 1, 0)),
        fixture: FixtureId::from_identity(Identity::new(world, 2, 0)),
        shape,
        transform: rotated_transform(),
        center: Vec2::new(0.15, 0.1),
        inverse_mass: 0.25,
        inverse_inertia: 0.2,
    }]
}

fn assert_contact_bits_equal(actual: &[ParticleBodyContact], expected: &[ParticleBodyContact]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual.particle, expected.particle);
        assert_eq!(actual.fixture, expected.fixture);
        assert_eq!(actual.weight.to_bits(), expected.weight.to_bits());
        assert_eq!(actual.normal.x.to_bits(), expected.normal.x.to_bits());
        assert_eq!(actual.normal.y.to_bits(), expected.normal.y.to_bits());
        assert_eq!(actual.mass.to_bits(), expected.mass.to_bits());
    }
}

#[test]
fn chain_fixture_contacts_match_per_particle_distance() {
    // Arrange
    let storage = storage(
        &rotated_positions(&[
            Vec2::new(0.0, -0.47),
            Vec2::new(0.52, 0.1),
            Vec2::new(-0.1, 0.48),
            Vec2::new(-0.53, -0.2),
            Vec2::new(0.49, -0.49),
            Vec2::new(5.0, 5.0),
        ]),
        0.1,
    );
    let view = ParticleSystemView::new(&storage);
    let chain = ChainShape::closed(&[
        Vec2::new(-0.5, -0.5),
        Vec2::new(0.5, -0.5),
        Vec2::new(0.5, 0.5),
        Vec2::new(-0.5, 0.5),
    ])
    .expect("chain");
    let sources = single_source(&storage, Shape::from(chain));
    let expected = legacy(&view, &sources, &[], 0.1, 2.0, false, |_| true).contacts;

    // Act
    let actual = generate(&view, &sources, &[], 0.1, 2.0, false, |_| true).contacts;

    // Assert
    assert!(
        expected.len() >= 4,
        "particles touch several chain children"
    );
    assert_contact_bits_equal(&actual, &expected);
}

#[test]
fn polygon_fixture_contacts_unchanged_by_hoist() {
    // Arrange
    let storage = storage(
        &rotated_positions(&[
            Vec2::new(0.0, -0.24),
            Vec2::new(0.33, 0.05),
            Vec2::new(-0.1, 0.22),
            Vec2::new(5.0, 5.0),
        ]),
        0.1,
    );
    let view = ParticleSystemView::new(&storage);
    let polygon = PolygonShape::oriented_box(0.3, 0.2, Vec2::ZERO, 0.0).expect("box");
    let sources = single_source(&storage, Shape::from(polygon));
    let expected = legacy(&view, &sources, &[], 0.1, 2.0, false, |_| true).contacts;

    // Act
    let actual = generate(&view, &sources, &[], 0.1, 2.0, false, |_| true).contacts;

    // Assert
    assert!(!expected.is_empty(), "particles touch the polygon");
    assert_contact_bits_equal(&actual, &expected);
}
