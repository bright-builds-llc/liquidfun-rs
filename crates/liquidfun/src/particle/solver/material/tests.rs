use crate::identity::{
    BodyId, FixtureId, HandleIdentity, Identity, ParticleGroupId, ParticleId, ParticleSystemId,
    WorldKey,
};
use crate::particle::body_contact::ParticleBodyContact as SemanticBodyContact;
use crate::particle::contact::ParticleContact as SemanticParticleContact;
use crate::particle::storage::ParticleInput;

use super::*;

#[derive(Debug, Clone, Copy)]
struct ParticleSpec {
    velocity: Vec2,
    flags: ParticleFlags,
    maybe_group_slot: Option<usize>,
    maybe_color: Option<ParticleColor>,
}

impl ParticleSpec {
    const fn new(velocity: Vec2, flags: ParticleFlags) -> Self {
        Self {
            velocity,
            flags,
            maybe_group_slot: None,
            maybe_color: None,
        }
    }

    const fn grouped(mut self, slot: usize) -> Self {
        self.maybe_group_slot = Some(slot);
        self
    }

    const fn colored(mut self, color: ParticleColor) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

struct Fixture {
    world: WorldKey,
    storage: ParticleStorage,
    particles: Vec<ParticleId>,
}

impl Fixture {
    fn new(specifications: &[ParticleSpec]) -> Self {
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        let capacity = specifications.len().max(8);
        let mut storage = ParticleStorage::new(world, system, 32, capacity, capacity)
            .expect("test storage contract is valid");
        let particles = specifications
            .iter()
            .copied()
            .map(|specification| {
                storage
                    .create(ParticleInput {
                        position: Vec2::ZERO,
                        velocity: specification.velocity,
                        flags: specification.flags,
                        maybe_group: specification.maybe_group_slot.map(|slot| {
                            ParticleGroupId::from_identity(Identity::new(world, slot, 0))
                        }),
                        maybe_color: specification.maybe_color,
                        maybe_user_association: None,
                        maybe_expiration_time: None,
                    })
                    .expect("test particle fits")
            })
            .collect();
        Self {
            world,
            storage,
            particles,
        }
    }

    fn group(&self, slot: usize) -> ParticleGroupId {
        ParticleGroupId::from_identity(Identity::new(self.world, slot, 0))
    }

    fn contact(&mut self, indices: [usize; 2], flags: ParticleFlags, weight: f32, normal: Vec2) {
        let contact = SemanticParticleContact::new_internal(
            indices.map(|index| self.particles[index]),
            flags,
            weight,
            normal,
        );
        self.storage
            .replace_particle_contacts(&[contact])
            .expect("test contact is valid");
    }

    fn contacts(&mut self, contacts: &[(usize, usize, ParticleFlags, f32, Vec2)]) {
        let contacts = contacts
            .iter()
            .copied()
            .map(|(a, b, flags, weight, normal)| {
                SemanticParticleContact::new_internal(
                    [self.particles[a], self.particles[b]],
                    flags,
                    weight,
                    normal,
                )
            })
            .collect::<Vec<_>>();
        self.storage
            .replace_particle_contacts(&contacts)
            .expect("test contacts are valid");
    }

    fn body_contact(&mut self, particle: usize, body: BodyId, weight: f32, mass: f32) {
        let contact = SemanticBodyContact::new_internal(
            self.particles[particle],
            body,
            FixtureId::from_identity(Identity::new(self.world, 200, 0)),
            weight,
            Vec2::new(0.0, 1.0),
            mass,
        );
        self.storage
            .replace_body_contacts(&[contact])
            .expect("test body contact is valid");
    }
}

#[derive(Clone)]
struct TestBodies {
    body: BodyId,
    velocity: Vec2,
    impulses: Vec<Vec2>,
}

impl MaterialBodyCoupling for TestBodies {
    fn contains_body(&self, body: BodyId) -> bool {
        body == self.body
    }

    fn velocity_at(&self, body: BodyId, _point: Vec2) -> Vec2 {
        assert_eq!(body, self.body);
        self.velocity
    }

