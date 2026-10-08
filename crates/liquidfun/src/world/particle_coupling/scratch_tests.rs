//! Fresh collider metadata and workspace restoration across step errors.
use super::*;
use crate::collision::{CircleShape, FilterData, PolygonShape};
use crate::{
    BodyDef, BodyType, FixtureDef, NoDecisionHook, ParticleDef, ParticleSystemDef, StepLimits,
};

fn fixture(world: &mut World, body: BodyId) -> FixtureId {
    let shape = Shape::from(PolygonShape::box_shape(0.4, 0.3).expect("box"));
    world
        .create_fixture(
            body,
            &FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default()).expect("fixture"),
        )
        .expect("fixture")
}

#[test]
fn reused_collider_buffers_refresh_transform_mass_sensor_and_identity() {
    // Arrange
    let mut world = World::new().expect("world");
    let body = world
        .create_body(&BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true).expect("body"))
        .expect("body");
    let old = fixture(&mut world, body);
    let mut sources = Vec::with_capacity(8);
    let mut buffers = CollisionBuffers::default();
    world.fixture_contact_sources(&world.bodies, &mut sources);
    let mass = sources[0].inverse_mass;
    ccd_fixture_records(&world, &world.bodies, Vec2::ZERO, &mut buffers).expect("records");
    let child_pointer = buffers.fixtures[0].children.as_ptr();

    // Act / Assert
    world
        .set_body_transform(body, Vec2::new(2.0, 3.0), 0.25)
        .expect("move");
    world.set_fixture_density(old, 3.0).expect("density");
    world.reset_body_mass_data(body).expect("mass");
    world.fixture_contact_sources(&world.bodies, &mut sources);
    assert_eq!(sources[0].transform.position(), Vec2::new(2.0, 3.0));
    assert!(sources[0].inverse_mass < mass);
    buffers.recycle_records();
    ccd_fixture_records(&world, &world.bodies, Vec2::ZERO, &mut buffers).expect("fresh records");
    assert_eq!(buffers.fixtures[0].children.as_ptr(), child_pointer);
    assert_eq!(
        buffers.fixtures[0].current_transform.position(),
        Vec2::new(2.0, 3.0)
    );
    world.set_fixture_sensor(old, true).expect("sensor");
    world.fixture_contact_sources(&world.bodies, &mut sources);
    assert!(sources.is_empty());
    world.destroy_fixture(old).expect("destroy");
    let new = fixture(&mut world, body);
    world.fixture_contact_sources(&world.bodies, &mut sources);
    assert_eq!(sources[0].fixture, new);
    assert_ne!(sources[0].fixture, old);
    buffers.recycle_records();
    ccd_fixture_records(&world, &world.bodies, Vec2::ZERO, &mut buffers)
        .expect("replacement records");
    assert_eq!(buffers.fixtures[0].fixture, new);
}

#[test]
fn repeated_limit_errors_restore_workspace_and_keep_legacy_rollback_state() {
    // Arrange
    let mut world = World::new().expect("world");
    world.set_gravity(Vec2::ZERO).expect("gravity");
    let body = world.create_body(&BodyDef::default()).expect("body");
    world
        .create_fixture(
            body,
            &FixtureDef::new(
                Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("circle")),
                0.0,
                0.2,
                0.0,
                false,
                FilterData::default(),
            )
            .expect("fixture"),
        )
        .expect("fixture");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("system");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_flags(ParticleFlags::FIXTURE_CONTACT_LISTENER)
                .with_position(Vec2::new(0.2, 0.0))
                .expect("position"),
        )
        .expect("particle")
        .created_particle();
    world.particle_step_scratch.body_sources.reserve(32);
    let capacity = world.particle_step_scratch.body_sources.capacity();
    let configuration = StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("step")
        .with_particle_iterations(4)
        .expect("iterations");

    // Act / Assert
    for _ in 0..2 {
        world
            .set_particle_position(particle, Vec2::new(0.2, 0.0))
            .expect("reset");
        let before = world
            .particle_system_view(system)
            .expect("view")
            .positions()
            .to_vec();
        let result = world.step(
            configuration,
            &mut NoDecisionHook,
            StepLimits::new(0, 0).expect("limits"),
        );
        assert!(matches!(result, Err(StepError::LimitExceeded { .. })));
        assert_eq!(
            world
                .particle_system_view(system)
                .expect("view")
                .positions(),
            before
        );
        assert!(world.particle_step_scratch.body_sources.is_empty());
        assert_eq!(
            world.particle_step_scratch.body_sources.capacity(),
            capacity
        );
        world
            .step(configuration, &mut NoDecisionHook, StepLimits::default())
            .expect("success after error");
    }
    world
        .destroy_particle_system(system)
        .expect("destroy system");
    let replacement = world.create_particle_system().expect("replacement");
    let receipt = world
        .create_particle_with_def(
            replacement,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(4.0, 4.0))
                .expect("position"),
        )
        .expect("particle");
    assert!(receipt.destruction_occurrences().is_empty());
    world
        .step(configuration, &mut NoDecisionHook, StepLimits::default())
        .expect("replacement step");
    assert_eq!(
        world
            .particle_system_view(replacement)
            .expect("view")
            .positions()
            .len(),
        1
    );
}
