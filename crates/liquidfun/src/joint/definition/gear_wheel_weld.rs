use super::{
    BodyId, JointDefError, JointId, validate_bodies, validate_non_negative, validate_scalar,
    validate_vec,
};

/// Definition of a gear joint over two live revolute or prismatic joints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GearJointDef {
    joint1: JointId,
    joint2: JointId,
    ratio: f32,
    collide_connected: bool,
}

impl GearJointDef {
    /// Creates a gear definition with the pinned default ratio of `1`.
    ///
    /// The source joints are resolved and kind-checked atomically by [`crate::World`].
    ///
    /// # Errors
    ///
    /// Returns [`JointDefError::SameJoint`] when both source identities are equal.
    pub fn new(joint1: JointId, joint2: JointId) -> Result<Self, JointDefError> {
        if joint1 == joint2 {
            return Err(JointDefError::SameJoint);
        }
        Ok(Self {
            joint1,
            joint2,
            ratio: 1.0,
            collide_connected: false,
        })
    }

    /// Sets the finite gear ratio. Positive, negative, and zero ratios are accepted.
    ///
    /// # Errors
    ///
    /// Returns [`JointDefError::NonFiniteValue`] for a non-finite ratio.
    pub fn with_ratio(mut self, ratio: f32) -> Result<Self, JointDefError> {
        validate_scalar(ratio)?;
        self.ratio = ratio;
        Ok(self)
    }

    /// Chooses whether the two derived gear bodies may collide.
    #[must_use]
    pub const fn with_collide_connected(mut self, value: bool) -> Self {
        self.collide_connected = value;
        self
    }

    /// Returns the two source-joint identities.
    #[must_use]
    pub const fn source_joints(self) -> [JointId; 2] {
        [self.joint1, self.joint2]
    }

    /// Returns the configured ratio.
    #[must_use]
    pub const fn ratio(self) -> f32 {
        self.ratio
    }

    pub(crate) const fn collide_connected(self) -> bool {
        self.collide_connected
    }
}

/// Definition of a wheel joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelJointDef {
    body_a: BodyId,
    body_b: BodyId,
    collide_connected: bool,
    local_anchor_a: crate::math::Vec2,
    local_anchor_b: crate::math::Vec2,
    local_axis_a: crate::math::Vec2,
    enable_motor: bool,
    motor_speed: f32,
    max_motor_torque: f32,
    frequency: f32,
    damping_ratio: f32,
}

impl WheelJointDef {
    /// Creates the pinned default wheel configuration.
    ///
    /// # Errors
    ///
    /// Returns [`JointDefError::SameBody`] for identical endpoints.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Result<Self, JointDefError> {
        validate_bodies(body_a, body_b)?;
        Ok(Self {
            body_a,
            body_b,
            collide_connected: false,
            local_anchor_a: crate::math::Vec2::ZERO,
            local_anchor_b: crate::math::Vec2::ZERO,
            local_axis_a: crate::math::Vec2::new(1.0, 0.0),
            enable_motor: false,
            motor_speed: 0.0,
            max_motor_torque: 0.0,
            frequency: 2.0,
            damping_ratio: 0.7,
        })
    }

    /// Chooses whether the connected bodies may collide.
    #[must_use]
    pub const fn with_collide_connected(mut self, value: bool) -> Self {
        self.collide_connected = value;
        self
    }

    /// Sets local anchors and a normalized local suspension axis.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite anchors or an invalid axis.
    pub fn with_frame(
        mut self,
        local_anchor_a: crate::math::Vec2,
        local_anchor_b: crate::math::Vec2,
        mut local_axis_a: crate::math::Vec2,
    ) -> Result<Self, JointDefError> {
        validate_vec(local_anchor_a)?;
        validate_vec(local_anchor_b)?;
        validate_vec(local_axis_a)?;
        let length = local_axis_a.normalize();
        if length == 0.0 || !local_axis_a.is_valid() {
            return Err(JointDefError::InvalidAxis);
        }
        self.local_anchor_a = local_anchor_a;
        self.local_anchor_b = local_anchor_b;
        self.local_axis_a = local_axis_a;
        Ok(self)
    }

    /// Configures the rotational motor.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-finite speed or negative torque cap.
    pub fn with_motor(
        mut self,
        enabled: bool,
        speed: f32,
        max_torque: f32,
    ) -> Result<Self, JointDefError> {
        validate_scalar(speed)?;
        validate_non_negative(max_torque)?;
        self.enable_motor = enabled;
        self.motor_speed = speed;
        self.max_motor_torque = max_torque;
        Ok(self)
    }

    /// Configures the non-negative spring frequency and damping ratio.
    ///
    /// # Errors
    ///
    /// Returns an error when either value is negative or non-finite.
    pub fn with_spring(
        mut self,
        frequency: f32,
        damping_ratio: f32,
    ) -> Result<Self, JointDefError> {
        validate_non_negative(frequency)?;
        validate_non_negative(damping_ratio)?;
        self.frequency = frequency;
        self.damping_ratio = damping_ratio;
        Ok(self)
    }

    /// Returns the local anchor on body A.
    #[must_use]
    pub const fn local_anchor_a(self) -> crate::math::Vec2 {
        self.local_anchor_a
    }
    /// Returns the local anchor on body B.
    #[must_use]
    pub const fn local_anchor_b(self) -> crate::math::Vec2 {
        self.local_anchor_b
    }
    /// Returns the normalized local suspension axis on body A.
    #[must_use]
    pub const fn local_axis_a(self) -> crate::math::Vec2 {
        self.local_axis_a
    }
    /// Returns whether the motor is enabled.
    #[must_use]
    pub const fn is_motor_enabled(self) -> bool {
        self.enable_motor
    }
    /// Returns the target motor speed.
    #[must_use]
    pub const fn motor_speed(self) -> f32 {
        self.motor_speed
    }
    /// Returns the maximum motor torque.
    #[must_use]
    pub const fn max_motor_torque(self) -> f32 {
        self.max_motor_torque
    }
    /// Returns the spring frequency in hertz.
    #[must_use]
    pub const fn frequency(self) -> f32 {
        self.frequency
    }
    /// Returns the spring damping ratio.
    #[must_use]
    pub const fn damping_ratio(self) -> f32 {
        self.damping_ratio
    }
    pub(crate) const fn bodies(self) -> [BodyId; 2] {
        [self.body_a, self.body_b]
    }
    pub(crate) const fn collide_connected(self) -> bool {
        self.collide_connected
    }
}

