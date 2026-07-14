//! World-owned joint lifecycle and checked common queries.

use std::error::Error;
use std::fmt;

use crate::math::Vec2;
use crate::{
    ArenaInsertError, BodyId, DestructionRecord, HandleError, JointDef, JointDefError, JointId,
    JointKind, JointSnapshot,
};

use super::object::{DestructionCause, DestructionReport, MutationReport, World};
use super::step::LifecycleEvent;

type GearCreation = ([BodyId; 2], [JointDef; 2], [gear::GearBodyGeometry; 4]);

mod distance;
mod friction;
mod gear;
mod motor;
mod mouse;
mod prismatic;
mod pulley;
mod revolute;
mod rope;
pub(super) mod solver;
mod weld;
mod wheel;

#[derive(Debug, Clone, Copy)]
pub(super) enum JointRuntime {
    Revolute(revolute::RevoluteRuntime),
    Prismatic(prismatic::PrismaticRuntime),
    Distance(distance::DistanceRuntime),
    Pulley(pulley::PulleyRuntime),
    Mouse(mouse::MouseRuntime),
    Wheel(wheel::WheelRuntime),
    Weld(weld::WeldRuntime),
    Friction(friction::FrictionRuntime),
    Gear(gear::GearRuntime),
    Rope(rope::RopeJointRuntime),
    Motor(motor::MotorRuntime),
}

impl JointRuntime {
    fn maybe_from_definition(
        definition: JointDef,
        body_b_transform: crate::math::Transform,
    ) -> Option<Self> {
        Some(match definition {
            JointDef::Revolute(definition) => {
                Self::Revolute(revolute::RevoluteRuntime::new(definition))
            }
            JointDef::Prismatic(definition) => {
                Self::Prismatic(prismatic::PrismaticRuntime::new(definition))
            }
            JointDef::Distance(definition) => {
                Self::Distance(distance::DistanceRuntime::new(definition))
            }
            JointDef::Pulley(definition) => Self::Pulley(pulley::PulleyRuntime::new(definition)),
            JointDef::Mouse(definition) => {
                Self::Mouse(mouse::MouseRuntime::new(definition, body_b_transform))
            }
            JointDef::Wheel(definition) => Self::Wheel(wheel::WheelRuntime::new(definition)),
            JointDef::Weld(definition) => Self::Weld(weld::WeldRuntime::new(definition)),
            JointDef::Friction(definition) => {
                Self::Friction(friction::FrictionRuntime::new(definition))
            }
            JointDef::Rope(definition) => Self::Rope(rope::RopeJointRuntime::new(definition)),
            JointDef::Motor(definition) => Self::Motor(motor::MotorRuntime::new(definition)),
            JointDef::Gear(_) => return None,
        })
    }
}

#[derive(Debug)]
pub(super) struct JointRecord {
    pub(super) diagnostic_id: u64,
    pub(super) bodies: [BodyId; 2],
    pub(super) definition: JointDef,
    pub(super) collide_connected: bool,
    pub(super) runtime: JointRuntime,
    #[allow(dead_code, reason = "consumed by the Phase 8 gear lifecycle plan")]
    pub(super) reverse_gear_dependents: Vec<JointId>,
}

