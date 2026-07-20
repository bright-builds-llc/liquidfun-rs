use crate::identity::{
    BodyId, FixtureId, HandleIdentity, Identity, ParticleId, ParticleSystemId, WorldKey,
};
use crate::math::Vec2;
use crate::particle::storage::{ParticleInput, ParticleStorage};
use crate::particle::{
    ParticleBodyContact, ParticleContact, ParticleFlags, ParticleSystemDef, ParticleSystemDefError,
};

use super::*;

#[derive(Debug, Clone, Copy)]
struct FakeBody {
    id: BodyId,
    center: Vec2,
    linear_velocity: Vec2,
    angular_velocity: f32,
    inverse_mass: f32,
    inverse_inertia: f32,
}

#[derive(Debug, Clone, Default)]
struct FakeBodies {
    bodies: Vec<FakeBody>,
    impulses: Vec<(BodyId, Vec2, Vec2)>,
}

impl FakeBodies {
    fn with_body(id: BodyId, inverse_mass: f32, inverse_inertia: f32) -> Self {
        Self {
            bodies: vec![FakeBody {
                id,
                center: Vec2::ZERO,
                linear_velocity: Vec2::ZERO,
                angular_velocity: 0.0,
                inverse_mass,
                inverse_inertia,
            }],
            impulses: Vec::new(),
        }
    }

    fn body(&self, id: BodyId) -> FakeBody {
        *self
            .bodies
            .iter()
            .find(|body| body.id == id)
            .expect("test body must exist")
    }
}

impl BodyCoupling for FakeBodies {
    fn contains_body(&self, body: BodyId) -> bool {
        self.bodies.iter().any(|candidate| candidate.id == body)
    }

    fn velocity_at(&self, body: BodyId, point: Vec2) -> Vec2 {
        let body = self.body(body);
        body.linear_velocity + Vec2::scalar_cross(body.angular_velocity, point - body.center)
    }

    fn apply_linear_impulse(&mut self, body: BodyId, impulse: Vec2, point: Vec2) {
        let candidate = self
            .bodies
            .iter_mut()
            .find(|candidate| candidate.id == body)
            .expect("validated test body must exist");
        candidate.linear_velocity += candidate.inverse_mass * impulse;
        candidate.angular_velocity +=
            candidate.inverse_inertia * (point - candidate.center).cross(impulse);
        self.impulses.push((body, impulse, point));
    }
}

struct Fixture {
    world: WorldKey,
    storage: ParticleStorage,
    particles: Vec<ParticleId>,
}

impl Fixture {
    fn new(flags: &[ParticleFlags], velocities: &[Vec2]) -> Self {
        assert_eq!(flags.len(), velocities.len());
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        let capacity = flags.len().max(4);
        let mut storage = ParticleStorage::new(world, system, 8, capacity, capacity)
            .expect("test storage contract is valid");
        let mut particles = Vec::new();
        let mut position_x = 0.0;
        for (flags, velocity) in flags.iter().copied().zip(velocities.iter().copied()) {
            let position = Vec2::new(position_x, 0.0);
            particles.push(
                storage
                    .create(ParticleInput {
                        position,
                        velocity,
                        flags,
                        maybe_group: None,
                        maybe_color: None,
                        maybe_user_association: None,
                        maybe_expiration_time: None,
                    })
                    .expect("test particle fits"),
            );
            position_x += 1.0;
        }
        Self {
            world,
            storage,
            particles,
        }
    }

    fn body(&self, slot: usize) -> BodyId {
        BodyId::from_identity(Identity::new(self.world, slot, 0))
    }

    fn fixture(&self, slot: usize) -> FixtureId {
        FixtureId::from_identity(Identity::new(self.world, slot, 0))
    }

    fn set_particle_contacts(&mut self, contacts: &[(usize, usize, f32, Vec2)]) {
        let contacts = contacts
            .iter()
            .map(|&(a, b, weight, normal)| {
                ParticleContact::new_internal(
                    [self.particles[a], self.particles[b]],
                    self.storage.flags()[a] | self.storage.flags()[b],
                    weight,
                    normal,
                )
            })
            .collect::<Vec<_>>();
        self.storage
            .replace_particle_contacts(&contacts)
            .expect("test contacts use live particles");
    }

