use super::{BodyId, JointId, JointLimitState, Vec2};

/// Owned semantic state of a gear joint and both source constraints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GearJointSnapshot {
    source_joints: [JointId; 2],
    source_bodies: [BodyId; 4],
    ratio: f32,
    constant: f32,
    coordinate1: f32,
    coordinate2: f32,
}

impl GearJointSnapshot {
    pub(crate) const fn new(
        source_joints: [JointId; 2],
        source_bodies: [BodyId; 4],
        ratio: f32,
        constant: f32,
        coordinate1: f32,
        coordinate2: f32,
    ) -> Self {
        Self {
            source_joints,
            source_bodies,
            ratio,
            constant,
            coordinate1,
            coordinate2,
        }
    }

    /// Returns the two source constraints in definition order.
    #[must_use]
    pub const fn source_joints(self) -> [JointId; 2] {
        self.source_joints
    }

    /// Returns derived bodies `[A, B, C, D]` in pinned solver order.
    #[must_use]
    pub const fn source_bodies(self) -> [BodyId; 4] {
        self.source_bodies
    }

    /// Returns the current ratio.
    #[must_use]
    pub const fn ratio(self) -> f32 {
        self.ratio
    }

    /// Returns the creation-time constraint constant.
    #[must_use]
    pub const fn constant(self) -> f32 {
        self.constant
    }

    /// Returns the current first source coordinate.
    #[must_use]
    pub const fn coordinate1(self) -> f32 {
        self.coordinate1
    }

    /// Returns the current second source coordinate.
    #[must_use]
    pub const fn coordinate2(self) -> f32 {
        self.coordinate2
    }
}

/// Owned semantic friction-joint state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrictionJointSnapshot {
    max_force: f32,
    max_torque: f32,
}

impl FrictionJointSnapshot {
    pub(crate) const fn new(max_force: f32, max_torque: f32) -> Self {
        Self {
            max_force,
            max_torque,
        }
    }
    /// Returns the maximum force.
    #[must_use]
    pub const fn max_force(self) -> f32 {
        self.max_force
    }
    /// Returns the maximum torque.
    #[must_use]
    pub const fn max_torque(self) -> f32 {
        self.max_torque
    }
}

/// Owned semantic rope-joint state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RopeJointSnapshot {
    max_length: f32,
    current_length: f32,
    limit_state: JointLimitState,
}

impl RopeJointSnapshot {
    pub(crate) const fn new(
        max_length: f32,
        current_length: f32,
        limit_state: JointLimitState,
    ) -> Self {
        Self {
            max_length,
            current_length,
            limit_state,
        }
    }
    /// Returns the maximum length.
    #[must_use]
    pub const fn max_length(self) -> f32 {
        self.max_length
    }
    /// Returns the current anchor separation.
    #[must_use]
    pub const fn current_length(self) -> f32 {
        self.current_length
    }
    /// Returns the unilateral limit state.
    #[must_use]
    pub const fn limit_state(self) -> JointLimitState {
        self.limit_state
    }
}

/// Owned semantic motor-joint state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotorJointSnapshot {
    linear_offset: Vec2,
    angular_offset: f32,
    max_force: f32,
    max_torque: f32,
    correction_factor: f32,
    linear_error: Vec2,
    angular_error: f32,
}

impl MotorJointSnapshot {
    #[allow(
        clippy::too_many_arguments,
        reason = "the snapshot mirrors one closed motor state"
    )]
    pub(crate) const fn new(
        linear_offset: Vec2,
        angular_offset: f32,
        max_force: f32,
        max_torque: f32,
        correction_factor: f32,
        linear_error: Vec2,
        angular_error: f32,
    ) -> Self {
        Self {
            linear_offset,
            angular_offset,
            max_force,
            max_torque,
            correction_factor,
            linear_error,
            angular_error,
        }
    }
    /// Returns the linear offset.
    #[must_use]
    pub const fn linear_offset(self) -> Vec2 {
        self.linear_offset
    }
    /// Returns the angular offset.
    #[must_use]
    pub const fn angular_offset(self) -> f32 {
        self.angular_offset
    }
    /// Returns the maximum force.
    #[must_use]
    pub const fn max_force(self) -> f32 {
        self.max_force
    }
    /// Returns the maximum torque.
    #[must_use]
    pub const fn max_torque(self) -> f32 {
        self.max_torque
    }
    /// Returns the correction factor.
    #[must_use]
    pub const fn correction_factor(self) -> f32 {
        self.correction_factor
    }
    /// Returns the last computed linear error.
    #[must_use]
    pub const fn linear_error(self) -> Vec2 {
        self.linear_error
    }
    /// Returns the last computed angular error.
    #[must_use]
    pub const fn angular_error(self) -> f32 {
        self.angular_error
    }
}