    fn apply_linear_impulse(&mut self, body: BodyId, impulse: Vec2, _point: Vec2) {
        assert_eq!(body, self.body);
        self.impulses.push(impulse);
    }
}

fn velocity_bits(storage: &ParticleStorage) -> Vec<[u32; 2]> {
    storage
        .velocities()
        .iter()
        .map(|velocity| [velocity.x.to_bits(), velocity.y.to_bits()])
        .collect()
}

#[test]
fn viscous_has_control_activation_zero_mixed_and_deterministic_witnesses() {
    // Arrange
    let default_definition = ParticleSystemDef::default();
    assert_eq!(
        default_definition.viscous_strength().to_bits(),
        0.25_f32.to_bits()
    );
    let base = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::VISCOUS),
        ParticleSpec::new(Vec2::new(4.0, 0.0), ParticleFlags::WATER),
    ];
    let mut control = Fixture::new(&base);
    control.contact([0, 1], ParticleFlags::WATER, 0.5, Vec2::new(1.0, 0.0));
    let mut active = Fixture::new(&base);
    active.contact([0, 1], ParticleFlags::VISCOUS, 0.5, Vec2::new(1.0, 0.0));
    let mut zero = Fixture::new(&base);
    zero.contact([0, 1], ParticleFlags::VISCOUS, 0.5, Vec2::new(1.0, 0.0));
    let zero_definition = default_definition
        .with_viscous_strength(0.0)
        .expect("zero viscous strength is valid");
    let mut mixed = Fixture::new(&base);
    mixed.contact(
        [0, 1],
        ParticleFlags::VISCOUS | ParticleFlags::POWDER,
        0.5,
        Vec2::new(1.0, 0.0),
    );
    let body = BodyId::from_identity(Identity::new(active.world, 100, 0));
    active.body_contact(0, body, 0.5, 0.01);
    let mut active_bodies = TestBodies {
        body,
        velocity: Vec2::new(0.0, 2.0),
        impulses: Vec::new(),
    };
    let mut empty_bodies = TestBodies {
        body,
        velocity: Vec2::ZERO,
        impulses: Vec::new(),
    };

    // Act
    viscous(&mut control.storage, default_definition, &mut empty_bodies)
        .expect("control solve succeeds");
    viscous(&mut active.storage, default_definition, &mut active_bodies)
        .expect("active solve succeeds");
    viscous(&mut zero.storage, zero_definition, &mut empty_bodies).expect("zero solve succeeds");
    viscous(&mut mixed.storage, default_definition, &mut empty_bodies)
        .expect("mixed solve succeeds");
    let mut repeated = Fixture::new(&base);
    repeated.contact(
        [0, 1],
        ParticleFlags::VISCOUS | ParticleFlags::POWDER,
        0.5,
        Vec2::new(1.0, 0.0),
    );
    viscous(&mut repeated.storage, default_definition, &mut empty_bodies)
        .expect("repeated solve succeeds");

    // Assert
    assert_eq!(
        control.storage.velocities(),
        &[Vec2::ZERO, Vec2::new(4.0, 0.0)]
    );
    assert_eq!(
        zero.storage.velocities(),
        &[Vec2::ZERO, Vec2::new(4.0, 0.0)]
    );
    assert_eq!(
        mixed.storage.velocities(),
        &[Vec2::new(0.5, 0.0), Vec2::new(3.5, 0.0)]
    );
    assert_eq!(
        velocity_bits(&mixed.storage),
        velocity_bits(&repeated.storage)
    );
    assert_eq!(active_bodies.impulses, vec![Vec2::new(0.0, -0.0025)]);
    let active_total = active.storage.velocities()[0] + active.storage.velocities()[1];
    assert!(active_total.y > 0.0);
}