    fn set_body_contacts(&mut self, contacts: &[(usize, BodyId, FixtureId, f32, Vec2, f32)]) {
        let contacts = contacts
            .iter()
            .map(|&(particle, body, fixture, weight, normal, mass)| {
                ParticleBodyContact::new_internal(
                    self.particles[particle],
                    body,
                    fixture,
                    weight,
                    normal,
                    mass,
                )
            })
            .collect::<Vec<_>>();
        self.storage
            .replace_body_contacts(&contacts)
            .expect("test body contacts use live particles");
    }
}

fn fully_connected_contacts() -> [(usize, usize, f32, Vec2); 3] {
    [
        (0, 1, 0.75, Vec2::new(1.0, 0.0)),
        (0, 2, 0.75, Vec2::new(0.0, 1.0)),
        (1, 2, 0.75, Vec2::new(1.0, 0.0)),
    ]
}

fn vec_bits(values: &[Vec2]) -> Vec<[u32; 2]> {
    values
        .iter()
        .map(|value| [value.x.to_bits(), value.y.to_bits()])
        .collect()
}

#[test]
fn static_pressure_rejects_out_of_range_iterations_before_solver_effects() {
    // Arrange
    let fixture = Fixture::new(&[ParticleFlags::STATIC_PRESSURE], &[Vec2::new(2.0, 0.0)]);
    let before = fixture.storage.clone();

    // Act
    let zero = ParticleSystemDef::default().with_static_pressure_iterations(0);
    let excessive = ParticleSystemDef::default()
        .with_static_pressure_iterations(ParticleSystemDef::MAX_STATIC_PRESSURE_ITERATIONS + 1);

    // Assert
    assert_eq!(zero, Err(ParticleSystemDefError::ZeroIterations));
    assert_eq!(
        excessive,
        Err(ParticleSystemDefError::StaticPressureIterationsOutOfRange {
            requested: ParticleSystemDef::MAX_STATIC_PRESSURE_ITERATIONS + 1,
            maximum: ParticleSystemDef::MAX_STATIC_PRESSURE_ITERATIONS,
        })
    );
    assert!(fixture.storage == before);
}

#[test]
fn static_pressure_uses_configured_strength_relaxation_and_iterations() {
    // Arrange
    let flags = [ParticleFlags::STATIC_PRESSURE; 3];
    let velocities = [Vec2::ZERO; 3];
    let mut one_iteration = Fixture::new(&flags, &velocities);
    one_iteration.set_particle_contacts(&fully_connected_contacts());
    let mut two_iterations = Fixture::new(&flags, &velocities);
    two_iterations.set_particle_contacts(&fully_connected_contacts());
    let definition = ParticleSystemDef::default()
        .with_static_pressure_strength(0.3)
        .expect("configured strength is valid")
        .with_static_pressure_relaxation(0.4)
        .expect("configured relaxation is valid");
    let one_definition = definition
        .with_static_pressure_iterations(1)
        .expect("one iteration is valid");
    let two_definition = definition
        .with_static_pressure_iterations(2)
        .expect("two iterations are valid");

    // Act
    static_pressure(&mut one_iteration.storage, one_definition, 10.0)
        .expect("checked static-pressure inputs solve");
    static_pressure(&mut two_iterations.storage, two_definition, 10.0)
        .expect("checked static-pressure inputs solve");

    // Assert
    let first = one_iteration
        .storage
        .maybe_static_pressures()
        .expect("static-pressure scratch is allocated");
    let second = two_iterations
        .storage
        .maybe_static_pressures()
        .expect("static-pressure scratch is allocated");
    assert!(
        first
            .iter()
            .all(|pressure| pressure.to_bits() == first[0].to_bits())
    );
    assert!(
        second
            .iter()
            .all(|pressure| pressure.to_bits() == second[0].to_bits())
    );
    assert_ne!(first[0].to_bits(), second[0].to_bits());
    assert!(second[0] > first[0]);
}

