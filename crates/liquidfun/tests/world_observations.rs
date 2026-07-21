//! Public default-feature coverage for bounded renderer-neutral observations.

use liquidfun::collision::{Aabb, CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, DiagnosticStepPhase, FixtureDef, NoDecisionHook, ParticleDef,
    ParticleSystemDef, StepConfiguration, StepLimits, World, WorldObservationError,
    WorldObservationLimits, WorldObservationResource,
};

fn circle_fixture(world: &mut World, body: liquidfun::BodyId) -> liquidfun::FixtureId {
    let shape = Shape::from(
        CircleShape::new(Vec2::ZERO, 1.0).expect("test circle geometry should be valid"),
    );
    let definition = FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
        .expect("test fixture should be valid");
    world
        .create_fixture(body, &definition)
        .expect("test fixture should fit")
}

fn step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3).expect("test step configuration should be valid")
}

#[test]
fn default_feature_observation_uses_stable_semantic_identities() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let static_body = world
        .create_body(&BodyDef::default())
        .expect("static body should fit");
    let static_fixture = circle_fixture(&mut world, static_body);
    let dynamic_body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(0.75, 0.0), 0.0, true)
                .expect("dynamic body should be valid"),
        )
        .expect("dynamic body should fit");
    let dynamic_fixture = circle_fixture(&mut world, dynamic_body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system should fit");
    let first = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(-0.5, 0.0))
                .expect("particle position should be finite"),
        )
        .expect("first particle should fit")
        .created_particle();
    let second = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(0.5, 0.0))
                .expect("particle position should be finite"),
        )
        .expect("second particle should fit")
        .created_particle();
    world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("contact-producing step should succeed");

    // Act
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should fit");

    // Assert
    assert_eq!(observation.diagnostics().body_count(), 2);
    assert_eq!(observation.diagnostics().fixture_count(), 2);
    assert_eq!(observation.contacts().len(), 1);
    assert_eq!(
        observation.contacts()[0].fixtures(),
        [static_fixture, dynamic_fixture]
    );
    assert_eq!(observation.particle_contacts().len(), 1);
    assert_eq!(
        observation.particle_contacts()[0].particles(),
        [first, second]
    );
    assert!(
        observation
            .particle_body_contacts()
            .iter()
            .all(|contact| [first, second].contains(&contact.particle()))
    );
    assert_eq!(
        observation
            .broad_phase_observations()
            .iter()
            .map(|entry| entry.fixture())
            .collect::<Vec<_>>(),
        vec![dynamic_fixture, static_fixture]
    );
    let query = Aabb::new(Vec2::new(-2.0, -2.0), Vec2::new(2.0, 2.0))
        .expect("test query bounds should be valid");
    assert!(
        observation
            .broad_phase_observations()
            .iter()
            .all(|entry| entry.overlaps(query))
    );
    assert_eq!(observation.particle_statistics().len(), 1);
    assert_eq!(observation.particle_statistics()[0].system(), system);
    assert_eq!(
        observation.particle_statistics()[0].particle_ids(),
        [first, second]
    );
    assert_eq!(observation.particle_world_statistics().system_count(), 1);
    let rendered = format!("{observation:?}");
    assert!(!rendered.contains("ProxyId"));
    assert!(!rendered.contains("dense"));
    assert!(!rendered.contains("arena"));
}

#[test]
fn observation_limits_accept_exact_capacity_and_reject_first_excess_record() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    circle_fixture(&mut world, body);
    circle_fixture(&mut world, body);
    let reviewed = WorldObservationLimits::reviewed();
    let at_limit = WorldObservationLimits::new(
        reviewed.max_contacts(),
        reviewed.max_particle_contacts(),
        reviewed.max_particle_body_contacts(),
        2,
        reviewed.max_particle_systems(),
        reviewed.max_particles(),
    )
    .expect("two broad-phase records are within reviewed maxima");
    let one_short = WorldObservationLimits::new(
        reviewed.max_contacts(),
        reviewed.max_particle_contacts(),
        reviewed.max_particle_body_contacts(),
        1,
        reviewed.max_particle_systems(),
        reviewed.max_particles(),
    )
    .expect("one broad-phase record is within reviewed maxima");

    // Act
    let accepted = world.world_observation(at_limit);
    let rejected = world.world_observation(one_short);

    // Assert
    assert_eq!(
        accepted
            .expect("the exact record limit should be accepted")
            .broad_phase_observations()
            .len(),
        2
    );
    assert_eq!(
        rejected,
        Err(WorldObservationError::CapacityExceeded {
            resource: WorldObservationResource::BroadPhaseObservations,
            limit: 1,
        })
    );
}

#[test]
fn profiled_step_keeps_wall_clock_diagnostics_separate_from_step_equality() {
    // Arrange
    let mut profiled_world = World::new().expect("world key should remain available");
    let mut ordinary_world = World::new().expect("world key should remain available");
    let configuration = step_configuration();

    // Act
    let (profiled_report, profile) = profiled_world
        .step_profiled(configuration, &mut NoDecisionHook, StepLimits::default())
        .expect("profiled step should succeed");
    let ordinary_report = ordinary_world
        .step(configuration, &mut NoDecisionHook, StepLimits::default())
        .expect("ordinary step should succeed");

    // Assert
    assert_eq!(profiled_report, ordinary_report);
    assert_eq!(
        profile
            .phases()
            .iter()
            .map(|timing| timing.phase())
            .collect::<Vec<_>>(),
        vec![
            DiagnosticStepPhase::ContactLifecycle,
            DiagnosticStepPhase::ParticleSolve,
            DiagnosticStepPhase::RigidSolve,
            DiagnosticStepPhase::ContinuousSolve,
            DiagnosticStepPhase::Finalize,
        ]
    );
    assert!(
        profile
            .phases()
            .iter()
            .all(|timing| timing.duration() <= std::time::Duration::from_mins(1))
    );
}
