use liquidfun::collision::{CircleShape, EdgeShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleGroupDestination, ParticleGroupFlags, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    CreateObjectError, HandleError, LifecycleEvent, NoDecisionHook, ParticleFlags, ParticleGroupId,
    ParticleGroupMutationError, ParticleSystemId, StepConfiguration, StepLimits, World,
};

use super::snapshot::{
    LifecycleKind, assert_invariants, lifecycle_kind, rollback_snapshot, semantic_snapshot,
};
use super::{MAX_GROUPS, MAX_PARTICLES, Operation, OperationKind, Outcome, Rejection, TraceEntry};

pub(super) struct Model {
    pub(super) world: World,
    pub(super) system: ParticleSystemId,
    foreign_group: ParticleGroupId,
    operation_index: usize,
    maybe_current_operation: Option<Operation>,
    pub(super) lifecycle_records: Vec<LifecycleEvent>,
    pub(super) known_groups: Vec<ParticleGroupId>,
    pub(super) lifecycle: Vec<LifecycleKind>,
}

impl Model {
    pub(super) fn new() -> Self {
        let mut world = World::new().expect("world key remains available");
        world
            .set_gravity(Vec2::ZERO)
            .expect("zero gravity is valid");
        let system = world.create_particle_system().expect("system should fit");
        let foreign_system = world
            .create_particle_system()
            .expect("foreign system should fit");
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
        Self {
            world,
            system,
            foreign_group,
            operation_index: 0,
            maybe_current_operation: None,
            lifecycle_records: Vec::new(),
            known_groups: Vec::new(),
            lifecycle: Vec::new(),
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

    pub(super) fn apply(&mut self, operation: Operation) -> TraceEntry {
        self.operation_index += 1;
        self.maybe_current_operation = Some(operation);
        let before = rollback_snapshot(self);
        let outcome = self.apply_inner(operation);
        let snapshot = semantic_snapshot(self);
        if matches!(outcome, Outcome::Rejected(_)) {
            assert_eq!(
                rollback_snapshot(self),
                before,
                "operation {} {operation:?}: typed rejection must be effect-free",
                self.operation_index
            );
        }
        assert_invariants(self);
        TraceEntry {
            operation,
            outcome,
            snapshot,
        }
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
            Err(CreateObjectError::InvalidHandle(HandleError::PendingDelete)) => {
                assert!(
                    self.world
                        .particle_system_statistics(self.system)
                        .expect("live system")
                        .pending_particle_count()
                        > 0
                );
                Outcome::Rejected(Rejection::PendingDelete)
            }
            Err(CreateObjectError::InvalidParticleGroupTopology) => {
                Outcome::Rejected(Rejection::CreationTopology)
            }
            Err(error) => panic!(
                "operation {} create failed unexpectedly: {error:?}; input {:?}",
                self.operation_index, self.maybe_current_operation
            ),
        }
    }

    fn record_lifecycle(&mut self, events: &[LifecycleEvent]) {
        self.lifecycle.extend(events.iter().map(lifecycle_kind));
        self.lifecycle_records.extend_from_slice(events);
    }

    fn append(&mut self, selector: usize) -> Outcome {
        if self.at_bound(1) {
            return Outcome::SkippedAtBound;
        }
        let live = self.live_groups();
        let Some(target) = select(&live, selector) else {
            return Outcome::SkippedAtBound;
        };
        let before_count = self.particle_count();
        let before_members = self
            .world
            .particle_group_view(target)
            .expect("live target")
            .member_ids()
            .to_vec();
        let position = self
            .world
            .particle_group_view(target)
            .expect("selected group remains live")
            .member_ids()
            .last()
            .and_then(|particle| match self.world.particle_snapshot(*particle) {
                Ok(snapshot) => Some(snapshot),
                Err(HandleError::PendingDelete) => None,
                Err(error) => panic!("append source identity returned unexpected {error:?}"),
            })
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
                assert_eq!(self.particle_count(), before_count + 1);
                let view = self
                    .world
                    .particle_group_view(returned)
                    .expect("append retains target");
                assert_eq!(view.member_count(), before_members.len() + 1);
                assert_eq!(&view.member_ids()[..before_members.len()], before_members);
                Outcome::Applied {
                    created: self.remember([returned]),
                    lifecycle: 0,
                }
            }
            Err(CreateObjectError::InvalidHandle(HandleError::PendingDelete)) => {
                assert!(
                    self.world
                        .particle_system_statistics(self.system)
                        .expect("live system")
                        .pending_particle_count()
                        > 0
                );
                Outcome::Rejected(Rejection::PendingDelete)
            }
            Err(CreateObjectError::InvalidParticleGroupTopology) => {
                Outcome::Rejected(Rejection::CreationTopology)
            }
            Err(error) => panic!(
                "operation {} append failed unexpectedly: {error:?}; input {:?}",
                self.operation_index, self.maybe_current_operation
            ),
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
            Err(ParticleGroupMutationError::InvalidTopology) => {
                Outcome::Rejected(Rejection::MutationTopology)
            }
            Err(error) => panic!(
                "operation {} join failed unexpectedly: {error:?}; input {:?}",
                self.operation_index, self.maybe_current_operation
            ),
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
            Err(ParticleGroupMutationError::InvalidTopology) => {
                Outcome::Rejected(Rejection::MutationTopology)
            }
            Err(error) => panic!(
                "operation {} split failed unexpectedly: {error:?}; input {:?}",
                self.operation_index, self.maybe_current_operation
            ),
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
            Err(ParticleGroupMutationError::InvalidTopology) => {
                Outcome::Rejected(Rejection::MutationTopology)
            }
            Err(error) => panic!(
                "operation {} set_flags failed unexpectedly: {error:?}; input {:?}",
                self.operation_index, self.maybe_current_operation
            ),
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
            Err(error) => panic!(
                "operation {} destroy_members failed unexpectedly: {error:?}; input {:?}",
                self.operation_index, self.maybe_current_operation
            ),
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
            Err(error) => panic!(
                "operation {} compact failed unexpectedly: {error:?}; input {:?}",
                self.operation_index, self.maybe_current_operation
            ),
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
            Err(liquidfun::StepError::InvalidParticleGroupTopology) => {
                Outcome::Rejected(Rejection::StepTopology)
            }
            Err(error) => panic!(
                "operation {} step failed unexpectedly: {error:?}; input {:?}",
                self.operation_index, self.maybe_current_operation
            ),
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
        Outcome::Rejected(Rejection::WrongParticleSystem)
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