#[test]
fn repulsive_has_control_activation_zero_mixed_and_deterministic_witnesses() {
    // Arrange
    let definition = ParticleSystemDef::default();
    assert_eq!(definition.repulsive_strength().to_bits(), 1.0_f32.to_bits());
    let distinct = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::REPULSIVE).grouped(10),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER).grouped(11),
    ];
    let same = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::REPULSIVE).grouped(10),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER).grouped(10),
    ];
    let mut control = Fixture::new(&same);
    control.contact([0, 1], ParticleFlags::REPULSIVE, 0.5, Vec2::new(1.0, 0.0));
    let mut active = Fixture::new(&distinct);
    active.contact([0, 1], ParticleFlags::REPULSIVE, 0.5, Vec2::new(1.0, 0.0));
    let mut zero = Fixture::new(&distinct);
    zero.contact([0, 1], ParticleFlags::REPULSIVE, 0.5, Vec2::new(1.0, 0.0));
    let zero_definition = definition
        .with_repulsive_strength(0.0)
        .expect("zero repulsive strength is valid");
    let mixed_specs = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::REPULSIVE).grouped(10),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER).grouped(10),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER).grouped(11),
    ];
    let mut mixed = Fixture::new(&mixed_specs);
    mixed.contacts(&[
        (0, 1, ParticleFlags::REPULSIVE, 0.5, Vec2::new(1.0, 0.0)),
        (1, 2, ParticleFlags::REPULSIVE, 0.25, Vec2::new(1.0, 0.0)),
    ]);

    // Act
    repulsive(&mut control.storage, definition, 0.5).expect("control solve succeeds");
    repulsive(&mut active.storage, definition, 0.5).expect("active solve succeeds");
    repulsive(&mut zero.storage, zero_definition, 0.5).expect("zero solve succeeds");
    repulsive(&mut mixed.storage, definition, 0.5).expect("mixed solve succeeds");
    let mut repeated = Fixture::new(&mixed_specs);
    repeated.contacts(&[
        (0, 1, ParticleFlags::REPULSIVE, 0.5, Vec2::new(1.0, 0.0)),
        (1, 2, ParticleFlags::REPULSIVE, 0.25, Vec2::new(1.0, 0.0)),
    ]);
    repulsive(&mut repeated.storage, definition, 0.5).expect("repeated solve succeeds");

    // Assert
    assert_eq!(control.storage.velocities(), &[Vec2::ZERO, Vec2::ZERO]);
    assert_eq!(
        active.storage.velocities(),
        &[Vec2::new(-0.5, 0.0), Vec2::new(0.5, 0.0)]
    );
    assert_eq!(zero.storage.velocities(), &[Vec2::ZERO, Vec2::ZERO]);
    assert_eq!(
        mixed.storage.velocities(),
        &[Vec2::ZERO, Vec2::new(-0.25, 0.0), Vec2::new(0.25, 0.0)]
    );
    assert_eq!(
        velocity_bits(&mixed.storage),
        velocity_bits(&repeated.storage)
    );
}

#[test]
fn powder_has_control_activation_zero_mixed_and_deterministic_witnesses() {
    // Arrange
    let definition = ParticleSystemDef::default();
    assert_eq!(definition.powder_strength().to_bits(), 0.5_f32.to_bits());
    let specifications = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::POWDER),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER),
    ];
    let mut control = Fixture::new(&specifications);
    control.contact([0, 1], ParticleFlags::POWDER, 0.25, Vec2::new(1.0, 0.0));
    let mut active = Fixture::new(&specifications);
    active.contact([0, 1], ParticleFlags::POWDER, 0.75, Vec2::new(1.0, 0.0));
    let mut zero = Fixture::new(&specifications);
    zero.contact([0, 1], ParticleFlags::POWDER, 0.75, Vec2::new(1.0, 0.0));
    let zero_definition = definition
        .with_powder_strength(0.0)
        .expect("zero powder strength is valid");
    let mut mixed = Fixture::new(&specifications);
    mixed.contact(
        [0, 1],
        ParticleFlags::POWDER | ParticleFlags::TENSILE,
        0.75,
        Vec2::new(1.0, 0.0),
    );

    // Act
    powder(&mut control.storage, definition, 0.5).expect("control solve succeeds");
    powder(&mut active.storage, definition, 0.5).expect("active solve succeeds");
    powder(&mut zero.storage, zero_definition, 0.5).expect("zero solve succeeds");
    powder(&mut mixed.storage, definition, 0.5).expect("mixed solve succeeds");
    let mut repeated = Fixture::new(&specifications);
    repeated.contact(
        [0, 1],
        ParticleFlags::POWDER | ParticleFlags::TENSILE,
        0.75,
        Vec2::new(1.0, 0.0),
    );
    powder(&mut repeated.storage, definition, 0.5).expect("repeated solve succeeds");

    // Assert
    assert_eq!(control.storage.velocities(), &[Vec2::ZERO, Vec2::ZERO]);
    assert_eq!(
        active.storage.velocities(),
        &[Vec2::new(-0.25, 0.0), Vec2::new(0.25, 0.0)]
    );
    assert_eq!(zero.storage.velocities(), &[Vec2::ZERO, Vec2::ZERO]);
    assert_eq!(active.storage.velocities(), mixed.storage.velocities());
    assert_eq!(
        velocity_bits(&mixed.storage),
        velocity_bits(&repeated.storage)
    );
}