impl JointRecord {
    pub(super) fn shifted_definition(&self, shift: Vec2) -> Result<JointDef, JointMutationError> {
        match self.definition {
            JointDef::Pulley(definition) => {
                let anchors = pulley::PulleyRuntime::shifted_ground_anchors(definition, shift)?;
                definition
                    .with_geometry(
                        anchors[0],
                        anchors[1],
                        definition.local_anchor_a(),
                        definition.local_anchor_b(),
                        definition.length_a(),
                        definition.length_b(),
                        definition.ratio(),
                    )
                    .map(JointDef::from)
                    .map_err(|_error| JointMutationError::NonFiniteDerivedState)
            }
            JointDef::Mouse(definition) => {
                let target = mouse::MouseRuntime::shifted_target(definition, shift)?;
                definition
                    .with_target(target)
                    .map(JointDef::from)
                    .map_err(|_error| JointMutationError::NonFiniteDerivedState)
            }
            definition => Ok(definition),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::fixture::test_fixture_definition;
    use crate::{BodyDef, RevoluteJointDef};

    fn test_world_with_bodies() -> (World, BodyId, BodyId) {
        let mut world = World::new().expect("test world key should remain available");
        let body_a = world
            .create_body(&BodyDef::default())
            .expect("body A should fit");
        let body_b = world
            .create_body(&BodyDef::default())
            .expect("body B should fit");
        (world, body_a, body_b)
    }

    #[test]
    fn locked_creation_is_rejected_without_adjacency_effects() {
        // Arrange
        let (mut world, body_a, body_b) = test_world_with_bodies();
        let definition = RevoluteJointDef::new(body_a, body_b)
            .expect("distinct bodies form a valid joint")
            .into();
        world.step_state.set_locked_for_test(true);

        // Act
        let result = world.create_joint(definition);
        world.step_state.set_locked_for_test(false);

        // Assert
        assert_eq!(result, Err(JointCreationError::Locked));
        assert!(world.body_mut_after_validation(body_a).joints.is_empty());
        assert!(world.body_mut_after_validation(body_b).joints.is_empty());
    }

    #[test]
    fn poisoned_creation_is_rejected_without_adjacency_effects() {
        // Arrange
        let (mut world, body_a, body_b) = test_world_with_bodies();
        let definition = RevoluteJointDef::new(body_a, body_b)
            .expect("distinct bodies form a valid joint")
            .into();
        world.step_state.set_poisoned_for_test(true);

        // Act
        let result = world.create_joint(definition);
        world.step_state.set_poisoned_for_test(false);

        // Assert
        assert_eq!(result, Err(JointCreationError::Poisoned));
        assert!(world.body_mut_after_validation(body_a).joints.is_empty());
        assert!(world.body_mut_after_validation(body_b).joints.is_empty());
    }

    #[test]
    fn collision_suppression_refilters_only_after_last_joint_is_removed() {
        // Arrange
        let (mut world, body_a, body_b) = test_world_with_bodies();
        let fixture = world
            .create_fixture(body_a, &test_fixture_definition())
            .expect("fixture should fit");
        let definition = RevoluteJointDef::new(body_a, body_b)
            .expect("distinct bodies form a valid joint")
            .into();
        let first = world.create_joint(definition).expect("joint should fit");
        let second = world.create_joint(definition).expect("joint should fit");
        world
            .fixtures
            .get_mut(fixture)
            .expect("fixture should remain live")
            .pending_refilter = false;

        // Act
        world.destroy_joint(second).expect("joint should be live");
        let after_first = world
            .fixtures
            .get(fixture)
            .expect("fixture should remain live")
            .pending_refilter;
        world.destroy_joint(first).expect("joint should be live");
        let after_last = world
            .fixtures
            .get(fixture)
            .expect("fixture should remain live")
            .pending_refilter;

        // Assert
        assert!(!after_first);
        assert!(after_last);
    }
}

/// A failure while creating a joint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JointCreationError {
    /// An endpoint does not resolve in this world.
    InvalidHandle(HandleError),
    /// The checked definition is invalid.
    InvalidDefinition(JointDefError),
    /// A gear dependency is live but is not revolute or prismatic.
    WrongDependencyKind {
        /// Source joint that has the unsupported kind.
        dependency: JointId,
        /// Concrete unsupported kind.
        actual: JointKind,
    },
    /// Gear source topology does not produce two distinct derived endpoints.
    InvalidGearTopology,
    /// Checked gear coordinate arithmetic produced a non-finite result.
    NonFiniteDerivedState,
    /// Joint storage cannot accept another entry.
    Arena(ArenaInsertError),
    /// The world is locked by an active step.
    Locked,
    /// A prior hook panic poisoned coherent world operations.
    Poisoned,
}

impl fmt::Display for JointCreationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid joint endpoint: {error}"),
            Self::InvalidDefinition(error) => {
                write!(formatter, "invalid joint definition: {error}")
            }
            Self::WrongDependencyKind { dependency, actual } => {
                write!(
                    formatter,
                    "gear dependency {dependency:?} has unsupported kind {actual:?}"
                )
            }
            Self::InvalidGearTopology => {
                formatter.write_str("gear sources must derive two distinct moving bodies")
            }
            Self::NonFiniteDerivedState => {
                formatter.write_str("gear creation produced non-finite derived state")
            }
            Self::Arena(error) => write!(formatter, "could not store joint: {error}"),
            Self::Locked => formatter.write_str("world is locked by an active step"),
            Self::Poisoned => formatter.write_str("world is poisoned by a prior hook panic"),
        }
    }
}

impl Error for JointCreationError {}

impl From<HandleError> for JointCreationError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<ArenaInsertError> for JointCreationError {
    fn from(error: ArenaInsertError) -> Self {
        Self::Arena(error)
    }
}

