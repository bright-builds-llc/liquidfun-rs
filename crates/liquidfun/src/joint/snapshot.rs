//! Owned semantic joint snapshots.

use crate::math::Vec2;
use crate::{BodyId, JointDef, JointId, JointKind};

/// Runtime state of a joint limit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum JointLimitState {
    /// No limit is active.
    #[default]
    Inactive,
    /// The lower limit is active.
    AtLower,
    /// The upper limit is active.
    AtUpper,
    /// Equal lower and upper limits are active.
    Equal,
}

/// Owned semantic state of one live joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JointSnapshot {
    kind: JointKind,
    bodies: [BodyId; 2],
    collide_connected: bool,
    anchor_a: Vec2,
    anchor_b: Vec2,
    definition: JointDef,
    specific: JointSpecificSnapshot,
}

/// Owned runtime details for a supported concrete joint family.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "owned gear evidence includes six opaque world-scoped identities"
)]
pub enum JointSpecificSnapshot {
    /// Revolute runtime state.
    Revolute(RevoluteJointSnapshot),
    /// Prismatic runtime state.
    Prismatic(PrismaticJointSnapshot),
    /// Distance runtime state.
    Distance(DistanceJointSnapshot),
    /// Pulley runtime state.
    Pulley(PulleyJointSnapshot),
    /// Mouse runtime state.
    Mouse(MouseJointSnapshot),
    /// Gear runtime state.
    Gear(GearJointSnapshot),
    /// Wheel runtime state.
    Wheel(WheelJointSnapshot),
    /// Weld runtime state.
    Weld(WeldJointSnapshot),
    /// Friction runtime state.
    Friction(FrictionJointSnapshot),
    /// Rope-joint runtime state.
    Rope(RopeJointSnapshot),
    /// Motor-joint runtime state.
    Motor(MotorJointSnapshot),
    /// A later Phase 8 family has not populated runtime state yet.
    Pending,
}

mod distance_pulley;
mod gear_friction_rope_motor;
mod mouse_revolute_prismatic;
mod wheel_weld;

pub use distance_pulley::*;
pub use gear_friction_rope_motor::*;
pub use mouse_revolute_prismatic::*;
pub use wheel_weld::*;

impl JointSnapshot {
    pub(crate) const fn from_definition(definition: JointDef, bodies: [BodyId; 2]) -> Self {
        Self {
            kind: JointKind::from_definition(definition),
            bodies,
            collide_connected: definition.collide_connected(),
            anchor_a: Vec2::ZERO,
            anchor_b: Vec2::ZERO,
            definition,
            specific: JointSpecificSnapshot::Pending,
        }
    }

    #[allow(
        clippy::large_types_passed_by_value,
        reason = "the closed owned snapshot remains Copy by design"
    )]
    pub(crate) const fn with_runtime(
        mut self,
        anchor_a: Vec2,
        anchor_b: Vec2,
        specific: JointSpecificSnapshot,
    ) -> Self {
        self.anchor_a = anchor_a;
        self.anchor_b = anchor_b;
        self.specific = specific;
        self
    }

    /// Returns the concrete joint kind.
    #[must_use]
    pub const fn kind(self) -> JointKind {
        self.kind
    }

    /// Returns the two connected bodies.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        self.bodies
    }

    /// Returns whether the connected bodies may collide.
    #[must_use]
    pub const fn collide_connected(self) -> bool {
        self.collide_connected
    }

    /// Returns the current world-space anchor on body A.
    #[must_use]
    pub const fn anchor_a(self) -> Vec2 {
        self.anchor_a
    }

    /// Returns the current world-space anchor on body B.
    #[must_use]
    pub const fn anchor_b(self) -> Vec2 {
        self.anchor_b
    }

    /// Returns the complete checked definition that created the joint.
    #[must_use]
    pub const fn definition(self) -> JointDef {
        self.definition
    }

    /// Returns the owned concrete runtime state.
    #[must_use]
    pub const fn specific(self) -> JointSpecificSnapshot {
        self.specific
    }
}
