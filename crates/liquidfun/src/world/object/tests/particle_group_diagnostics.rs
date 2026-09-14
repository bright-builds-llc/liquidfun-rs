// Temporary exact-input replay; mapped against the integration Model in 14-DIAGNOSIS.md.
use crate::collision::{CircleShape, EdgeShape, Shape};
use crate::math::Vec2;
use crate::particle::storage::diagnostic_record;
use crate::particle::{
    ParticleGroupDestination, ParticleGroupFlags, ParticleGroupRecipe, ParticleGroupSource,
};
use crate::{
    HandleError, LifecycleEvent, NoDecisionHook, ParticleFlags, ParticleGroupId,
    ParticleGroupMutationError, ParticleSystemId, StepConfiguration, StepLimits, World,
};

pub(super) struct Model {
    pub(super) world: World,
    pub(super) system: ParticleSystemId,
    foreign_group: ParticleGroupId,
    pub(super) known_groups: Vec<ParticleGroupId>,
}

impl Model {
    pub(super) fn new() -> Self {
        diagnostic_record(format_args!("init.before_world"));
        let mut world = World::new().expect("world key remains available");
        diagnostic_record(format_args!("init.after_world before_gravity"));
        world
            .set_gravity(Vec2::ZERO)
            .expect("zero gravity is valid");
        diagnostic_record(format_args!("init.before_primary_system"));
        let system = world.create_particle_system().expect("system should fit");
        diagnostic_record(format_args!(
            "init.after_primary_system before_foreign_system"
        ));
        let foreign_system = world
            .create_particle_system()
            .expect("foreign system should fit");
        diagnostic_record(format_args!(
            "init.after_foreign_system before_foreign_group"
        ));
        let foreign_group = world
            .create_particle_group(
                foreign_system,
                &positions_recipe(
                    vec![Vec2::new(100.0, 100.0)],
                    ParticleGroupDestination::New,
                    ParticleFlags::WATER,
                    ParticleGroupFlags::empty(),
                ),
            )
            .expect("foreign control group should fit");
        diagnostic_record(format_args!("init.after_foreign_group"));
        Self {
            world,
            system,
            foreign_group,
            known_groups: Vec::new(),
        }
    }

    pub(super) fn live_groups(&self) -> Vec<ParticleGroupId> {
        self.known_groups
            .iter()
            .copied()
            .filter(|group| self.world.particle_group_view(*group).is_ok())
            .collect()
    }

    fn remember(&mut self, groups: impl IntoIterator<Item = ParticleGroupId>) -> usize {
        let mut added = 0;
        for group in groups {
            if !self.known_groups.contains(&group) {
                self.known_groups.push(group);
                added += 1;
            }
        }
        added
    }

    fn particle_count(&self) -> usize {
        self.world
            .particle_system_statistics(self.system)
            .expect("model system remains live")
            .particle_count()
    }

    fn apply(&mut self, operation: Operation) -> Outcome {
        let outcome = self.apply_inner(operation);
        diagnostic_record(format_args!("public outcome={outcome:?}"));
        for (_, system) in self.world.particle_systems.iter() {
            system.storage.diagnostic_observe("public.after");
        }
        outcome
    }

    fn apply_inner(&mut self, operation: Operation) -> Outcome {
        match operation.kind {
            OperationKind::CreateExplicit => self.create_explicit(operation.first),
            OperationKind::CreateFilled => self.create_filled(operation.first),
            OperationKind::CreateStroke => self.create_stroke(operation.first),
            OperationKind::Append => self.append(operation.first),
            OperationKind::Join => self.join(operation.first, operation.second),
            OperationKind::Split => self.split(operation.first),
            OperationKind::SetFlags => self.set_flags(operation.first, operation.second),
            OperationKind::CreateReactive => self.create_reactive(operation.first),
            OperationKind::CreateLifetime => self.create_lifetime(operation.first),
            OperationKind::DestroyMembers => self.destroy_members(operation.first),
            OperationKind::Compact => self.compact(),
            OperationKind::Step => self.step(),
            OperationKind::InvalidJoin => self.invalid_join(operation.first),
        }
    }

    fn at_bound(&self, additional_particles: usize) -> bool {
        self.live_groups().len() >= MAX_GROUPS
            || self.particle_count().saturating_add(additional_particles) > MAX_PARTICLES
    }