/// A failure while querying a live joint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JointQueryError {
    /// The identity does not resolve in this world.
    InvalidHandle(HandleError),
    /// A kind-specific query received another joint kind.
    WrongKind {
        /// Required joint kind.
        expected: JointKind,
        /// Actual joint kind.
        actual: JointKind,
    },
    /// The inverse timestep is negative or non-finite.
    InvalidInverseTimestep,
    /// Checked source arithmetic produced a non-finite semantic result.
    NonFiniteDerivedState,
    /// A prior hook panic poisoned coherent world operations.
    Poisoned,
}

impl fmt::Display for JointQueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid joint handle: {error}"),
            Self::WrongKind { expected, actual } => {
                write!(formatter, "expected {expected:?} joint, found {actual:?}")
            }
            Self::InvalidInverseTimestep => {
                formatter.write_str("inverse timestep must be finite and non-negative")
            }
            Self::NonFiniteDerivedState => {
                formatter.write_str("joint query produced non-finite state")
            }
            Self::Poisoned => formatter.write_str("world is poisoned by a prior hook panic"),
        }
    }
}

impl Error for JointQueryError {}

impl From<HandleError> for JointQueryError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

/// A failure while mutating or destroying a joint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JointMutationError {
    /// The identity does not resolve in this world.
    InvalidHandle(HandleError),
    /// A kind-specific operation received another joint kind.
    WrongKind {
        /// Required joint kind.
        expected: JointKind,
        /// Actual joint kind.
        actual: JointKind,
    },
    /// A requested scalar or vector is outside its checked domain.
    InvalidValue,
    /// The world is locked by an active step.
    Locked,
    /// A prior hook panic poisoned coherent world operations.
    Poisoned,
    /// Candidate arithmetic produced non-finite state.
    NonFiniteDerivedState,
}

impl fmt::Display for JointMutationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid joint handle: {error}"),
            Self::WrongKind { expected, actual } => {
                write!(formatter, "expected {expected:?} joint, found {actual:?}")
            }
            Self::InvalidValue => formatter.write_str("invalid joint value"),
            Self::Locked => formatter.write_str("world is locked by an active step"),
            Self::Poisoned => formatter.write_str("world is poisoned by a prior hook panic"),
            Self::NonFiniteDerivedState => {
                formatter.write_str("joint mutation produced non-finite state")
            }
        }
    }
}

impl Error for JointMutationError {}

impl From<HandleError> for JointMutationError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl World {
    /// Creates a joint from a complete checked definition.
    ///
    /// The definition is validated against this world before storage or body
    /// adjacency changes. New joints are inserted newest-first on both body lanes.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for foreign or stale endpoints, locked or
    /// poisoned worlds, or exhausted storage.
    pub fn create_joint(&mut self, definition: JointDef) -> Result<JointId, JointCreationError> {
        if self.step_state.is_poisoned() {
            return Err(JointCreationError::Poisoned);
        }
        if self.step_state.is_locked() {
            return Err(JointCreationError::Locked);
        }
        let (bodies, runtime, maybe_dependencies) = if let JointDef::Gear(gear_definition) =
            definition
        {
            let (bodies, source_definitions, geometries) =
                self.prepare_gear_creation(gear_definition)?;
            let runtime = gear::GearRuntime::new(gear_definition, &source_definitions, geometries)
                .map_err(|_| JointCreationError::NonFiniteDerivedState)?;
            (
                bodies,
                JointRuntime::Gear(runtime),
                Some(gear_definition.source_joints()),
            )
        } else {
            let Some(bodies) = definition.bodies() else {
                return Err(JointCreationError::InvalidGearTopology);
            };
            self.bodies.get(bodies[0])?;
            let body_b_transform = self.bodies.get(bodies[1])?.state.snapshot().transform();
            let Some(runtime) = JointRuntime::maybe_from_definition(definition, body_b_transform)
            else {
                return Err(JointCreationError::InvalidGearTopology);
            };
            (bodies, runtime, None)
        };
        if bodies[0] == bodies[1] {
            return Err(JointCreationError::InvalidDefinition(
                JointDefError::SameBody,
            ));
        }

        let diagnostic_id = self.allocate_diagnostic_id()?;
        let collide_connected = definition.collide_connected();
        let joint = self.joints.insert(JointRecord {
            diagnostic_id,
            bodies,
            definition,
            collide_connected,
            runtime,
            reverse_gear_dependents: Vec::new(),
        })?;
        self.body_mut_after_validation(bodies[0])
            .joints
            .insert(0, joint);
        self.body_mut_after_validation(bodies[1])
            .joints
            .insert(0, joint);
        if let Some(dependencies) = maybe_dependencies {
            for dependency in dependencies {
                self.joint_mut_after_validation(dependency)
                    .reverse_gear_dependents
                    .insert(0, joint);
            }
        }
        if !collide_connected {
            self.mark_joint_bodies_for_refilter(bodies);
        }
        Ok(joint)
    }

