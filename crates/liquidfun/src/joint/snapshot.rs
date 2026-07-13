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
        }
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
}
