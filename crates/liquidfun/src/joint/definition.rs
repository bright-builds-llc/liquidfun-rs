//! Checked, owned joint definitions.

use std::error::Error;
use std::fmt;

use crate::BodyId;

/// A failure while constructing a reusable joint definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JointDefError {
    /// A joint cannot connect a body to itself.
    SameBody,
}

impl fmt::Display for JointDefError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SameBody => formatter.write_str("a joint must connect two distinct bodies"),
        }
    }
}

impl Error for JointDefError {}

macro_rules! basic_joint_definition {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            body_a: BodyId,
            body_b: BodyId,
            collide_connected: bool,
        }

        impl $name {
            /// Creates a checked definition with collision disabled between its bodies.
            ///
            /// # Errors
            ///
            /// Returns [`JointDefError::SameBody`] when both endpoints are identical.
            pub fn new(body_a: BodyId, body_b: BodyId) -> Result<Self, JointDefError> {
                if body_a == body_b {
                    return Err(JointDefError::SameBody);
                }
                Ok(Self {
                    body_a,
                    body_b,
                    collide_connected: false,
                })
            }

            /// Chooses whether the connected bodies may collide.
            #[must_use]
            pub const fn with_collide_connected(mut self, collide_connected: bool) -> Self {
                self.collide_connected = collide_connected;
                self
            }

            pub(crate) const fn bodies(self) -> [BodyId; 2] {
                [self.body_a, self.body_b]
            }

            pub(crate) const fn collide_connected(self) -> bool {
                self.collide_connected
            }
        }
    };
}

basic_joint_definition!(RevoluteJointDef, "Definition of a revolute joint.");
basic_joint_definition!(PrismaticJointDef, "Definition of a prismatic joint.");
basic_joint_definition!(DistanceJointDef, "Definition of a distance joint.");
basic_joint_definition!(PulleyJointDef, "Definition of a pulley joint.");
basic_joint_definition!(MouseJointDef, "Definition of a mouse joint.");
basic_joint_definition!(GearJointDef, "Definition of a gear joint.");
basic_joint_definition!(WheelJointDef, "Definition of a wheel joint.");
basic_joint_definition!(WeldJointDef, "Definition of a weld joint.");
basic_joint_definition!(FrictionJointDef, "Definition of a friction joint.");
basic_joint_definition!(RopeJointDef, "Definition of a unilateral rope joint.");
basic_joint_definition!(MotorJointDef, "Definition of a motor joint.");

/// The closed set of checked joint definitions accepted by [`crate::World`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JointDef {
    /// Revolute joint definition.
    Revolute(RevoluteJointDef),
    /// Prismatic joint definition.
    Prismatic(PrismaticJointDef),
    /// Distance joint definition.
    Distance(DistanceJointDef),
    /// Pulley joint definition.
    Pulley(PulleyJointDef),
    /// Mouse joint definition.
    Mouse(MouseJointDef),
    /// Gear joint definition.
    Gear(GearJointDef),
    /// Wheel joint definition.
    Wheel(WheelJointDef),
    /// Weld joint definition.
    Weld(WeldJointDef),
    /// Friction joint definition.
    Friction(FrictionJointDef),
    /// Rope joint definition.
    Rope(RopeJointDef),
    /// Motor joint definition.
    Motor(MotorJointDef),
}

impl JointDef {
    /// Returns the connected body identities.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        match self {
            Self::Revolute(definition) => definition.bodies(),
            Self::Prismatic(definition) => definition.bodies(),
            Self::Distance(definition) => definition.bodies(),
            Self::Pulley(definition) => definition.bodies(),
            Self::Mouse(definition) => definition.bodies(),
            Self::Gear(definition) => definition.bodies(),
            Self::Wheel(definition) => definition.bodies(),
            Self::Weld(definition) => definition.bodies(),
            Self::Friction(definition) => definition.bodies(),
            Self::Rope(definition) => definition.bodies(),
            Self::Motor(definition) => definition.bodies(),
        }
    }

    /// Returns whether the connected bodies may collide.
    #[must_use]
    pub const fn collide_connected(self) -> bool {
        match self {
            Self::Revolute(definition) => definition.collide_connected(),
            Self::Prismatic(definition) => definition.collide_connected(),
            Self::Distance(definition) => definition.collide_connected(),
            Self::Pulley(definition) => definition.collide_connected(),
            Self::Mouse(definition) => definition.collide_connected(),
            Self::Gear(definition) => definition.collide_connected(),
            Self::Wheel(definition) => definition.collide_connected(),
            Self::Weld(definition) => definition.collide_connected(),
            Self::Friction(definition) => definition.collide_connected(),
            Self::Rope(definition) => definition.collide_connected(),
            Self::Motor(definition) => definition.collide_connected(),
        }
    }
}

macro_rules! impl_joint_def_from {
    ($definition:ident, $variant:ident) => {
        impl From<$definition> for JointDef {
            fn from(definition: $definition) -> Self {
                Self::$variant(definition)
            }
        }
    };
}

impl_joint_def_from!(RevoluteJointDef, Revolute);
impl_joint_def_from!(PrismaticJointDef, Prismatic);
impl_joint_def_from!(DistanceJointDef, Distance);
impl_joint_def_from!(PulleyJointDef, Pulley);
impl_joint_def_from!(MouseJointDef, Mouse);
impl_joint_def_from!(GearJointDef, Gear);
impl_joint_def_from!(WheelJointDef, Wheel);
impl_joint_def_from!(WeldJointDef, Weld);
impl_joint_def_from!(FrictionJointDef, Friction);
impl_joint_def_from!(RopeJointDef, Rope);
impl_joint_def_from!(MotorJointDef, Motor);
