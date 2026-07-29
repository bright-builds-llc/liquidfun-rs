//! Checked, owned joint definitions.

use std::error::Error;
use std::fmt;

use crate::{BodyId, JointId};

/// A failure while constructing a reusable joint definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JointDefError {
    /// A joint cannot connect a body to itself.
    SameBody,
    /// A gear joint requires two distinct source joints.
    SameJoint,
    /// A scalar or vector component is not finite.
    NonFiniteValue,
    /// A force or torque cap is negative.
    NegativeValue,
    /// A length or ratio is not strictly positive.
    NonPositiveValue,
    /// A lower limit is greater than its upper limit.
    InvalidRange,
    /// A prismatic axis has no representable direction.
    InvalidAxis,
}

impl fmt::Display for JointDefError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SameBody => formatter.write_str("a joint must connect two distinct bodies"),
            Self::SameJoint => formatter.write_str("a gear joint requires two distinct sources"),
            Self::NonFiniteValue => formatter.write_str("joint values must be finite"),
            Self::NegativeValue => {
                formatter.write_str("joint force and torque caps must be non-negative")
            }
            Self::NonPositiveValue => {
                formatter.write_str("joint lengths and ratios must be finite and positive")
            }
            Self::InvalidRange => {
                formatter.write_str("joint lower limit must not exceed its upper limit")
            }
            Self::InvalidAxis => {
                formatter.write_str("prismatic axis must have a finite non-zero direction")
            }
        }
    }
}

impl Error for JointDefError {}

mod distance_pulley_mouse;
mod friction_rope_motor;
mod gear_wheel_weld;
mod revolute_prismatic;

pub use distance_pulley_mouse::*;
pub use friction_rope_motor::*;
pub use gear_wheel_weld::*;
pub use revolute_prismatic::*;

/// The closed set of checked joint definitions accepted by [`crate::World`].
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Returns the connected body identities when they are definition-owned.
    ///
    /// Gear endpoints are derived from their two live source joints during
    /// world creation, so gear definitions return `None` here.
    #[must_use]
    pub const fn bodies(self) -> Option<[BodyId; 2]> {
        match self {
            Self::Revolute(definition) => Some(definition.bodies()),
            Self::Prismatic(definition) => Some(definition.bodies()),
            Self::Distance(definition) => Some(definition.bodies()),
            Self::Pulley(definition) => Some(definition.bodies()),
            Self::Mouse(definition) => Some(definition.bodies()),
            Self::Gear(_) => None,
            Self::Wheel(definition) => Some(definition.bodies()),
            Self::Weld(definition) => Some(definition.bodies()),
            Self::Friction(definition) => Some(definition.bodies()),
            Self::Rope(definition) => Some(definition.bodies()),
            Self::Motor(definition) => Some(definition.bodies()),
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

fn validate_bodies(body_a: BodyId, body_b: BodyId) -> Result<(), JointDefError> {
    if body_a == body_b {
        Err(JointDefError::SameBody)
    } else {
        Ok(())
    }
}

fn validate_vec(value: crate::math::Vec2) -> Result<(), JointDefError> {
    if value.is_valid() {
        Ok(())
    } else {
        Err(JointDefError::NonFiniteValue)
    }
}

fn validate_scalar(value: f32) -> Result<(), JointDefError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(JointDefError::NonFiniteValue)
    }
}

fn validate_non_negative(value: f32) -> Result<(), JointDefError> {
    validate_scalar(value)?;
    if value < 0.0 {
        Err(JointDefError::NegativeValue)
    } else {
        Ok(())
    }
}

fn validate_positive(value: f32) -> Result<(), JointDefError> {
    validate_scalar(value)?;
    if value <= 0.0 {
        Err(JointDefError::NonPositiveValue)
    } else {
        Ok(())
    }
}

fn validate_range(lower: f32, upper: f32) -> Result<(), JointDefError> {
    validate_scalar(lower)?;
    validate_scalar(upper)?;
    if lower > upper {
        Err(JointDefError::InvalidRange)
    } else {
        Ok(())
    }
}
