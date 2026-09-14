use super::*;
use crate::identity::HandleIdentity;
use crate::particle::{
    ParticleGroupDestination, ParticleGroupFlags, ParticleGroupRecipe, ParticleGroupSource,
};
use crate::{NoDecisionHook, ParticleColor, ParticleFlags, StepConfiguration, StepLimits};

fn recipe(positions: Vec<Vec2>, destination: ParticleGroupDestination) -> ParticleGroupRecipe<()> {
    ParticleGroupRecipe::new(
        ParticleGroupSource::positions(positions).expect("finite bounded positions"),
        destination,
    )
}

fn fixture() -> (World, ParticleSystemId, ParticleGroupId) {
    let mut world = test_world();
    world.set_gravity(Vec2::ZERO).expect("finite gravity");
    let body = world.create_body(&BodyDef::default()).expect("body fits");
    world
        .create_fixture(body, &test_fixture_definition())
        .expect("fixture fits");
    let system = world.create_particle_system().expect("system fits");
    let group = world
        .create_particle_group(
            system,
            &recipe(
                vec![Vec2::ZERO, Vec2::new(1.5, 0.0), Vec2::new(0.75, 1.3)],
                ParticleGroupDestination::New,
            )
            .with_particle_flags(
                ParticleFlags::SPRING
                    | ParticleFlags::ELASTIC
                    | ParticleFlags::STATIC_PRESSURE
                    | ParticleFlags::TENSILE
                    | ParticleFlags::WALL,
            )
            .with_group_flags(ParticleGroupFlags::SOLID | ParticleGroupFlags::CAN_BE_EMPTY)
            .with_color(ParticleColor::new(1, 2, 3, 4))
            .with_lifetime(10.0)
            .expect("bounded lifetime"),
        )
        .expect("populated group fits");
    let first = world
        .create_particle(system, None)
        .expect("particle fits")
        .created_particle();
    let second = world
        .create_particle(system, None)
        .expect("particle fits")
        .created_particle();
    world.destroy_particle(first).expect("live particle");
    world.destroy_particle(second).expect("live particle");
    let pending_group = world
        .create_particle_group(
            system,
            &recipe(vec![Vec2::new(20.0, 0.0)], ParticleGroupDestination::New)
                .with_group_flags(ParticleGroupFlags::CAN_BE_EMPTY)
                .with_particle_flags(ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("pending fixture group fits");
    let empty = world
        .create_particle_group(
            system,
            &recipe(vec![Vec2::new(40.0, 0.0)], ParticleGroupDestination::New)
                .with_group_flags(ParticleGroupFlags::CAN_BE_EMPTY),
        )
        .expect("empty shell fits");
    let removed = world
        .particle_group_view(empty)
        .expect("temporary group live")
        .member_ids()[0];
    world
        .destroy_particle(removed)
        .expect("temporary member live");
    world
        .destroy_empty_particle_group(empty)
        .expect("empty shell removed");
    let report = step(&mut world);
    assert!(report.lifecycle().is_empty());
    world
        .destroy_particle_group_particles(pending_group, true)
        .expect("mark pending member");
    world
        .particle_systems
        .get_mut(system)
        .expect("system live")
        .storage
        .populate_transaction_test_state();
    (world, system, group)
}

fn step(world: &mut World) -> crate::StepReport {
    world
        .step(
            StepConfiguration::new(1.0 / 60.0, 8, 3).expect("valid step"),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("valid world steps")
}

fn invalid_recipe(destination: ParticleGroupDestination) -> ParticleGroupRecipe<()> {
    let positions = (0..64)
        .scan(0.0_f32, |x, _| {
            let position = Vec2::new(*x, 0.0);
            *x += 1000.0;
            Some(position)
        })
        .collect();
    recipe(positions, destination)
        .with_particle_flags(ParticleFlags::ELASTIC)
        .with_lifetime(5.0)
        .expect("bounded lifetime")
}

struct Before {
    systems: Arena<ParticleSystem, ParticleSystemId>,
    groups: Arena<ParticleGroup, ParticleGroupId>,
    system_order: Vec<ParticleSystemId>,
    maybe_diagnostic: Option<u64>,
}

impl Before {
    fn capture(world: &World) -> Self {
        Self {
            systems: world.particle_systems.clone(),
            groups: world.particle_groups.clone(),
            system_order: world.particle_system_order.clone(),
            maybe_diagnostic: world.next_diagnostic_id,
        }
    }

    fn assert_unchanged(&self, world: &World) {
        world
            .particle_systems
            .assert_same_state_for_test(&self.systems, |after, before| {
                assert!(after.storage == before.storage);
                assert_eq!(after.lifetime, before.lifetime);
                assert_eq!(after.definition, before.definition);
                assert_eq!(after.groups, before.groups);
                assert_eq!(after.diagnostic_id, before.diagnostic_id);
                assert_eq!(after.timestamp, before.timestamp);
            });
        world
            .particle_groups
            .assert_same_state_for_test(&self.groups, |after, before| {
                assert_eq!(after.system, before.system);
                assert_eq!(after.diagnostic_id, before.diagnostic_id);
            });
        assert_eq!(world.particle_system_order, self.system_order);
        assert_eq!(world.next_diagnostic_id, self.maybe_diagnostic);
        assert!(!world.step_state.is_locked());
        assert!(!world.step_state.is_poisoned());
    }
}

#[test]
fn new_group_topology_rejection_preserves_complete_private_state() {
    // Arrange
    let (mut world, system, _) = fixture();
    let before = Before::capture(&world);

    // Act
    let result =
        world.create_particle_group(system, &invalid_recipe(ParticleGroupDestination::New));

    // Assert
    assert_eq!(result, Err(CreateObjectError::InvalidParticleGroupTopology));
    before.assert_unchanged(&world);
}

#[test]
fn append_topology_rejection_preserves_complete_private_state() {
    // Arrange
    let (mut world, system, group) = fixture();
    let before = Before::capture(&world);

    // Act
    let result = world.create_particle_group(
        system,
        &invalid_recipe(ParticleGroupDestination::AppendTo(group)),
    );

    // Assert
    assert_eq!(result, Err(CreateObjectError::InvalidParticleGroupTopology));
    before.assert_unchanged(&world);
}

#[test]
fn rejection_preserves_next_allocations_and_deferred_lifecycle() {
    // Arrange
    let (mut world, system, target) = fixture();
    let (mut control, control_system, _) = fixture();
    let next_group = world
        .particle_groups
        .next_handle()
        .expect("next shell fits");
    let diagnostic = world.next_diagnostic_id.expect("diagnostic fits");
    let success = recipe(vec![Vec2::new(30.0, 0.0)], ParticleGroupDestination::New);
    let next_particle = world
        .particle_systems
        .get(system)
        .expect("system live")
        .storage
        .next_particle_for_test();
    let before = Before::capture(&world);

    // Act
    let rejection =
        world.create_particle_group(system, &invalid_recipe(ParticleGroupDestination::New));
    before.assert_unchanged(&world);
    let append_rejection = world.create_particle_group(
        system,
        &invalid_recipe(ParticleGroupDestination::AppendTo(target)),
    );
    before.assert_unchanged(&world);
    let created = world
        .create_particle_group(system, &success)
        .expect("success after rejection");
    let control_created = control
        .create_particle_group(control_system, &success)
        .expect("control success");
    let member = world
        .particle_group_view(created)
        .expect("group live")
        .member_ids()[0];
    let control_member = control
        .particle_group_view(control_created)
        .expect("control live")
        .member_ids()[0];
    let report = step(&mut world);
    let control_report = step(&mut control);

    // Assert
    assert_eq!(
        rejection,
        Err(CreateObjectError::InvalidParticleGroupTopology)
    );
    assert_eq!(created, next_group);
    assert_eq!(
        append_rejection,
        Err(CreateObjectError::InvalidParticleGroupTopology)
    );
    assert_eq!(member, next_particle);
    assert_eq!(
        world
            .particle_groups
            .get(created)
            .expect("live shell")
            .diagnostic_id,
        diagnostic
    );
    assert_eq!(
        world
            .particle_systems
            .get(system)
            .expect("live system")
            .storage
            .snapshot(member)
            .expect("live particle")
            .diagnostic_id,
        diagnostic + 1
    );
    assert_eq!(world.next_diagnostic_id, Some(diagnostic + 2));
    assert_eq!(created.identity().slot(), control_created.identity().slot());
    assert_eq!(
        created.identity().generation(),
        control_created.identity().generation()
    );
    assert_eq!(member.identity().slot(), control_member.identity().slot());
    assert_eq!(
        member.identity().generation(),
        control_member.identity().generation()
    );
    assert_eq!(lifecycle_trace(&report), lifecycle_trace(&control_report));
    assert!(
        !report.lifecycle().is_empty(),
        "pending fixture must exercise deferred output"
    );
    assert!(step(&mut world).lifecycle().is_empty());
    assert!(step(&mut control).lifecycle().is_empty());
}

fn lifecycle_trace(report: &crate::StepReport) -> Vec<(bool, usize, u64, u64, DestructionCause)> {
    report
        .lifecycle()
        .iter()
        .map(|event| {
            let (listener, record) = match event {
                crate::LifecycleEvent::ParticleDestruction(record) => (true, record),
                crate::LifecycleEvent::Destruction(record) => (false, record),
                _ => panic!("fixture must emit only its pending particle lifecycle"),
            };
            let DestroyedId::Particle(id) = record.destroyed() else {
                panic!("retained shells must not be destroyed");
            };
            (
                listener,
                id.identity().slot(),
                id.identity().generation(),
                record.diagnostic_id(),
                record.cause(),
            )
        })
        .collect()
}
