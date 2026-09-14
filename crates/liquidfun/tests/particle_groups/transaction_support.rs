use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleGroupDestination, ParticleGroupFlags, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    BodyDef, FixtureDef, HandleError, LifecycleEvent, NoDecisionHook, ParticleColor, ParticleFlags,
    ParticleGroupId, ParticleId, ParticleSystemId, ParticleSystemSnapshot,
    ParticleSystemStatistics, ParticleWorldStatistics, StepConfiguration, StepLimits, World,
};

#[derive(Debug, PartialEq)]
pub(super) struct Snapshot {
    rows: Vec<Row>,
    groups: Vec<(ParticleGroupId, Result<Group, HandleError>)>,
    pairs: Vec<([ParticleId; 2], u32, [u32; 2])>,
    triads: Vec<([ParticleId; 3], u32, [u32; 11])>,
    contacts: Vec<([ParticleId; 2], u32, u32, [u32; 2])>,
    body_contacts: Vec<(
        ParticleId,
        liquidfun::BodyId,
        liquidfun::FixtureId,
        [u32; 4],
    )>,
    maybe_colors: Option<Vec<[u8; 4]>>,
    maybe_expiration_order: Option<Vec<ParticleId>>,
    system: ParticleSystemSnapshot,
    statistics: ParticleSystemStatistics,
    world_statistics: ParticleWorldStatistics,
}

#[derive(Debug, PartialEq)]
struct Row {
    id: ParticleId,
    maybe_group: Option<ParticleGroupId>,
    position: [u32; 2],
    velocity: [u32; 2],
    force: [u32; 2],
    weight: u32,
    flags: u32,
    snapshot: Result<liquidfun::ParticleSnapshot, HandleError>,
}

#[derive(Debug, PartialEq)]
struct Group {
    id: ParticleGroupId,
    flags: u32,
    members: Vec<ParticleId>,
    transform: [u32; 4],
    angle: u32,
    center: [u32; 2],
    velocity: [u32; 2],
    angular_velocity: u32,
    mass: u32,
    inertia: u32,
    maybe_depths: Option<Vec<u32>>,
}

fn bits(vector: Vec2) -> [u32; 2] {
    [vector.x.to_bits(), vector.y.to_bits()]
}

pub(super) fn snapshot(
    world: &World,
    system: ParticleSystemId,
    groups: &[ParticleGroupId],
) -> Snapshot {
    let view = world.particle_system_view(system).expect("live system");
    Snapshot {
        rows: view
            .particle_ids()
            .iter()
            .copied()
            .enumerate()
            .map(|(index, id)| Row {
                id,
                maybe_group: view.group_ids()[index],
                position: bits(view.positions()[index]),
                velocity: bits(view.velocities()[index]),
                force: bits(view.forces()[index]),
                weight: view.weights()[index].to_bits(),
                flags: view.flags()[index].bits(),
                snapshot: world.particle_snapshot(id),
            })
            .collect(),
        groups: groups
            .iter()
            .copied()
            .map(|id| (id, snapshot_group(world, id)))
            .collect(),
        pairs: view
            .pairs()
            .map(|pair| {
                (
                    pair.particles(),
                    pair.flags().bits(),
                    [pair.strength().to_bits(), pair.distance().to_bits()],
                )
            })
            .collect(),
        triads: view
            .triads()
            .map(|triad| {
                (
                    triad.particles(),
                    triad.flags().bits(),
                    [
                        triad.strength().to_bits(),
                        triad.pa().x.to_bits(),
                        triad.pa().y.to_bits(),
                        triad.pb().x.to_bits(),
                        triad.pb().y.to_bits(),
                        triad.pc().x.to_bits(),
                        triad.pc().y.to_bits(),
                        triad.ka().to_bits(),
                        triad.kb().to_bits(),
                        triad.kc().to_bits(),
                        triad.s().to_bits(),
                    ],
                )
            })
            .collect(),
        contacts: view
            .particle_contacts()
            .map(|contact| {
                (
                    contact.particles(),
                    contact.flags().bits(),
                    contact.weight().to_bits(),
                    bits(contact.normal()),
                )
            })
            .collect(),
        body_contacts: view
            .body_contacts()
            .map(|contact| {
                (
                    contact.particle(),
                    contact.body(),
                    contact.fixture(),
                    [
                        contact.weight().to_bits(),
                        contact.normal().x.to_bits(),
                        contact.normal().y.to_bits(),
                        contact.mass().to_bits(),
                    ],
                )
            })
            .collect(),
        maybe_colors: view
            .maybe_colors()
            .map(|colors| colors.iter().map(|color| color.components()).collect()),
        maybe_expiration_order: view.maybe_expiration_order().map(Iterator::collect),
        system: world.particle_system_snapshot(system).expect("live system"),
        statistics: world
            .particle_system_statistics(system)
            .expect("live system"),
        world_statistics: world.particle_world_statistics(),
    }
}

