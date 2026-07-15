//! Black-box fixture-particle contact and Phase 9 coupling regressions.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::{Vec2, settings};
use liquidfun::{
    BodyDef, CollisionDecisionHook, CollisionDirective, FixtureDef, FixtureParticleView,
    NoDecisionHook, ParticleBodyContactEffect, ParticleDef, ParticleFlags, ParticleSystemDef,
    StepConfiguration, StepLifecycleEvent, StepLimits, World,
};

fn step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3).expect("test step is valid")
}

fn circle_fixture(world: &mut World, body: liquidfun::BodyId) -> liquidfun::FixtureId {
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test fixture geometry is valid"));
    world
        .create_fixture(
            body,
            &FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
                .expect("test fixture is valid"),
        )
        .expect("fixture fits")
}

#[test]
fn contacts_include_fixture_body_particle_and_source_fields() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    let fixture = circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid"),
        )
        .expect("particle fits");

    // Act
    world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("contact-only step succeeds");

    // Assert
    let contacts = world
        .particle_system_view(system)
        .expect("system remains live")
        .body_contacts()
        .collect::<Vec<_>>();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].particle(), particle);
    assert_eq!(contacts[0].body(), body);
    assert_eq!(contacts[0].fixture(), fixture);
    assert_eq!(contacts[0].weight().to_bits(), 0.75_f32.to_bits());
    assert_eq!(contacts[0].normal(), Vec2::new(-1.0, 0.0));
    let inverse_stride = 0.5 * (1.0 / settings::PARTICLE_STRIDE);
    let expected_mass = 1.0 / (inverse_stride * inverse_stride);
    assert_eq!(contacts[0].mass().to_bits(), expected_mass.to_bits());
}

#[test]
fn contacts_strict_pruning_matches_the_independent_equal_weight_witness() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    let fixtures = (0..6)
        .map(|_| circle_fixture(&mut world, body))
        .collect::<Vec<_>>();
    let system = world
        .create_particle_system_with_def(
            &ParticleSystemDef::default().with_strict_contact_check(true),
        )
        .expect("particle system fits");
    world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid"),
        )
        .expect("particle fits");

    // Act
    world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("strict contact step succeeds");

    // Assert
    let retained = world
        .particle_system_view(system)
        .expect("system remains live")
        .body_contacts()
        .map(liquidfun::ParticleBodyContactView::fixture)
        .collect::<Vec<_>>();
    assert_eq!(
        retained,
        fixtures[2..].iter().rev().copied().collect::<Vec<_>>()
    );
}

#[derive(Default)]
struct RejectFlaggedFixtureContact {
    calls: Vec<liquidfun::ParticleId>,
}

impl CollisionDecisionHook for RejectFlaggedFixtureContact {
    fn should_collide_fixture_particle(
        &mut self,
        contact: FixtureParticleView<'_>,
    ) -> CollisionDirective {
        self.calls.push(contact.particle());
        CollisionDirective::Ignore
    }
}

#[test]
fn contacts_fixture_filter_is_borrowed_and_flag_gated() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let unflagged = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid"),
        )
        .expect("particle fits");
    let flagged = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(-1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_FILTER),
        )
        .expect("particle fits");
    let mut hook = RejectFlaggedFixtureContact::default();

    // Act
    world
        .step(step_configuration(), &mut hook, StepLimits::default())
        .expect("filtered contact step succeeds");

    // Assert
    assert_eq!(hook.calls, vec![flagged]);
    let retained = world
        .particle_system_view(system)
        .expect("system remains live")
        .body_contacts()
        .map(liquidfun::ParticleBodyContactView::particle)
        .collect::<Vec<_>>();
    assert_eq!(retained, vec![unflagged]);
}

#[test]
fn contacts_fixture_listener_begins_and_ends_in_the_shared_journal() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    let fixture = circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_LISTENER),
        )
        .expect("particle fits");

    // Act
    let begin = world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("begin step succeeds");
    world
        .set_body_transform(body, Vec2::new(10.0, 0.0), 0.0)
        .expect("body translation is valid");
    let end = world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("end step succeeds");

    // Assert
    assert!(begin.lifecycle().iter().any(|event| matches!(
        event,
        StepLifecycleEvent::ParticleBodyContact(ParticleBodyContactEffect::Begin(contact))
            if contact.fixture() == fixture && contact.particle() == particle
    )));
    assert!(end.lifecycle().iter().any(|event| matches!(
        event,
        StepLifecycleEvent::ParticleBodyContact(ParticleBodyContactEffect::End {
            fixture: ended_fixture,
            particle: ended_particle,
        }) if *ended_fixture == fixture && *ended_particle == particle
    )));
}

#[test]
fn contacts_stuck_candidates_require_more_than_the_configured_threshold() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
    circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default().with_stuck_threshold(1))
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid"),
        )
        .expect("particle fits");

    // Act
    world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("first contact step succeeds");
    let first = world
        .particle_system_view(system)
        .expect("system remains live")
        .stuck_candidates()
        .collect::<Vec<_>>();
    world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("second contact step succeeds");
    let second = world
        .particle_system_view(system)
        .expect("system remains live")
        .stuck_candidates()
        .collect::<Vec<_>>();

    // Assert
    assert!(first.is_empty());
    assert_eq!(second, vec![particle]);
}
