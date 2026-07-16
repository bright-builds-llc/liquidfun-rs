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
fn step_pressure_reacts_on_dynamic_body_without_integrating_particle_position() {
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
    assert_eq!(particle_after.position(), Vec2::new(1.5, 0.0));
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

struct PanickingFixtureParticleHook;

impl CollisionDecisionHook for PanickingFixtureParticleHook {
    fn should_collide_fixture_particle(
        &mut self,
        _contact: FixtureParticleView<'_>,
    ) -> CollisionDirective {
        panic!("intentional fixture-particle hook panic");
    }
}

#[test]
fn step_fixture_particle_hook_panic_poisoning_is_sticky() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let _particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_FILTER),
        )
        .expect("particle fits")
        .created_particle();

    // Act
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.step(
            step_configuration(),
            &mut PanickingFixtureParticleHook,
            StepLimits::default(),
        );
    }));

    // Assert
    assert!(panic.is_err());
    assert_eq!(
        world.step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        ),
        Err(StepError::Poisoned)
    );
}

#[derive(Default)]
struct ParticlePrefixOrderHook {
    fixture_particles: Vec<liquidfun::ParticleId>,
    pair_calls: Vec<[liquidfun::ParticleId; 2]>,
}

impl CollisionDecisionHook for ParticlePrefixOrderHook {
    fn should_collide_fixture_particle(
        &mut self,
        contact: FixtureParticleView<'_>,
    ) -> CollisionDirective {
        self.fixture_particles.push(contact.particle());
        CollisionDirective::Collide
    }

    fn should_collide_particle_pair(
        &mut self,
        contact: ParticlePairContactView<'_>,
    ) -> CollisionDirective {
        self.pair_calls.push(contact.particles());
        CollisionDirective::Collide
    }
}

#[test]
fn step_runs_all_subiterations_newest_system_first_and_skips_paused_systems() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
    let old_system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("old system fits");
    let old_particle = world
        .create_particle_with_def(
            old_system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_FILTER),
        )
        .expect("old particle fits")
        .created_particle();
    let paused_system = world
        .create_particle_system_with_def(&ParticleSystemDef::default().with_paused(true))
        .expect("paused system fits");
    let paused_particle = world
        .create_particle_with_def(
            paused_system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_FILTER),
        )
        .expect("paused particle fits")
        .created_particle();
    let new_system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("new system fits");
    let new_particle = world
        .create_particle_with_def(
            new_system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_FILTER),
        )
        .expect("new particle fits")
        .created_particle();
    let step = step_configuration()
        .with_particle_iterations(3)
        .expect("particle iterations are valid");
    let mut hook = ParticlePrefixOrderHook::default();

    // Act
    world
        .step(step, &mut hook, StepLimits::default())
        .expect("multi-system prefix succeeds");

    // Assert
    assert_eq!(
        hook.fixture_particles,
        vec![
            new_particle,
            new_particle,
            new_particle,
            old_particle,
            old_particle,
            old_particle,
        ]
    );
    assert!(!hook.fixture_particles.contains(&paused_particle));
    assert_eq!(
        world
            .particle_system_view(paused_system)
            .expect("paused system remains live")
            .body_contacts()
            .count(),
        0
    );
}

#[test]
fn step_particle_pair_filter_and_listener_share_the_source_timed_journal() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let flags = ParticleFlags::PARTICLE_CONTACT_FILTER | ParticleFlags::PARTICLE_CONTACT_LISTENER;
    let first = world
        .create_particle_with_def(system, None, &ParticleDef::default().with_flags(flags))
        .expect("first particle fits")
        .created_particle();
    let second = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.0, 0.0))
                .expect("particle position is valid"),
        )
        .expect("second particle fits")
        .created_particle();
    let mut hook = ParticlePrefixOrderHook::default();

    // Act
    let report = world
        .step(step_configuration(), &mut hook, StepLimits::default())
        .expect("particle pair prefix succeeds");

    // Assert
    assert_eq!(hook.pair_calls, vec![[first, second]]);
    assert!(report.lifecycle().iter().any(|event| matches!(
        event,
        StepLifecycleEvent::ParticleContact(liquidfun::ParticleContactEffect::Begin(contact))
            if contact.particles() == [first, second]
    )));
}

#[test]
fn step_off_center_body_impulse_updates_angular_velocity() {
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
    let mass_shape =
        Shape::from(CircleShape::new(Vec2::new(0.0, -5.0), 1.0).expect("mass fixture is valid"));
    world
        .create_fixture(
            body,
            &FixtureDef::new(mass_shape, 1.0, 0.0, 0.0, false, FilterData::default())
                .expect("mass fixture is valid"),
        )
        .expect("mass fixture fits");
    circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
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
        .expect("off-center coupling succeeds");

    // Assert
    assert_ne!(
        world
            .body_snapshot(body)
            .expect("body remains live")
            .angular_velocity()
            .to_bits(),
        0.0_f32.to_bits()
    );
}

#[test]
fn step_listener_limit_rolls_back_particle_contacts_and_rigid_reaction() {
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
                .expect("particle position is valid")
                .with_flags(ParticleFlags::FIXTURE_CONTACT_LISTENER),
        )
        .expect("particle fits")
        .created_particle();
    let zero_events = StepLimits::new(0, StepLimits::default().max_commands())
        .expect("zero event limit is valid");

    // Act
    let result = world.step(step_configuration(), &mut NoDecisionHook, zero_events);

    // Assert
    assert_eq!(
        result,
        Err(StepError::LimitExceeded {
            resource: "event",
            limit: 0,
        })
    );
    assert_eq!(
        world
            .particle_snapshot(particle)
            .expect("particle remains live")
            .velocity(),
        Vec2::ZERO
    );
    assert_eq!(
        world
            .particle_system_view(system)
            .expect("system remains live")
            .body_contacts()
            .count(),
        0
    );
}
