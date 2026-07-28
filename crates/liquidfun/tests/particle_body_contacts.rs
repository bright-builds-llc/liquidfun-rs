//! Black-box fixture-particle contact and Phase 9 coupling regressions.

use std::panic::{AssertUnwindSafe, catch_unwind};

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::{Vec2, settings};
use liquidfun::{
    BodyDef, BodyType, CollisionDecisionHook, CollisionDirective, FixtureDef, FixtureParticleView,
    NoDecisionHook, ParticleBodyContactEffect, ParticleDef, ParticleFlags, ParticlePairContactView,
    ParticleSystemDef, StepConfiguration, StepError, StepLifecycleEvent, StepLimits, World,
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
        .expect("particle fits")
        .created_particle();

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
    let _particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid"),
        )
        .expect("particle fits")
        .created_particle();

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
        .expect("particle fits")
        .created_particle();
    let flagged = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(-1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_FILTER),
        )
        .expect("particle fits")
        .created_particle();
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
        .expect("particle fits")
        .created_particle();

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
        .expect("particle fits")
        .created_particle();

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

#[test]
fn step_pressure_reacts_on_dynamic_body_and_integrates_particle_position() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity is valid");
    let body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true).expect("body is valid"),
        )
        .expect("dynamic body fits");
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test fixture geometry is valid"));
    world
        .create_fixture(
            body,
            &FixtureDef::new(shape, 1.0, 0.0, 0.0, false, FilterData::default())
                .expect("dynamic fixture is valid"),
        )
        .expect("dynamic fixture fits");
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
        .expect("particle fits")
        .created_particle();
    let step = StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("step is valid")
        .with_particle_iterations(2)
        .expect("particle iteration count is valid");

    // Act
    world
        .step(step, &mut NoDecisionHook, StepLimits::default())
        .expect("particle coupling step succeeds");

    // Assert
    let particle_after = world
        .particle_snapshot(particle)
        .expect("particle remains live");
    let body_after = world.body_snapshot(body).expect("body remains live");
    assert!(particle_after.position().x > 1.5);
    assert_eq!(particle_after.position().y.to_bits(), 0.0_f32.to_bits());
    assert!(particle_after.velocity().x > 0.0);
    assert!(body_after.linear_velocity().x < 0.0);
}

#[test]
fn step_static_pressure_preserves_the_pinned_equation_grouping() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
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
        .expect("particle fits")
        .created_particle();
    let diameter = 2.0_f32;
    let inverse_stride = (1.0 / diameter) * (1.0 / settings::PARTICLE_STRIDE);
    let particle_inverse_mass = inverse_stride * inverse_stride;
    let contact_mass = 1.0 / particle_inverse_mass;
    let inverse_time_step = 1.0 / (1.0_f32 / 60.0);
    let critical_velocity = diameter * inverse_time_step;
    let critical_pressure = critical_velocity * critical_velocity;
    let pressure_per_weight = 0.05 * critical_pressure;
    let velocity_per_pressure = (1.0 / 60.0) / diameter;
    let impulse_x = velocity_per_pressure * 0.75 * contact_mass * (pressure_per_weight * 0.75);
    let expected_velocity_x = particle_inverse_mass * impulse_x;

    // Act
    world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("static pressure step succeeds");

    // Assert
    let velocity = world
        .particle_snapshot(particle)
        .expect("particle remains live")
        .velocity();
    assert_eq!(velocity.x.to_bits(), expected_velocity_x.to_bits());
    assert_eq!(velocity.y.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn step_zero_weight_boundary_is_excluded_from_body_contacts() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(3.0, 0.0))
                .expect("boundary position is valid"),
        )
        .expect("particle fits")
        .created_particle();

    // Act
    world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("boundary step succeeds");

    // Assert
    let view = world
        .particle_system_view(system)
        .expect("system remains live");
    assert_eq!(view.body_contacts().len(), 0);
    assert_eq!(
        world
            .particle_snapshot(particle)
            .expect("particle remains live")
            .velocity(),
        Vec2::ZERO
    );
}

fn separating_particle_velocity(damping: f32) -> Vec2 {
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
    let definition = ParticleSystemDef::default()
        .with_damping(damping)
        .expect("test damping is valid");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_velocity(Vec2::new(10.0, 0.0))
                .expect("particle velocity is valid"),
        )
        .expect("particle fits")
        .created_particle();
    world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("separating step succeeds");
    world
        .particle_snapshot(particle)
        .expect("particle remains live")
        .velocity()
}

#[test]
fn step_damping_skips_separating_fixture_particle_velocity() {
    // Arrange
    let without_damping = separating_particle_velocity(0.0);

    // Act
    let with_damping = separating_particle_velocity(1.0);

    // Assert
    assert_eq!(with_damping, without_damping);
}

include!("particle_body_contacts/step_ordering.rs");
