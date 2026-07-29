use super::{JointLimitState, Vec2};

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