    fn create_explicit(&mut self, selector: usize) -> Outcome {
        if self.at_bound(3) {
            return Outcome::SkippedAtBound;
        }
        let base = bounded_coordinate(selector);
        let flags = if selector & 1 == 0 {
            ParticleGroupFlags::CAN_BE_EMPTY
        } else {
            ParticleGroupFlags::empty()
        };
        let recipe = positions_recipe(
            vec![base, base + Vec2::new(0.4, 0.0), base + Vec2::new(3.0, 0.0)],
            ParticleGroupDestination::New,
            ParticleFlags::WATER,
            flags,
        );
        self.create(&recipe)
    }

    fn create_filled(&mut self, selector: usize) -> Outcome {
        if self.at_bound(8) {
            return Outcome::SkippedAtBound;
        }
        let source = ParticleGroupSource::filled_shapes(vec![Shape::Circle(
            CircleShape::new(bounded_coordinate(selector), 0.45).expect("circle is valid"),
        )])
        .expect("filled source is valid");
        let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
            .with_stride(0.4)
            .expect("stride is valid");
        self.create(&recipe)
    }

    fn create_stroke(&mut self, selector: usize) -> Outcome {
        if self.at_bound(5) {
            return Outcome::SkippedAtBound;
        }
        let base = bounded_coordinate(selector);
        let source = ParticleGroupSource::stroke_shape(Shape::Edge(
            EdgeShape::new(base, base + Vec2::new(0.8, 0.0)).expect("edge is valid"),
        ))
        .expect("stroke source is valid");
        let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
            .with_stride(0.3)
            .expect("stride is valid");
        self.create(&recipe)
    }

    fn create_reactive(&mut self, selector: usize) -> Outcome {
        if self.at_bound(3) {
            return Outcome::SkippedAtBound;
        }
        let base = bounded_coordinate(selector);
        let recipe = positions_recipe(
            vec![
                base,
                base + Vec2::new(0.4, 0.0),
                base + Vec2::new(0.2, 0.35),
            ],
            ParticleGroupDestination::New,
            ParticleFlags::REACTIVE | ParticleFlags::SPRING | ParticleFlags::ELASTIC,
            ParticleGroupFlags::SOLID,
        );
        self.create(&recipe)
    }

    fn create_lifetime(&mut self, selector: usize) -> Outcome {
        if self.at_bound(1) {
            return Outcome::SkippedAtBound;
        }
        let recipe = positions_recipe(
            vec![bounded_coordinate(selector)],
            ParticleGroupDestination::New,
            ParticleFlags::DESTRUCTION_LISTENER,
            ParticleGroupFlags::CAN_BE_EMPTY,
        )
        .with_lifetime(0.001)
        .expect("lifetime is finite");
        self.create(&recipe)
    }

    fn create(&mut self, recipe: &ParticleGroupRecipe) -> Outcome {
        match self.world.create_particle_group(self.system, recipe) {
            Ok(group) => Outcome::Applied {
                created: self.remember([group]),
                lifecycle: 0,
            },
            Err(error) => {
                diagnostic_record(format_args!("public typed rejection={error:?}"));
                Outcome::Rejected
            }
        }
    }

    fn record_lifecycle(&mut self, events: &[LifecycleEvent]) {
        diagnostic_record(format_args!(
            "lifecycle primary={:?} events={events:?}",
            self.system
        ));
    }

    fn append(&mut self, selector: usize) -> Outcome {
        if self.at_bound(1) {
            return Outcome::SkippedAtBound;
        }
        let live = self.live_groups();
        let Some(target) = select(&live, selector) else {
            return Outcome::SkippedAtBound;
        };
        let position = self
            .world
            .particle_group_view(target)
            .expect("selected group remains live")
            .member_ids()
            .last()
            .and_then(|particle| self.world.particle_snapshot(*particle).ok())
            .map_or(Vec2::ZERO, |snapshot| {
                snapshot.position() + Vec2::new(0.3, 0.0)
            });
        let recipe = positions_recipe(
            vec![position],
            ParticleGroupDestination::AppendTo(target),
            ParticleFlags::WATER,
            ParticleGroupFlags::empty(),
        );
        match self.world.create_particle_group(self.system, &recipe) {
            Ok(returned) => {
                assert_eq!(returned, target);
                Outcome::Applied {
                    created: self.remember([returned]),
                    lifecycle: 0,
                }
            }
            Err(error) => {
                diagnostic_record(format_args!("public typed rejection={error:?}"));
                Outcome::Rejected
            }
        }
    }

