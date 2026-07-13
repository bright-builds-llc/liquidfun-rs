//! Owned semantic joint snapshots.

use crate::math::Vec2;
use crate::{BodyId, JointDef, JointKind};

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
pub enum JointSpecificSnapshot {
    /// Revolute runtime state.
    Revolute(RevoluteJointSnapshot),
    /// Prismatic runtime state.
    Prismatic(PrismaticJointSnapshot),
    /// A later Phase 8 family has not populated runtime state yet.
    Pending,
}

/// Owned semantic revolute-joint state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RevoluteJointSnapshot {
    angle: f32,
    speed: f32,
    limit_state: JointLimitState,
    motor_impulse: f32,
}

impl RevoluteJointSnapshot {
    pub(crate) const fn new(
        angle: f32,
        speed: f32,
        limit_state: JointLimitState,
        motor_impulse: f32,
    ) -> Self {
        Self {
            angle,
            speed,
            limit_state,
            motor_impulse,
        }
    }
    /// Returns the relative angle after subtracting the reference angle.
    #[must_use]
    pub const fn angle(self) -> f32 {
        self.angle
    }
    /// Returns the relative angular speed.
    #[must_use]
    pub const fn speed(self) -> f32 {
        self.speed
    }
    /// Returns the current source-classified limit state.
    #[must_use]
    pub const fn limit_state(self) -> JointLimitState {
        self.limit_state
    }
    /// Returns the cached motor impulse.
    #[must_use]
    pub const fn motor_impulse(self) -> f32 {
        self.motor_impulse
    }
}

/// Owned semantic prismatic-joint state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrismaticJointSnapshot {
    translation: f32,
    speed: f32,
    limit_state: JointLimitState,
    motor_impulse: f32,
}

impl PrismaticJointSnapshot {
    pub(crate) const fn new(
        translation: f32,
        speed: f32,
        limit_state: JointLimitState,
        motor_impulse: f32,
    ) -> Self {
        Self {
            translation,
            speed,
            limit_state,
            motor_impulse,
        }
    }
    /// Returns translation along the body-A axis.
    #[must_use]
    pub const fn translation(self) -> f32 {
        self.translation
    }
    /// Returns translation speed along the body-A axis.
    #[must_use]
    pub const fn speed(self) -> f32 {
        self.speed
    }
    /// Returns the current source-classified limit state.
    #[must_use]
    pub const fn limit_state(self) -> JointLimitState {
        self.limit_state
    }
    /// Returns the cached motor impulse.
    #[must_use]
    pub const fn motor_impulse(self) -> f32 {
        self.motor_impulse
    }
}

impl JointSnapshot {
    pub(crate) const fn from_definition(definition: JointDef) -> Self {
        Self {
            kind: JointKind::from_definition(definition),
            bodies: definition.bodies(),
            collide_connected: definition.collide_connected(),
            anchor_a: Vec2::ZERO,
            anchor_b: Vec2::ZERO,
            definition,
            specific: JointSpecificSnapshot::Pending,
        }
    }

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
