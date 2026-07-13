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
    /// Distance runtime state.
    Distance(DistanceJointSnapshot),
    /// Pulley runtime state.
    Pulley(PulleyJointSnapshot),
    /// Mouse runtime state.
    Mouse(MouseJointSnapshot),
    /// Wheel runtime state.
    Wheel(WheelJointSnapshot),
    /// Weld runtime state.
    Weld(WeldJointSnapshot),
    /// A later Phase 8 family has not populated runtime state yet.
    Pending,
}

/// Owned semantic wheel-joint state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelJointSnapshot {
    translation: f32,
    speed: f32,
    motor_enabled: bool,
    motor_speed: f32,
    max_motor_torque: f32,
    frequency: f32,
    damping_ratio: f32,
    gamma: f32,
    bias: f32,
}

impl WheelJointSnapshot {
    #[allow(
        clippy::too_many_arguments,
        reason = "the snapshot mirrors one closed wheel state"
    )]
    pub(crate) const fn new(
        translation: f32,
        speed: f32,
        motor_enabled: bool,
        motor_speed: f32,
        max_motor_torque: f32,
        frequency: f32,
        damping_ratio: f32,
        gamma: f32,
        bias: f32,
    ) -> Self {
        Self {
            translation,
            speed,
            motor_enabled,
            motor_speed,
            max_motor_torque,
            frequency,
            damping_ratio,
            gamma,
            bias,
        }
    }
    /// Returns translation along the body-A suspension axis.
    #[must_use]
    pub const fn translation(self) -> f32 {
        self.translation
    }
    /// Returns relative angular speed.
    #[must_use]
    pub const fn speed(self) -> f32 {
        self.speed
    }
    /// Returns whether the rotational motor is enabled.
    #[must_use]
    pub const fn is_motor_enabled(self) -> bool {
        self.motor_enabled
    }
    /// Returns target motor speed.
    #[must_use]
    pub const fn motor_speed(self) -> f32 {
        self.motor_speed
    }
    /// Returns maximum motor torque.
    #[must_use]
    pub const fn max_motor_torque(self) -> f32 {
        self.max_motor_torque
    }
    /// Returns spring frequency in hertz.
    #[must_use]
    pub const fn frequency(self) -> f32 {
        self.frequency
    }
    /// Returns spring damping ratio.
    #[must_use]
    pub const fn damping_ratio(self) -> f32 {
        self.damping_ratio
    }
    /// Returns the last initialized softness gamma.
    #[must_use]
    pub const fn gamma(self) -> f32 {
        self.gamma
    }
    /// Returns the last initialized spring bias.
    #[must_use]
    pub const fn bias(self) -> f32 {
        self.bias
    }
}

/// Owned semantic weld-joint state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeldJointSnapshot {
    reference_angle: f32,
    frequency: f32,
    damping_ratio: f32,
    gamma: f32,
    bias: f32,
}

impl WeldJointSnapshot {
    pub(crate) const fn new(
        reference_angle: f32,
        frequency: f32,
        damping_ratio: f32,
        gamma: f32,
        bias: f32,
    ) -> Self {
        Self {
            reference_angle,
            frequency,
            damping_ratio,
            gamma,
            bias,
        }
    }
    /// Returns the configured reference angle.
    #[must_use]
    pub const fn reference_angle(self) -> f32 {
        self.reference_angle
    }
    /// Returns rotational softness frequency in hertz.
    #[must_use]
    pub const fn frequency(self) -> f32 {
        self.frequency
    }
    /// Returns the rotational damping ratio.
    #[must_use]
    pub const fn damping_ratio(self) -> f32 {
        self.damping_ratio
    }
    /// Returns the last initialized angular gamma.
    #[must_use]
    pub const fn gamma(self) -> f32 {
        self.gamma
    }
    /// Returns the last initialized angular bias.
    #[must_use]
    pub const fn bias(self) -> f32 {
        self.bias
    }
}

/// Owned semantic distance-joint state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistanceJointSnapshot {
    length: f32,
    current_length: f32,
    frequency: f32,
    damping_ratio: f32,
    gamma: f32,
    bias: f32,
}