pub(super) fn recipe(
    positions: Vec<Vec2>,
    destination: ParticleGroupDestination,
) -> ParticleGroupRecipe {
    ParticleGroupRecipe::new(
        ParticleGroupSource::positions(positions).expect("finite positions"),
        destination,
    )
}

pub(super) struct Fixture {
    pub(super) world: World,
    pub(super) system: ParticleSystemId,
    pub(super) target: ParticleGroupId,
    pub(super) pending: ParticleId,
    pub(super) stale: ParticleGroupId,
    pub(super) groups: Vec<ParticleGroupId>,
}

impl Fixture {
    pub(super) fn new() -> Self {
        Self::with_capacity(128)
    }

    pub(super) fn with_capacity(capacity: usize) -> Self {
        let mut world = World::new().expect("world key available");
        world.set_gravity(Vec2::ZERO).expect("finite gravity");
        let body = world.create_body(&BodyDef::default()).expect("body fits");
        let shape = Shape::Circle(CircleShape::new(Vec2::ZERO, 1.0).expect("valid circle"));
        let definition = FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
            .expect("valid fixture");
        world
            .create_fixture(body, &definition)
            .expect("fixture fits");
        let system_definition = liquidfun::ParticleSystemDef::default()
            .with_capacity(liquidfun::ParticleCapacity::fixed(capacity).expect("positive capacity"))
            .expect("valid capacity");
        let system = world
            .create_particle_system_with_def(&system_definition)
            .expect("system fits");
        let target = world
            .create_particle_group(
                system,
                &recipe(
                    vec![Vec2::ZERO, Vec2::new(1.5, 0.0), Vec2::new(0.75, 1.3)],
                    ParticleGroupDestination::New,
                )
                .with_group_flags(ParticleGroupFlags::SOLID | ParticleGroupFlags::CAN_BE_EMPTY)
                .with_particle_flags(
                    ParticleFlags::SPRING | ParticleFlags::ELASTIC | ParticleFlags::WALL,
                )
                .with_color(ParticleColor::new(1, 2, 3, 4))
                .with_lifetime(10.0)
                .expect("bounded lifetime"),
            )
            .expect("populated group fits");
        let pending_group = world
            .create_particle_group(
                system,
                &recipe(vec![Vec2::new(20.0, 0.0)], ParticleGroupDestination::New)
                    .with_group_flags(ParticleGroupFlags::CAN_BE_EMPTY)
                    .with_particle_flags(ParticleFlags::DESTRUCTION_LISTENER),
            )
            .expect("listener group fits");
        let pending = world
            .particle_group_view(pending_group)
            .expect("live listener group")
            .member_ids()[0];
        let stale = world
            .create_particle_group(
                system,
                &recipe(vec![Vec2::new(40.0, 0.0)], ParticleGroupDestination::New),
            )
            .expect("temporary group fits");
        let stale_member = world
            .particle_group_view(stale)
            .expect("temporary group live")
            .member_ids()[0];
        world
            .destroy_particle(stale_member)
            .expect("temporary member removed");
        world
            .destroy_empty_particle_group(stale)
            .expect("empty shell removed");
        assert!(step(&mut world).is_empty());
        world
            .apply_particle_force(pending, Vec2::new(0.25, -0.5))
            .expect("live particle accepts finite force");
        world
            .destroy_particle_group_particles(pending_group, true)
            .expect("mark pending member");
        let fixture = Self {
            world,
            system,
            target,
            pending,
            stale,
            groups: vec![target, pending_group, stale],
        };
        fixture.assert_populated();
        fixture
    }

