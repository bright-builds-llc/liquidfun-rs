//! Default-feature proof for the complete renderer-neutral observation surface.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, DebugDrawOptions, DebugLayer, FixtureDef, NoDecisionHook, ParticleDef,
    ParticleSystemDef, StepConfiguration, StepLimits, World, WorldObservationLimits,
};

fn add_circle(world: &mut World, body: liquidfun::BodyId) -> liquidfun::FixtureId {
    let circle = CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid");
    let fixture = FixtureDef::new(
        Shape::from(circle),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("fixture should be valid");
    world
        .create_fixture(body, &fixture)
        .expect("fixture should fit")
}

#[test]
fn public_default_features_expose_complete_headless_observability() {
    // Arrange
    let mut world = World::new().expect("world identity should be available");
    let ground = world
        .create_body(&BodyDef::default())
        .expect("ground should fit");
    let ground_fixture = add_circle(&mut world, ground);
    let dynamic = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(0.75, 0.0), 0.0, true)
                .expect("dynamic body should be valid"),
        )
        .expect("dynamic body should fit");
    let dynamic_fixture = add_circle(&mut world, dynamic);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system should fit");
    let particles = [Vec2::new(-0.5, 0.0), Vec2::new(0.5, 0.0)].map(|position| {
        world
            .create_particle_with_def(
                system,
                None,
                &ParticleDef::default()
                    .with_position(position)
                    .expect("particle position should be finite"),
            )
            .expect("particle should fit")
            .created_particle()
    });
    world
        .step(
            StepConfiguration::new(1.0 / 60.0, 8, 3).expect("settings should be valid"),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("headless step should succeed");

    // Act
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should fit");
    let primitives = world
        .collect_debug_primitives(DebugDrawOptions::all())
        .expect("reviewed debug collection should fit");

    // Assert
    let diagnostics = observation.diagnostics();
    assert_eq!(
        (diagnostics.body_count(), diagnostics.fixture_count()),
        (2, 2)
    );
    assert!(diagnostics.tree_height() >= 0);
    assert!(diagnostics.tree_balance() >= 0);
    assert!(diagnostics.tree_quality().is_finite());
    assert_eq!(observation.contacts().len(), 1);
    assert_eq!(
        observation.contacts()[0].fixtures(),
        [ground_fixture, dynamic_fixture]
    );
    assert_eq!(observation.particle_contacts().len(), 1);
    assert_eq!(observation.particle_contacts()[0].particles(), particles);
    assert_eq!(observation.broad_phase_observations().len(), 2);
    assert_eq!(observation.particle_statistics().len(), 1);
    assert_eq!(
        observation.particle_statistics()[0].particle_ids(),
        particles
    );
    assert!(
        primitives
            .primitives()
            .iter()
            .any(|primitive| primitive.layer() == DebugLayer::Contacts)
    );
    assert!(
        primitives
            .primitives()
            .iter()
            .any(|primitive| primitive.layer() == DebugLayer::ParticleContacts)
    );
    assert!(
        primitives
            .primitives()
            .iter()
            .any(|primitive| primitive.layer() == DebugLayer::BroadPhase)
    );
}