#[test]
fn static_pressure_zero_strength_and_no_contacts_are_exact_controls() {
    // Arrange
    let mut fixture = Fixture::new(&[ParticleFlags::STATIC_PRESSURE], &[Vec2::new(3.0, -4.0)]);
    let before_velocity = vec_bits(fixture.storage.velocities());
    let definition = ParticleSystemDef::default()
        .with_static_pressure_strength(0.0)
        .expect("zero strength is valid");

    // Act
    static_pressure(&mut fixture.storage, definition, 10.0)
        .expect("isolated checked particle solves");

    // Assert
    assert_eq!(
        fixture
            .storage
            .maybe_static_pressures()
            .expect("static-pressure scratch is allocated")[0]
            .to_bits(),
        0.0_f32.to_bits()
    );
    assert_eq!(vec_bits(fixture.storage.velocities()), before_velocity);
}

#[test]
fn pressure_uses_configured_coefficient_and_suppresses_powder_and_tensile() {
    // Arrange
    let flags = [
        ParticleFlags::WATER,
        ParticleFlags::POWDER,
        ParticleFlags::TENSILE,
    ];
    let mut fixture = Fixture::new(&flags, &[Vec2::ZERO; 3]);
    fixture.set_particle_contacts(&fully_connected_contacts());
    let definition = ParticleSystemDef::default()
        .with_pressure_strength(0.125)
        .expect("configured pressure strength is valid");
    let mut bodies = FakeBodies::default();

    // Act
    pressure(&mut fixture.storage, definition, 0.1, 10.0, &mut bodies)
        .expect("particle-only pressure solve succeeds");

    // Assert
    let velocities = fixture.storage.velocities();
    let velocity_per_pressure = 0.1_f32 / (1.0 * 2.0);
    let expected_impulse = velocity_per_pressure * 0.75 * 25.0;
    assert_eq!(velocities[0].x.to_bits(), (-expected_impulse).to_bits());
    assert_eq!(velocities[0].y.to_bits(), (-expected_impulse).to_bits());
    assert_eq!(velocities[1].x.to_bits(), expected_impulse.to_bits());
    assert_eq!(velocities[1].y.to_bits(), 0.0_f32.to_bits());
    assert_eq!(velocities[2].x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(velocities[2].y.to_bits(), expected_impulse.to_bits());
}

#[test]
fn pressure_zero_coefficient_is_byte_identical_without_static_pressure() {
    // Arrange
    let mut fixture = Fixture::new(
        &[ParticleFlags::WATER, ParticleFlags::WATER],
        &[Vec2::new(1.0, 2.0), Vec2::new(-3.0, 4.0)],
    );
    fixture.set_particle_contacts(&[(0, 1, 0.75, Vec2::new(1.0, 0.0))]);
    let before = vec_bits(fixture.storage.velocities());
    let definition = ParticleSystemDef::default()
        .with_pressure_strength(0.0)
        .expect("zero pressure strength is valid");
    let mut bodies = FakeBodies::default();

    // Act
    pressure(&mut fixture.storage, definition, 0.1, 10.0, &mut bodies)
        .expect("zero-strength pressure solve succeeds");

    // Assert
    assert_eq!(vec_bits(fixture.storage.velocities()), before);
    assert!(bodies.impulses.is_empty());
}

#[test]
fn pressure_traverses_body_contacts_before_particle_contacts() {
    // Arrange
    let mut fixture = Fixture::new(
        &[ParticleFlags::WATER, ParticleFlags::WATER],
        &[Vec2::ZERO, Vec2::ZERO],
    );
    let body = fixture.body(20);
    let body_fixture = fixture.fixture(30);
    fixture.set_particle_contacts(&[(0, 1, 0.75, Vec2::new(1.0, 0.0))]);
    fixture.set_body_contacts(&[(0, body, body_fixture, 0.5, Vec2::new(1.0, 0.0), 2.0)]);
    let definition = ParticleSystemDef::default()
        .with_pressure_strength(0.125)
        .expect("configured pressure strength is valid");
    let mut bodies = FakeBodies::with_body(body, 1.0, 0.0);

    // Act
    pressure(&mut fixture.storage, definition, 0.1, 10.0, &mut bodies)
        .expect("body-coupled pressure solve succeeds");

    // Assert
    assert_eq!(bodies.impulses.len(), 1);
    let velocity_per_pressure = 0.1_f32 / (1.0 * 2.0);
    let dynamic_pressure = 50.0 * (1.25 - settings::MIN_PARTICLE_WEIGHT);
    let contact_pressure = dynamic_pressure + 50.0 * 0.5;
    let expected_impulse = velocity_per_pressure * 0.5 * 2.0 * contact_pressure;
    assert_eq!(bodies.impulses[0].1.x.to_bits(), expected_impulse.to_bits());
    assert_eq!(bodies.impulses[0].1.y.to_bits(), 0.0_f32.to_bits());
    assert!(fixture.storage.velocities()[0].x < 0.0);
    assert!(fixture.storage.velocities()[1].x > 0.0);
}

#[test]
fn damping_preserves_particle_contact_traversal_order() {
    // Arrange
    let flags = [ParticleFlags::WATER; 3];
    let velocities = [Vec2::new(3.0, 0.0), Vec2::new(1.0, 0.0), Vec2::ZERO];
    let first_order = [
        (0, 1, 0.5, Vec2::new(1.0, 0.0)),
        (1, 2, 0.5, Vec2::new(1.0, 0.0)),
    ];
    let reverse_order = [first_order[1], first_order[0]];
    let mut forward = Fixture::new(&flags, &velocities);
    forward.set_particle_contacts(&first_order);
    let mut repeated = Fixture::new(&flags, &velocities);
    repeated.set_particle_contacts(&first_order);
    let mut reversed = Fixture::new(&flags, &velocities);
    reversed.set_particle_contacts(&reverse_order);

    // Act
    damping(
        &mut forward.storage,
        ParticleSystemDef::default(),
        10.0,
        &mut FakeBodies::default(),
    )
    .expect("forward damping solve succeeds");
    damping(
        &mut repeated.storage,
        ParticleSystemDef::default(),
        10.0,
        &mut FakeBodies::default(),
    )
    .expect("repeated damping solve succeeds");
    damping(
        &mut reversed.storage,
        ParticleSystemDef::default(),
        10.0,
        &mut FakeBodies::default(),
    )
    .expect("reversed damping solve succeeds");

    // Assert
    let forward_bits = vec_bits(forward.storage.velocities());
    assert_eq!(forward_bits, vec_bits(repeated.storage.velocities()));
    assert_ne!(forward_bits, vec_bits(reversed.storage.velocities()));
    assert_eq!(
        forward.storage.velocities()[0].x.to_bits(),
        2.0_f32.to_bits()
    );
    assert_eq!(
        forward.storage.velocities()[1].x.to_bits(),
        1.0_f32.to_bits()
    );
    assert_eq!(
        forward.storage.velocities()[2].x.to_bits(),
        1.0_f32.to_bits()
    );
}

#[test]
fn damping_preserves_sequential_body_contact_updates() {
    // Arrange
    let flags = [ParticleFlags::WATER; 2];
    let velocities = [Vec2::new(2.0, 0.0), Vec2::new(1.0, 0.0)];
    let mut fixture = Fixture::new(&flags, &velocities);
    let body = fixture.body(20);
    fixture.set_body_contacts(&[
        (0, body, fixture.fixture(30), 0.5, Vec2::new(1.0, 0.0), 1.0),
        (1, body, fixture.fixture(31), 0.5, Vec2::new(1.0, 0.0), 1.0),
    ]);
    let mut bodies = FakeBodies::with_body(body, 1.0, 0.0);

    // Act
    damping(
        &mut fixture.storage,
        ParticleSystemDef::default(),
        10.0,
        &mut bodies,
    )
    .expect("body damping solve succeeds");

    // Assert
    assert_eq!(bodies.impulses.len(), 1);
    assert_eq!(
        bodies.body(body).linear_velocity.x.to_bits(),
        1.0_f32.to_bits()
    );
    assert_eq!(
        fixture.storage.velocities()[1].x.to_bits(),
        1.0_f32.to_bits()
    );
}

#[test]
fn extra_damping_checks_each_particle_gate_independently() {
    // Arrange
    let flags = [
        ParticleFlags::STATIC_PRESSURE,
        ParticleFlags::WATER,
        ParticleFlags::STATIC_PRESSURE,
    ];
    let velocities = [
        Vec2::new(2.0, 0.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(-1.0, 0.0),
    ];
    let mut fixture = Fixture::new(&flags, &velocities);
    let body = fixture.body(20);
    fixture.set_body_contacts(&[
        (0, body, fixture.fixture(30), 0.5, Vec2::new(1.0, 0.0), 1.0),
        (1, body, fixture.fixture(31), 0.5, Vec2::new(1.0, 0.0), 1.0),
        (2, body, fixture.fixture(32), 0.5, Vec2::new(1.0, 0.0), 1.0),
    ]);
    let before = vec_bits(fixture.storage.velocities());
    let mut bodies = FakeBodies::with_body(body, 0.0, 0.0);

    // Act
    extra_damping(
        &mut fixture.storage,
        ParticleSystemDef::default(),
        &mut bodies,
    )
    .expect("extra damping solve succeeds");

    // Assert
    assert_ne!(fixture.storage.velocities()[0].x.to_bits(), before[0][0]);
    assert_eq!(fixture.storage.velocities()[1].x.to_bits(), before[1][0]);
    assert_eq!(fixture.storage.velocities()[2].x.to_bits(), before[2][0]);
    assert_eq!(bodies.impulses.len(), 1);
}

#[test]
fn empty_and_no_contact_pressure_family_controls_are_byte_identical() {
    // Arrange
    let mut empty = Fixture::new(&[], &[]);
    let mut isolated = Fixture::new(&[ParticleFlags::WATER], &[Vec2::new(3.0, -4.0)]);
    let empty_before = vec_bits(empty.storage.velocities());
    let isolated_before = vec_bits(isolated.storage.velocities());
    let mut bodies = FakeBodies::default();

    // Act
    pressure(
        &mut empty.storage,
        ParticleSystemDef::default(),
        0.1,
        10.0,
        &mut bodies,
    )
    .expect("empty pressure is a no-op");
    damping(
        &mut empty.storage,
        ParticleSystemDef::default(),
        10.0,
        &mut bodies,
    )
    .expect("empty damping is a no-op");
    extra_damping(
        &mut empty.storage,
        ParticleSystemDef::default(),
        &mut bodies,
    )
    .expect("empty extra damping is a no-op");
    pressure(
        &mut isolated.storage,
        ParticleSystemDef::default(),
        0.1,
        10.0,
        &mut bodies,
    )
    .expect("isolated pressure is a no-op");
    damping(
        &mut isolated.storage,
        ParticleSystemDef::default(),
        10.0,
        &mut bodies,
    )
    .expect("isolated damping is a no-op");
    extra_damping(
        &mut isolated.storage,
        ParticleSystemDef::default(),
        &mut bodies,
    )
    .expect("isolated extra damping is a no-op");

    // Assert
    assert_eq!(vec_bits(empty.storage.velocities()), empty_before);
    assert_eq!(vec_bits(isolated.storage.velocities()), isolated_before);
    assert!(bodies.impulses.is_empty());
}

#[test]
fn missing_body_fails_before_particle_velocity_mutation() {
    // Arrange
    let mut fixture = Fixture::new(&[ParticleFlags::WATER], &[Vec2::new(2.0, 0.0)]);
    let missing_body = fixture.body(20);
    fixture.set_body_contacts(&[(
        0,
        missing_body,
        fixture.fixture(30),
        0.5,
        Vec2::new(1.0, 0.0),
        1.0,
    )]);
    let before = vec_bits(fixture.storage.velocities());

    // Act
    let result = damping(
        &mut fixture.storage,
        ParticleSystemDef::default(),
        10.0,
        &mut FakeBodies::default(),
    );

    // Assert
    assert_eq!(result, Err(PressureSolverError::MissingBody(missing_body)));
    assert_eq!(vec_bits(fixture.storage.velocities()), before);
}