#[test]
fn tensile_has_control_activation_zero_mixed_and_deterministic_witnesses() {
    // Arrange
    let definition = ParticleSystemDef::default();
    assert_eq!(
        definition.surface_tension_pressure_strength().to_bits(),
        0.2_f32.to_bits()
    );
    assert_eq!(
        definition.surface_tension_normal_strength().to_bits(),
        0.2_f32.to_bits()
    );
    let active_specs = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::TENSILE),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER),
    ];
    let control_specs = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER),
    ];
    let mut control = Fixture::new(&control_specs);
    control.contact([0, 1], ParticleFlags::WATER, 0.75, Vec2::new(1.0, 0.0));
    let mut active = Fixture::new(&active_specs);
    active.contact([0, 1], ParticleFlags::TENSILE, 0.75, Vec2::new(1.0, 0.0));
    let mut zero = Fixture::new(&active_specs);
    zero.contact([0, 1], ParticleFlags::TENSILE, 0.75, Vec2::new(1.0, 0.0));
    let zero_definition = definition
        .with_surface_tension_pressure_strength(0.0)
        .expect("zero pressure strength is valid")
        .with_surface_tension_normal_strength(0.0)
        .expect("zero normal strength is valid");
    let mut mixed = Fixture::new(&active_specs);
    mixed.contact(
        [0, 1],
        ParticleFlags::TENSILE | ParticleFlags::POWDER,
        0.75,
        Vec2::new(1.0, 0.0),
    );

    // Act
    tensile(&mut control.storage, definition, 0.5).expect("control solve succeeds");
    tensile(&mut active.storage, definition, 0.5).expect("active solve succeeds");
    tensile(&mut zero.storage, zero_definition, 0.5).expect("zero solve succeeds");
    tensile(&mut mixed.storage, definition, 0.5).expect("mixed solve succeeds");
    let mut repeated = Fixture::new(&active_specs);
    repeated.contact(
        [0, 1],
        ParticleFlags::TENSILE | ParticleFlags::POWDER,
        0.75,
        Vec2::new(1.0, 0.0),
    );
    tensile(&mut repeated.storage, definition, 0.5).expect("repeated solve succeeds");

    // Assert
    assert_eq!(control.storage.velocities(), &[Vec2::ZERO, Vec2::ZERO]);
    assert_eq!(zero.storage.velocities(), &[Vec2::ZERO, Vec2::ZERO]);
    let expected = -(0.2_f32 * (1.5_f32 - 2.0_f32) + 0.2_f32 * 0.375_f32) * 0.75_f32;
    assert_eq!(
        active.storage.velocities(),
        &[Vec2::new(expected, 0.0), Vec2::new(-expected, 0.0)]
    );
    assert_eq!(active.storage.velocities(), mixed.storage.velocities());
    assert_eq!(
        velocity_bits(&mixed.storage),
        velocity_bits(&repeated.storage)
    );
    assert_eq!(
        active.storage.velocities()[0] + active.storage.velocities()[1],
        Vec2::ZERO
    );
}

#[test]
fn solid_has_control_activation_zero_mixed_and_deterministic_witnesses() {
    // Arrange
    let definition = ParticleSystemDef::default();
    assert_eq!(definition.ejection_strength().to_bits(), 0.5_f32.to_bits());
    let specifications = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER).grouped(10),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER).grouped(11),
    ];
    let mut control = Fixture::new(&specifications);
    control.contact([0, 1], ParticleFlags::WATER, 0.5, Vec2::new(1.0, 0.0));
    let mut active = Fixture::new(&specifications);
    active.contact([0, 1], ParticleFlags::WATER, 0.5, Vec2::new(1.0, 0.0));
    active
        .storage
        .set_group_flags_internal(active.group(10), ParticleGroupFlags::SOLID)
        .expect("solid group flag commits");
    active
        .storage
        .replace_depths(vec![1.0, 0.5])
        .expect("aligned depth witness commits");
    let mut zero = active_fixture(&specifications);
    let zero_definition = definition
        .with_ejection_strength(0.0)
        .expect("zero ejection strength is valid");
    let same_group_specs = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER).grouped(10),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER).grouped(10),
    ];
    let mut mixed = active_fixture(&same_group_specs);

    // Act
    solid(&mut control.storage, definition, 10.0).expect("control solve succeeds");
    solid(&mut active.storage, definition, 10.0).expect("active solve succeeds");
    solid(&mut zero.storage, zero_definition, 10.0).expect("zero solve succeeds");
    solid(&mut mixed.storage, definition, 10.0).expect("mixed solve succeeds");
    let mut repeated = active_fixture(&specifications);
    solid(&mut repeated.storage, definition, 10.0).expect("repeated solve succeeds");

    // Assert
    assert_eq!(control.storage.velocities(), &[Vec2::ZERO, Vec2::ZERO]);
    assert_eq!(
        active.storage.velocities(),
        &[Vec2::new(-3.75, 0.0), Vec2::new(3.75, 0.0)]
    );
    assert_eq!(zero.storage.velocities(), &[Vec2::ZERO, Vec2::ZERO]);
    assert_eq!(mixed.storage.velocities(), &[Vec2::ZERO, Vec2::ZERO]);
    assert_eq!(
        velocity_bits(&active.storage),
        velocity_bits(&repeated.storage)
    );
    assert_eq!(
        active.storage.velocities()[0] + active.storage.velocities()[1],
        Vec2::ZERO
    );
}

