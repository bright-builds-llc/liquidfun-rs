//! Checked joint definitions, identity-neutral snapshots, and common vocabulary.

mod definition;
mod snapshot;

pub use definition::{
    DistanceJointDef, FrictionJointDef, GearJointDef, JointDef, JointDefError, MotorJointDef,
    MouseJointDef, PrismaticJointDef, PulleyJointDef, RevoluteJointDef, RopeJointDef, WeldJointDef,
    WheelJointDef,
};
pub use snapshot::{
    DistanceJointSnapshot, JointLimitState, JointSnapshot, JointSpecificSnapshot,
    MouseJointSnapshot, PrismaticJointSnapshot, PulleyJointSnapshot, RevoluteJointSnapshot,
};

/// The closed set of joint kinds in the pinned `LiquidFun` revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JointKind {
    /// Revolute joint.
    Revolute,
    /// Prismatic joint.
    Prismatic,
    /// Distance joint.
    Distance,
    /// Pulley joint.
    Pulley,
    /// Mouse joint.
    Mouse,
    /// Gear joint.
    Gear,
    /// Wheel joint.
    Wheel,
    /// Weld joint.
    Weld,
    /// Friction joint.
    Friction,
    /// Rope joint.
    Rope,
    /// Motor joint.
    Motor,
}

impl JointKind {
    pub(crate) const fn from_definition(definition: JointDef) -> Self {
        match definition {
            JointDef::Revolute(_) => Self::Revolute,
            JointDef::Prismatic(_) => Self::Prismatic,
            JointDef::Distance(_) => Self::Distance,
            JointDef::Pulley(_) => Self::Pulley,
            JointDef::Mouse(_) => Self::Mouse,
            JointDef::Gear(_) => Self::Gear,
            JointDef::Wheel(_) => Self::Wheel,
            JointDef::Weld(_) => Self::Weld,
            JointDef::Friction(_) => Self::Friction,
            JointDef::Rope(_) => Self::Rope,
            JointDef::Motor(_) => Self::Motor,
        }
    }
}