    fn join(&mut self, first: usize, second: usize) -> Outcome {
        let live = self.live_groups();
        let (Some(group_a), Some(mut group_b)) = (select(&live, first), select(&live, second))
        else {
            return Outcome::SkippedAtBound;
        };
        if group_a == group_b {
            let Some(replacement) = live.iter().copied().find(|group| *group != group_a) else {
                return Outcome::SkippedAtBound;
            };
            group_b = replacement;
        }
        match self.world.join_particle_groups(group_a, group_b) {
            Ok(report) => {
                assert_eq!(*report.value(), group_a);
                self.record_lifecycle(report.lifecycle());
                Outcome::Applied {
                    created: 0,
                    lifecycle: report.lifecycle().len(),
                }
            }
            Err(error) => {
                diagnostic_record(format_args!("public typed rejection={error:?}"));
                Outcome::Rejected
            }
        }
    }

    fn split(&mut self, selector: usize) -> Outcome {
        let live = self.live_groups();
        let Some(group) = select(&live, selector) else {
            return Outcome::SkippedAtBound;
        };
        let maximum_new_groups = self
            .world
            .particle_group_view(group)
            .expect("selected group remains live")
            .member_count()
            .saturating_sub(1);
        if live.len().saturating_add(maximum_new_groups) > MAX_GROUPS {
            return Outcome::SkippedAtBound;
        }
        match self.world.split_particle_group(group) {
            Ok(groups) => Outcome::Applied {
                created: self.remember(groups),
                lifecycle: 0,
            },
            Err(error) => {
                diagnostic_record(format_args!("public typed rejection={error:?}"));
                Outcome::Rejected
            }
        }
    }

    fn set_flags(&mut self, selector: usize, bits: usize) -> Outcome {
        let live = self.live_groups();
        let Some(group) = select(&live, selector) else {
            return Outcome::SkippedAtBound;
        };
        let flags = match bits % 4 {
            0 => ParticleGroupFlags::empty(),
            1 => ParticleGroupFlags::SOLID,
            2 => ParticleGroupFlags::RIGID,
            _ => ParticleGroupFlags::SOLID | ParticleGroupFlags::CAN_BE_EMPTY,
        };
        match self.world.set_particle_group_flags(group, flags) {
            Ok(()) => Outcome::Applied {
                created: 0,
                lifecycle: 0,
            },
            Err(error) => {
                diagnostic_record(format_args!("public typed rejection={error:?}"));
                Outcome::Rejected
            }
        }
    }

    fn destroy_members(&mut self, selector: usize) -> Outcome {
        let live = self.live_groups();
        let Some(group) = select(&live, selector) else {
            return Outcome::SkippedAtBound;
        };
        match self.world.destroy_particle_group_particles(group, true) {
            Ok(()) => Outcome::Applied {
                created: 0,
                lifecycle: 0,
            },
            Err(error) => {
                diagnostic_record(format_args!("public typed rejection={error:?}"));
                Outcome::Rejected
            }
        }
    }

    fn compact(&mut self) -> Outcome {
        match self.world.compact_pending_particles(self.system) {
            Ok(report) => {
                self.record_lifecycle(report.lifecycle());
                Outcome::Applied {
                    created: 0,
                    lifecycle: report.lifecycle().len(),
                }
            }
            Err(error) => {
                diagnostic_record(format_args!("public typed rejection={error:?}"));
                Outcome::Rejected
            }
        }
    }

    fn step(&mut self) -> Outcome {
        let configuration = StepConfiguration::new(1.0 / 60.0, 8, 3)
            .expect("configuration is valid")
            .with_particle_iterations(2)
            .expect("iteration count is valid");
        match self
            .world
            .step(configuration, &mut NoDecisionHook, StepLimits::default())
        {
            Ok(report) => {
                self.record_lifecycle(report.lifecycle());
                Outcome::Applied {
                    created: 0,
                    lifecycle: report.lifecycle().len(),
                }
            }
            Err(error) => {
                diagnostic_record(format_args!("public typed rejection={error:?}"));
                Outcome::Rejected
            }
        }
    }

    fn invalid_join(&mut self, selector: usize) -> Outcome {
        let live = self.live_groups();
        let Some(group) = select(&live, selector) else {
            return Outcome::SkippedAtBound;
        };
        let result = self.world.join_particle_groups(group, self.foreign_group);
        assert_eq!(
            result,
            Err(ParticleGroupMutationError::InvalidHandle(
                HandleError::WrongParticleSystem
            ))
        );
        Outcome::Rejected
    }
}