/// Definition of a weld joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeldJointDef {
    body_a: BodyId,
    body_b: BodyId,
    collide_connected: bool,
    local_anchor_a: crate::math::Vec2,
    local_anchor_b: crate::math::Vec2,
    reference_angle: f32,
    frequency: f32,
    damping_ratio: f32,
}

impl WeldJointDef {
    /// Creates the pinned default rigid weld configuration.
    ///
    /// # Errors
    ///
    /// Returns [`JointDefError::SameBody`] for identical endpoints.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Result<Self, JointDefError> {
        validate_bodies(body_a, body_b)?;
        Ok(Self {
            body_a,
            body_b,
            collide_connected: false,
            local_anchor_a: crate::math::Vec2::ZERO,
            local_anchor_b: crate::math::Vec2::ZERO,
            reference_angle: 0.0,
            frequency: 0.0,
            damping_ratio: 0.0,
        })
    }

    /// Chooses whether the connected bodies may collide.
    #[must_use]
    pub const fn with_collide_connected(mut self, value: bool) -> Self {
        self.collide_connected = value;
        self
    }

    /// Sets the two local anchors and reference angle.
    ///
    /// # Errors
    ///
    /// Returns an error when any value is non-finite.
    pub fn with_frame(
        mut self,
        local_anchor_a: crate::math::Vec2,
        local_anchor_b: crate::math::Vec2,
        reference_angle: f32,
    ) -> Result<Self, JointDefError> {
        validate_vec(local_anchor_a)?;
        validate_vec(local_anchor_b)?;
        validate_scalar(reference_angle)?;
        self.local_anchor_a = local_anchor_a;
        self.local_anchor_b = local_anchor_b;
        self.reference_angle = reference_angle;
        Ok(self)
    }

    /// Sets the non-negative rotational spring frequency.
    ///
    /// # Errors
    ///
    /// Returns an error for a negative or non-finite value.
    pub fn with_frequency(mut self, frequency: f32) -> Result<Self, JointDefError> {
        validate_non_negative(frequency)?;
        self.frequency = frequency;
        Ok(self)
    }

    /// Sets the non-negative rotational damping ratio.
    ///
    /// # Errors
    ///
    /// Returns an error for a negative or non-finite value.
    pub fn with_damping_ratio(mut self, damping_ratio: f32) -> Result<Self, JointDefError> {
        validate_non_negative(damping_ratio)?;
        self.damping_ratio = damping_ratio;
        Ok(self)
    }

    /// Returns the local anchor on body A.
    #[must_use]
    pub const fn local_anchor_a(self) -> crate::math::Vec2 {
        self.local_anchor_a
    }
    /// Returns the local anchor on body B.
    #[must_use]
    pub const fn local_anchor_b(self) -> crate::math::Vec2 {
        self.local_anchor_b
    }
    /// Returns the reference angle.
    #[must_use]
    pub const fn reference_angle(self) -> f32 {
        self.reference_angle
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
    pub(crate) const fn bodies(self) -> [BodyId; 2] {
        [self.body_a, self.body_b]
    }
    pub(crate) const fn collide_connected(self) -> bool {
        self.collide_connected
    }
}