fn active_fixture(specifications: &[ParticleSpec]) -> Fixture {
    let mut fixture = Fixture::new(specifications);
    fixture.contact([0, 1], ParticleFlags::WATER, 0.5, Vec2::new(1.0, 0.0));
    fixture
        .storage
        .set_group_flags_internal(fixture.group(10), ParticleGroupFlags::SOLID)
        .expect("solid group flag commits");
    fixture
        .storage
        .replace_depths(vec![1.0, 0.5])
        .expect("aligned depth witness commits");
    fixture
}

#[test]
fn color_mixing_has_control_activation_zero_mixed_and_deterministic_witnesses() {
    // Arrange
    let definition = ParticleSystemDef::default();
    assert_eq!(
        definition.color_mixing_strength().to_bits(),
        0.5_f32.to_bits()
    );
    let first = ParticleColor::new(255, 0, 10, 250);
    let second = ParticleColor::new(0, 255, 250, 10);
    let both_flagged = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::COLOR_MIXING).colored(first),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::COLOR_MIXING).colored(second),
    ];
    let one_flagged = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::COLOR_MIXING).colored(first),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::WATER).colored(second),
    ];
    let mut control = Fixture::new(&one_flagged);
    control.contact(
        [0, 1],
        ParticleFlags::COLOR_MIXING,
        1.0,
        Vec2::new(1.0, 0.0),
    );
    let mut active = Fixture::new(&both_flagged);
    active.contact(
        [0, 1],
        ParticleFlags::COLOR_MIXING,
        1.0,
        Vec2::new(1.0, 0.0),
    );
    let mut zero = Fixture::new(&both_flagged);
    zero.contact(
        [0, 1],
        ParticleFlags::COLOR_MIXING,
        1.0,
        Vec2::new(1.0, 0.0),
    );
    let zero_definition = definition
        .with_color_mixing_strength(0.0)
        .expect("zero color-mixing strength is valid");
    let mixed_specs = [
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::COLOR_MIXING).colored(first),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::COLOR_MIXING).colored(second),
        ParticleSpec::new(Vec2::ZERO, ParticleFlags::COLOR_MIXING)
            .colored(ParticleColor::new(128, 128, 128, 128)),
    ];
    let mut mixed = Fixture::new(&mixed_specs);
    mixed.contacts(&[
        (0, 1, ParticleFlags::COLOR_MIXING, 1.0, Vec2::new(1.0, 0.0)),
        (1, 2, ParticleFlags::COLOR_MIXING, 1.0, Vec2::new(1.0, 0.0)),
    ]);

    // Act
    color_mixing(&mut control.storage, definition).expect("control solve succeeds");
    color_mixing(&mut active.storage, definition).expect("active solve succeeds");
    color_mixing(&mut zero.storage, zero_definition).expect("zero solve succeeds");
    color_mixing(&mut mixed.storage, definition).expect("mixed solve succeeds");
    let mut repeated = Fixture::new(&mixed_specs);
    repeated.contacts(&[
        (0, 1, ParticleFlags::COLOR_MIXING, 1.0, Vec2::new(1.0, 0.0)),
        (1, 2, ParticleFlags::COLOR_MIXING, 1.0, Vec2::new(1.0, 0.0)),
    ]);
    color_mixing(&mut repeated.storage, definition).expect("repeated solve succeeds");

    // Assert
    assert_eq!(
        control.storage.maybe_colors(),
        Some([first, second].as_slice())
    );
    assert_eq!(
        zero.storage.maybe_colors(),
        Some([first, second].as_slice())
    );
    assert_eq!(
        active.storage.maybe_colors(),
        Some(
            [
                ParticleColor::new(191, 63, 70, 190),
                ParticleColor::new(64, 192, 190, 70),
            ]
            .as_slice()
        )
    );
    assert_eq!(
        mixed.storage.maybe_colors(),
        repeated.storage.maybe_colors()
    );
}

#[test]
fn manifest_admits_every_material_kernel_once_in_exact_order() {
    // Arrange
    use crate::particle::solver::PassId;
    use crate::particle::solver::manifest::PASS_GRAPH;

    let expected = [
        PassId::Viscous,
        PassId::Repulsive,
        PassId::Powder,
        PassId::Tensile,
        PassId::Solid,
        PassId::ColorMixing,
    ];

    // Act
    let actual = PASS_GRAPH[11..17]
        .iter()
        .map(|descriptor| descriptor.id)
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(actual, expected);
}
