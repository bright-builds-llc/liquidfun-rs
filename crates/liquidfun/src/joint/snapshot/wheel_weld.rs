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