    /// Returns an owned semantic snapshot of a live joint.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the world is poisoned or the identity is foreign or stale.
    pub fn joint_snapshot(&self, joint: JointId) -> Result<JointSnapshot, JointQueryError> {
        self.ensure_joint_queryable()?;
        let record = self.joints.get(joint)?;
        match record.runtime {
            JointRuntime::Revolute(runtime) => revolute::snapshot(self, record, runtime),
            JointRuntime::Prismatic(runtime) => prismatic::snapshot(self, record, runtime),
            JointRuntime::Distance(runtime) => distance::snapshot(self, record, runtime),
            JointRuntime::Pulley(runtime) => pulley::snapshot(self, record, runtime),
            JointRuntime::Mouse(runtime) => mouse::snapshot(self, record, runtime),
            JointRuntime::Wheel(runtime) => wheel::snapshot(self, record, runtime),
            JointRuntime::Weld(runtime) => weld::snapshot(self, record, runtime),
            JointRuntime::Friction(runtime) => friction::snapshot(self, record, runtime),
            JointRuntime::Gear(runtime) => gear::snapshot(self, record, runtime),
            JointRuntime::Rope(runtime) => rope::snapshot(self, record, runtime),
            JointRuntime::Motor(runtime) => motor::snapshot(self, record, runtime),
        }
    }

    /// Returns a snapshot after checking its concrete kind.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid identity, poisoned world, or kind mismatch.
    pub fn joint_snapshot_of_kind(
        &self,
        joint: JointId,
        expected: JointKind,
    ) -> Result<JointSnapshot, JointQueryError> {
        let snapshot = self.joint_snapshot(joint)?;
        let actual = snapshot.kind();
        if actual != expected {
            return Err(JointQueryError::WrongKind { expected, actual });
        }
        Ok(snapshot)
    }