fn positions_recipe(
    positions: Vec<Vec2>,
    destination: ParticleGroupDestination,
    particle_flags: ParticleFlags,
    group_flags: ParticleGroupFlags,
) -> ParticleGroupRecipe {
    let source = ParticleGroupSource::positions(positions).expect("positions are finite");
    ParticleGroupRecipe::new(source, destination)
        .with_particle_flags(particle_flags)
        .with_group_flags(group_flags)
}

fn bounded_coordinate(selector: usize) -> Vec2 {
    let bucket = u16::try_from(selector % 32).unwrap_or(0);
    Vec2::new(f32::from(bucket) * 4.0, 0.0)
}

fn select(values: &[ParticleGroupId], selector: usize) -> Option<ParticleGroupId> {
    values.get(selector % values.len().max(1)).copied()
}

const MAX_GROUPS: usize = 8;
const MAX_PARTICLES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationKind {
    CreateExplicit,
    CreateFilled,
    CreateStroke,
    Append,
    Join,
    Split,
    SetFlags,
    CreateReactive,
    CreateLifetime,
    DestroyMembers,
    Compact,
    Step,
    InvalidJoin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Operation {
    kind: OperationKind,
    first: usize,
    second: usize,
}

const AUDITED_WINDOWS_SEED: u64 = 0x3995_60c9_ead9_4a3f;
const AUDITED_WINDOWS_CONTROLS: [u8; 14] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 59];
const AUDITED_WINDOWS_OPERATIONS: [Operation; 14] = [
    Operation {
        kind: OperationKind::CreateExplicit,
        first: 71_787_583_439_247_902,
        second: 4_278_873_410,
    },
    Operation {
        kind: OperationKind::Append,
        first: 47_981_589_295_040_740,
        second: 2_859_925_585,
    },
    Operation {
        kind: OperationKind::CreateFilled,
        first: 56_436_091_559_000_168,
        second: 3_363_853_189,
    },
    Operation {
        kind: OperationKind::CreateStroke,
        first: 26_405_087_307_191_014,
        second: 1_573_865_849,
    },
    Operation {
        kind: OperationKind::CreateReactive,
        first: 33_181_658_083_951_976,
        second: 1_977_780_943,
    },
    Operation {
        kind: OperationKind::Join,
        first: 6_396_812_461_881_086,
        second: 381_279_734,
    },
    Operation {
        kind: OperationKind::Split,
        first: 10_357_184_829_122_123,
        second: 617_336_322,
    },
    Operation {
        kind: OperationKind::SetFlags,
        first: 5_749_946_986_639_520,
        second: 342_723_547,
    },
    Operation {
        kind: OperationKind::CreateLifetime,
        first: 25_775_792_647_897_003,
        second: 1_536_356_964,
    },
    Operation {
        kind: OperationKind::Step,
        first: 53_393_286_103_193_178,
        second: 3_182_487_851,
    },
    Operation {
        kind: OperationKind::DestroyMembers,
        first: 5_176_720_587_242_457,
        second: 308_556_591,
    },
    Operation {
        kind: OperationKind::Compact,
        first: 1_779_414_347_435_098,
        second: 106_061_360,
    },
    Operation {
        kind: OperationKind::InvalidJoin,
        first: 13_697_499_052_459_992,
        second: 816_434_565,
    },
    Operation {
        kind: OperationKind::Append,
        first: 31_141_724_268_561_272,
        second: 1_856_191_412,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Applied { created: usize, lifecycle: usize },
    Rejected,
    SkippedAtBound,
}

#[test]
fn audited_windows_transition_trace() {
    // Arrange
    crate::particle::storage::diagnostic_enable();
    diagnostic_record(format_args!(
        "seed={AUDITED_WINDOWS_SEED} controls={AUDITED_WINDOWS_CONTROLS:?}"
    ));
    let mut model = Model::new();

    // Act
    for (ordinal, operation) in AUDITED_WINDOWS_OPERATIONS.into_iter().enumerate() {
        diagnostic_record(format_args!(
            "operation.before ordinal={} input={operation:?}",
            ordinal + 1
        ));
        model.apply(operation);
        diagnostic_record(format_args!("operation.after ordinal={}", ordinal + 1));
    }

    // Assert
    assert_eq!(AUDITED_WINDOWS_OPERATIONS.len(), 14);
    crate::particle::storage::diagnostic_disable();
}