    pub(super) fn snapshot(&self) -> Snapshot {
        snapshot(&self.world, self.system, &self.groups)
    }

    fn assert_populated(&self) {
        assert_eq!(
            self.world
                .particle_group_view(self.stale)
                .map(|view| view.id()),
            Err(HandleError::StaleOrDestroyed)
        );
        let view = self
            .world
            .particle_system_view(self.system)
            .expect("live system");
        assert!(view.pairs().len() > 0);
        assert!(view.triads().len() > 0);
        assert!(view.particle_contacts().len() > 0);
        assert!(view.body_contacts().len() > 0);
        assert!(view.maybe_colors().is_some());
        assert!(view.maybe_expiration_order().is_some());
        assert!(view.weights().iter().any(|weight| *weight > 0.0));
        assert!(view.forces().iter().any(|force| *force != Vec2::ZERO));
        assert!(
            view.flags()
                .iter()
                .any(|flags| flags.contains(ParticleFlags::ZOMBIE))
        );
        assert_eq!(
            self.world.particle_snapshot(self.pending),
            Err(HandleError::PendingDelete)
        );
        assert!(
            self.world
                .particle_group_view(self.target)
                .expect("live target")
                .maybe_depths()
                .is_some()
        );
    }

    pub(super) fn assert_deferred_lifecycle(&mut self) {
        let events = step(&mut self.world);
        assert_eq!(events.len(), 1);
        let [LifecycleEvent::ParticleDestruction(destroyed)] = events.as_slice() else {
            panic!("expected exactly the pre-existing pending particle listener: {events:?}");
        };
        assert_eq!(destroyed.cause(), liquidfun::DestructionCause::Explicit);
        assert_eq!(
            destroyed.destroyed(),
            liquidfun::DestroyedId::Particle(self.pending)
        );
        assert_eq!(
            destroyed.snapshot(),
            &liquidfun::ObjectSnapshot::Particle {
                system: self.system,
                maybe_group: Some(self.groups[1])
            }
        );
        assert_eq!(
            self.world.particle_snapshot(self.pending),
            Err(HandleError::StaleOrDestroyed)
        );
        assert!(step(&mut self.world).is_empty());
    }
}

pub(super) fn step(world: &mut World) -> Vec<LifecycleEvent> {
    world
        .step(
            StepConfiguration::new(1.0 / 60.0, 8, 3).expect("valid step"),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("fixture steps")
        .lifecycle()
        .to_vec()
}

fn snapshot_group(world: &World, id: ParticleGroupId) -> Result<Group, HandleError> {
    world.particle_group_view(id).map(|group| Group {
        id: group.id(),
        flags: group.flags().bits(),
        members: group.member_ids().to_vec(),
        transform: [
            group.position().x.to_bits(),
            group.position().y.to_bits(),
            group.transform().rotation().sine().to_bits(),
            group.transform().rotation().cosine().to_bits(),
        ],
        angle: group.angle().to_bits(),
        center: bits(group.center()),
        velocity: bits(group.linear_velocity()),
        angular_velocity: group.angular_velocity().to_bits(),
        mass: group.mass().to_bits(),
        inertia: group.inertia().to_bits(),
        maybe_depths: group
            .maybe_depths()
            .map(|depths| depths.iter().map(|value| value.to_bits()).collect()),
    })
}