    /// Returns the reaction force on body B for an explicit inverse timestep.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid identity, poisoned world, or invalid timestep.
    pub fn joint_reaction_force(
        &self,
        joint: JointId,
        inverse_timestep: f32,
    ) -> Result<Vec2, JointQueryError> {
        self.validate_reaction_query(joint, inverse_timestep)?;
        let record = self.joints.get(joint)?;
        match record.runtime {
            JointRuntime::Revolute(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
            JointRuntime::Prismatic(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
            JointRuntime::Distance(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
            JointRuntime::Pulley(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
            JointRuntime::Mouse(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
            JointRuntime::Wheel(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
            JointRuntime::Weld(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
            JointRuntime::Friction(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
            JointRuntime::Gear(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
            JointRuntime::Rope(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
            JointRuntime::Motor(runtime) => Ok(runtime.reaction_force(inverse_timestep)),
        }
    }

    /// Returns the reaction torque on body B for an explicit inverse timestep.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid identity, poisoned world, or invalid timestep.
    pub fn joint_reaction_torque(
        &self,
        joint: JointId,
        inverse_timestep: f32,
    ) -> Result<f32, JointQueryError> {
        self.validate_reaction_query(joint, inverse_timestep)?;
        let record = self.joints.get(joint)?;
        match record.runtime {
            JointRuntime::Revolute(runtime) => Ok(runtime.reaction_torque(inverse_timestep)),
            JointRuntime::Prismatic(runtime) => Ok(runtime.reaction_torque(inverse_timestep)),
            JointRuntime::Wheel(runtime) => Ok(runtime.reaction_torque(inverse_timestep)),
            JointRuntime::Weld(runtime) => Ok(runtime.reaction_torque(inverse_timestep)),
            JointRuntime::Friction(runtime) => Ok(runtime.reaction_torque(inverse_timestep)),
            JointRuntime::Gear(runtime) => Ok(runtime.reaction_torque(inverse_timestep)),
            JointRuntime::Motor(runtime) => Ok(runtime.reaction_torque(inverse_timestep)),
            JointRuntime::Distance(_)
            | JointRuntime::Pulley(_)
            | JointRuntime::Mouse(_)
            | JointRuntime::Rope(_) => Ok(0.0),
        }
    }

    /// Destroys one live joint after validating the complete operation.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for an invalid identity or locked or poisoned world.
    pub fn destroy_joint(
        &mut self,
        joint: JointId,
    ) -> Result<DestructionReport, JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = self.joints.get(joint)?;
        let dependents = record.reverse_gear_dependents.clone();
        let mut records = Vec::with_capacity(dependents.len() + 1);
        let mut lifecycle = Vec::with_capacity(dependents.len() + 1);
        for dependent in dependents {
            let record = self.remove_joint_with_refilter(
                dependent,
                DestructionCause::GearDependencyCascade { source: joint },
            );
            lifecycle.push(LifecycleEvent::JointGoodbye(record.clone()));
            records.push(record);
        }
        let root = self.remove_joint_with_refilter(joint, DestructionCause::Explicit);
        lifecycle.push(LifecycleEvent::Destruction(root.clone()));
        records.push(root);
        Ok(MutationReport::new(records, lifecycle))
    }

    fn prepare_gear_creation(
        &self,
        definition: crate::GearJointDef,
    ) -> Result<GearCreation, JointCreationError> {
        let dependencies = definition.source_joints();
        let source1 = self.joints.get(dependencies[0])?;
        let source2 = self.joints.get(dependencies[1])?;
        for (dependency, source) in dependencies.into_iter().zip([source1, source2]) {
            if !matches!(
                source.definition,
                JointDef::Revolute(_) | JointDef::Prismatic(_)
            ) {
                return Err(JointCreationError::WrongDependencyKind {
                    dependency,
                    actual: JointKind::from_definition(source.definition),
                });
            }
        }
        let source_bodies = [
            source1.bodies[1],
            source2.bodies[1],
            source1.bodies[0],
            source2.bodies[0],
        ];
        if source_bodies[0] == source_bodies[1] {
            return Err(JointCreationError::InvalidGearTopology);
        }
        let geometries = [
            gear::body_geometry(self, source_bodies[0])?,
            gear::body_geometry(self, source_bodies[1])?,
            gear::body_geometry(self, source_bodies[2])?,
            gear::body_geometry(self, source_bodies[3])?,
        ];
        Ok((
            [source_bodies[0], source_bodies[1]],
            [source1.definition, source2.definition],
            geometries,
        ))
    }

    fn remove_joint_with_refilter(
        &mut self,
        joint: JointId,
        cause: DestructionCause,
    ) -> DestructionRecord {
        let record = self
            .joints
            .get(joint)
            .expect("collected joint cascade remains live");
        let bodies = record.bodies;
        let was_suppressing = !record.collide_connected;
        let destruction = self.remove_joint(joint, cause);
        if was_suppressing && !self.has_suppressing_joint_between(bodies) {
            self.mark_joint_bodies_for_refilter(bodies);
        }
        destruction
    }

    fn validate_reaction_query(
        &self,
        joint: JointId,
        inverse_timestep: f32,
    ) -> Result<(), JointQueryError> {
        self.ensure_joint_queryable()?;
        if !inverse_timestep.is_finite() || inverse_timestep < 0.0 {
            return Err(JointQueryError::InvalidInverseTimestep);
        }
        self.joints.get(joint)?;
        Ok(())
    }

    fn ensure_joint_queryable(&self) -> Result<(), JointQueryError> {
        if self.step_state.is_poisoned() {
            return Err(JointQueryError::Poisoned);
        }
        Ok(())
    }

    fn ensure_joint_mutable(&self) -> Result<(), JointMutationError> {
        if self.step_state.is_poisoned() {
            return Err(JointMutationError::Poisoned);
        }
        if self.step_state.is_locked() {
            return Err(JointMutationError::Locked);
        }
        Ok(())
    }

    fn wake_joint_bodies(&mut self, bodies: [BodyId; 2]) {
        for body in bodies {
            let record = self.body_mut_after_validation(body);
            record.state = record.state.candidate_set_awake(true);
        }
    }

    fn joint_mut_after_validation(&mut self, joint: JointId) -> &mut JointRecord {
        self.joints
            .get_mut(joint)
            .expect("validated joint remains live during one operation")
    }

    fn has_suppressing_joint_between(&self, bodies: [BodyId; 2]) -> bool {
        self.joints.iter().any(|(_joint, record)| {
            !record.collide_connected
                && (record.bodies == bodies || record.bodies == [bodies[1], bodies[0]])
        })
    }

    fn mark_joint_bodies_for_refilter(&mut self, bodies: [BodyId; 2]) {
        let mut fixtures = self
            .bodies
            .get(bodies[0])
            .expect("validated joint body remains live")
            .fixtures
            .clone();
        fixtures.extend(
            self.bodies
                .get(bodies[1])
                .expect("validated joint body remains live")
                .fixtures
                .iter()
                .copied(),
        );
        for fixture in fixtures {
            let record = self
                .fixtures
                .get_mut(fixture)
                .expect("body fixture adjacency remains live");
            record.pending_refilter = true;
            record
                .proxies
                .touch(&mut self.broad_phase, fixture, record.body);
            self.contact_manager.flag_fixture_for_filtering(fixture);
        }
    }
}