impl DistanceJointSnapshot {
    pub(crate) const fn new(
        length: f32,
        current_length: f32,
        frequency: f32,
        damping_ratio: f32,
        gamma: f32,
        bias: f32,
    ) -> Self {
        Self {
            length,
            current_length,
            frequency,
            damping_ratio,
            gamma,
            bias,
        }
    }
    /// Returns the configured natural length.
    #[must_use]
    pub const fn length(self) -> f32 {
        self.length
    }
    /// Returns the current distance between world anchors.
    #[must_use]
    pub const fn current_length(self) -> f32 {
        self.current_length
    }
    /// Returns the softness frequency in hertz.
    #[must_use]
    pub const fn frequency(self) -> f32 {
        self.frequency
    }
    /// Returns the damping ratio.
    #[must_use]
    pub const fn damping_ratio(self) -> f32 {
        self.damping_ratio
    }
    /// Returns the last initialized softness gamma.
    #[must_use]
    pub const fn gamma(self) -> f32 {
        self.gamma
    }
    /// Returns the last initialized softness bias.
    #[must_use]
    pub const fn bias(self) -> f32 {
        self.bias
    }
}

/// Owned semantic pulley-joint state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PulleyJointSnapshot {
    ground_anchor_a: Vec2,
    ground_anchor_b: Vec2,
    length_a: f32,
    length_b: f32,
    current_length_a: f32,
    current_length_b: f32,
    ratio: f32,
    constant: f32,
}

impl PulleyJointSnapshot {
    #[allow(
        clippy::too_many_arguments,
        reason = "the snapshot mirrors one closed pulley state"
    )]
    pub(crate) const fn new(
        ground_anchor_a: Vec2,
        ground_anchor_b: Vec2,
        length_a: f32,
        length_b: f32,
        current_length_a: f32,
        current_length_b: f32,
        ratio: f32,
        constant: f32,
    ) -> Self {
        Self {
            ground_anchor_a,
            ground_anchor_b,
            length_a,
            length_b,
            current_length_a,
            current_length_b,
            ratio,
            constant,
        }
    }
    /// Returns the fixed world-space ground anchor on side A.
    #[must_use]
    pub const fn ground_anchor_a(self) -> Vec2 {
        self.ground_anchor_a
    }
    /// Returns the fixed world-space ground anchor on side B.
    #[must_use]
    pub const fn ground_anchor_b(self) -> Vec2 {
        self.ground_anchor_b
    }
    /// Returns the reference segment length on side A.
    #[must_use]
    pub const fn length_a(self) -> f32 {
        self.length_a
    }
    /// Returns the reference segment length on side B.
    #[must_use]
    pub const fn length_b(self) -> f32 {
        self.length_b
    }
    /// Returns the current segment length on side A.
    #[must_use]
    pub const fn current_length_a(self) -> f32 {
        self.current_length_a
    }
    /// Returns the current segment length on side B.
    #[must_use]
    pub const fn current_length_b(self) -> f32 {
        self.current_length_b
    }
    /// Returns the pulley ratio.
    #[must_use]
    pub const fn ratio(self) -> f32 {
        self.ratio
    }
    /// Returns the source constant.
    #[must_use]
    pub const fn constant(self) -> f32 {
        self.constant
    }
}

/// Owned semantic mouse-joint state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseJointSnapshot {
    target: Vec2,
    max_force: f32,
    frequency: f32,
    damping_ratio: f32,
    gamma: f32,
    beta: f32,
}

impl MouseJointSnapshot {
    pub(crate) const fn new(
        target: Vec2,
        max_force: f32,
        frequency: f32,
        damping_ratio: f32,
        gamma: f32,
        beta: f32,
    ) -> Self {
        Self {
            target,
            max_force,
            frequency,
            damping_ratio,
            gamma,
            beta,
        }
    }
    /// Returns the world-space target.
    #[must_use]
    pub const fn target(self) -> Vec2 {
        self.target
    }
    /// Returns the maximum force.
    #[must_use]
    pub const fn max_force(self) -> f32 {
        self.max_force
    }
    /// Returns the response frequency in hertz.
    #[must_use]
    pub const fn frequency(self) -> f32 {
        self.frequency
    }
    /// Returns the damping ratio.
    #[must_use]
    pub const fn damping_ratio(self) -> f32 {
        self.damping_ratio
    }
    /// Returns the last initialized softness gamma.
    #[must_use]
    pub const fn gamma(self) -> f32 {
        self.gamma
    }
    /// Returns the last initialized softness beta.
    #[must_use]
    pub const fn beta(self) -> f32 {
        self.beta
    }
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
