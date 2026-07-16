//! Public regressions for the pinned fresh-positive-dt particle stage boundary.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleDef, ParticleFlags, ParticleSystemDef};
use liquidfun::{
    BodyDef, BodyType, FixtureDef, HandleError, NoDecisionHook, StepCompletion, StepConfiguration,
    StepError, StepLimits, World,
};

fn configuration(time_step: f32) -> StepConfiguration {
    StepConfiguration::new(time_step, 8, 3).expect("test step configuration is valid")
}

fn create_contact_fixture(world: &mut World) -> liquidfun::BodyId {
    let body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
                .expect("dynamic body definition is valid"),
        )
        .expect("dynamic body fits");
    let fixture = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("circle is valid")),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("fixture definition is valid");
    world.create_fixture(body, &fixture).expect("fixture fits");
    body
}

#[test]
fn zero_dt_preserves_particle_lifecycle_contacts_and_rigid_reaction() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity is valid");
    let body = create_contact_fixture(&mut world);
    let definition = ParticleSystemDef::default()
        .with_lifetime_granularity(1.0)
        .expect("lifetime granularity is valid");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits");
    let finite = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("contact position is valid")
                .with_lifetime(2.0)
                .expect("finite lifetime is valid"),
        )
        .expect("finite particle fits");
    let pending = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.5))
                .expect("pending position is valid")
                .with_flags(ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("pending particle fits");
    world
        .step(
            configuration(1.0),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("fresh positive step populates particle contacts");
    world
        .mark_particle_for_destruction(pending)
        .expect("particle becomes pending");
    let before_particle = world
        .particle_snapshot(finite)
        .expect("finite particle remains live");
    let before_body = world.body_snapshot(body).expect("body remains live");
    let (before_weights, before_particle_contacts, before_body_contacts) = {
        let view = world
            .particle_system_view(system)
            .expect("system remains live");
        (
            view.weights().to_vec(),
            view.particle_contacts().count(),
            view.body_contacts().count(),
        )
    };

    // Act
    let report = world
        .step(
            configuration(0.0),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("zero-duration rigid maintenance succeeds");

    // Assert
    assert_eq!(
        world.particle_snapshot(finite),
        Ok(before_particle),
        "zero dt must not advance the finite lifetime"
    );
    assert_eq!(
        world.particle_snapshot(pending),
        Err(HandleError::PendingDelete)
    );
    assert_eq!(world.body_snapshot(body), Ok(before_body));
    let after = world
        .particle_system_view(system)
        .expect("system remains live");
    assert_eq!(after.weights(), before_weights);
    assert_eq!(after.particle_contacts().count(), before_particle_contacts);
    assert_eq!(after.body_contacts().count(), before_body_contacts);
    assert!(report.lifecycle().is_empty());
}

#[test]
fn continuous_resume_does_not_repeat_particle_stages() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity is valid");
    let _body = create_contact_fixture(&mut world);
    let definition = ParticleSystemDef::default()
        .with_lifetime_granularity(1.0)
        .expect("lifetime granularity is valid");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("contact position is valid")
                .with_lifetime(2.0)
                .expect("finite lifetime is valid"),
        )
        .expect("finite particle fits");
    let exhausted_limits = StepLimits::default()
        .with_continuous_work_limit(0)
        .expect("zero is a coherent continuous boundary");
    let step = configuration(1.0);

    // Act
    let exhausted = world.step(step, &mut NoDecisionHook, exhausted_limits);
    let snapshot_after_fresh = world
        .particle_snapshot(particle)
        .expect("one fresh step does not expire the particle");
    let (weights_after_fresh, contacts_after_fresh) = {
        let view = world
            .particle_system_view(system)
            .expect("system remains live");
        (view.weights().to_vec(), view.body_contacts().count())
    };
    let resumed = world
        .step(step, &mut NoDecisionHook, StepLimits::default())
        .expect("matching continuation completes");

    // Assert
    assert!(matches!(
        exhausted,
        Err(StepError::ContinuousWorkLimitExceeded { limit: 0, .. })
    ));
    assert_eq!(resumed.completion(), StepCompletion::Complete);
    assert_eq!(world.particle_snapshot(particle), Ok(snapshot_after_fresh));
    let after_resume = world
        .particle_system_view(system)
        .expect("system remains live");
    assert_eq!(after_resume.weights(), weights_after_fresh);
    assert_eq!(after_resume.body_contacts().count(), contacts_after_fresh);
}

#[test]
fn fresh_positive_dt_still_runs_particle_lifecycle() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let definition = ParticleSystemDef::default()
        .with_lifetime_granularity(1.0)
        .expect("lifetime granularity is valid")
        .with_paused(true);
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_lifetime(1.0)
                .expect("finite lifetime is valid"),
        )
        .expect("particle fits");

    // Act
    world
        .step(
            configuration(1.0),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("fresh positive step succeeds");

    // Assert
    assert_eq!(
        world.particle_snapshot(particle),
        Err(HandleError::StaleOrDestroyed)
    );
}
