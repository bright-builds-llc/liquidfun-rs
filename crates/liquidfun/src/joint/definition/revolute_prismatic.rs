use super::{
    BodyId, JointDefError, validate_bodies, validate_non_negative, validate_range, validate_scalar,
    validate_vec,
};

/// Definition of a revolute joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RevoluteJointDef {
    body_a: BodyId,
    body_b: BodyId,
    collide_connected: bool,
    local_anchor_a: crate::math::Vec2,
    local_anchor_b: crate::math::Vec2,
    reference_angle: f32,
    enable_limit: bool,
    lower_angle: f32,
    upper_angle: f32,
    enable_motor: bool,
    motor_speed: f32,
    max_motor_torque: f32,
}

impl RevoluteJointDef {
    /// Creates the pinned default revolute configuration.
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
            enable_limit: false,
            lower_angle: 0.0,
            upper_angle: 0.0,
            enable_motor: false,
            motor_speed: 0.0,
            max_motor_torque: 0.0,
        })
    }

    /// Chooses whether the connected bodies may collide.
    #[must_use]
    pub const fn with_collide_connected(mut self, value: bool) -> Self {
        self.collide_connected = value;
        self
    }

    /// Sets local anchors and the reference angle.
    ///
    /// # Errors
    ///
    /// Returns an error when any coordinate or the angle is non-finite.
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

    /// Configures angular limits.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite or inverted limits.
    pub fn with_limits(
        mut self,
        enabled: bool,
        lower: f32,
        upper: f32,
    ) -> Result<Self, JointDefError> {
        validate_range(lower, upper)?;
        self.enable_limit = enabled;
        self.lower_angle = lower;
        self.upper_angle = upper;
        Ok(self)
    }

    /// Configures the angular motor.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-finite speed or negative/non-finite torque.
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
    /// Returns whether angular limits are enabled.
    #[must_use]
    pub const fn is_limit_enabled(self) -> bool {
        self.enable_limit
    }
    /// Returns the lower angular limit.
    #[must_use]
    pub const fn lower_angle(self) -> f32 {
        self.lower_angle
    }
    /// Returns the upper angular limit.
    #[must_use]
    pub const fn upper_angle(self) -> f32 {
        self.upper_angle
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
    pub(crate) const fn bodies(self) -> [BodyId; 2] {
        [self.body_a, self.body_b]
    }
    pub(crate) const fn collide_connected(self) -> bool {
        self.collide_connected
    }
}

/// Definition of a prismatic joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrismaticJointDef {
    body_a: BodyId,
    body_b: BodyId,
    collide_connected: bool,
    local_anchor_a: crate::math::Vec2,
    local_anchor_b: crate::math::Vec2,
    local_axis_a: crate::math::Vec2,
    reference_angle: f32,
    enable_limit: bool,
    lower_translation: f32,
    upper_translation: f32,
    enable_motor: bool,
    motor_speed: f32,
    max_motor_force: f32,
}

impl PrismaticJointDef {
    /// Creates the pinned default prismatic configuration.
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
            reference_angle: 0.0,
            enable_limit: false,
            lower_translation: 0.0,
            upper_translation: 0.0,
            enable_motor: false,
            motor_speed: 0.0,
            max_motor_force: 0.0,
        })
    }

    /// Chooses whether the connected bodies may collide.
    #[must_use]
    pub const fn with_collide_connected(mut self, value: bool) -> Self {
        self.collide_connected = value;
        self
    }

    /// Sets local anchors, a normalized local axis, and the reference angle.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite values or a zero-length axis.
    pub fn with_frame(
        mut self,
        local_anchor_a: crate::math::Vec2,
        local_anchor_b: crate::math::Vec2,
        mut local_axis_a: crate::math::Vec2,
        reference_angle: f32,
    ) -> Result<Self, JointDefError> {
        validate_vec(local_anchor_a)?;
        validate_vec(local_anchor_b)?;
        validate_vec(local_axis_a)?;
        validate_scalar(reference_angle)?;
        let length = local_axis_a.normalize();
        if length == 0.0 || !local_axis_a.is_valid() {
            return Err(JointDefError::InvalidAxis);
        }
        self.local_anchor_a = local_anchor_a;
        self.local_anchor_b = local_anchor_b;
        self.local_axis_a = local_axis_a;
        self.reference_angle = reference_angle;
        Ok(self)
    }

    /// Configures translation limits.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite or inverted limits.
    pub fn with_limits(
        mut self,
        enabled: bool,
        lower: f32,
        upper: f32,
    ) -> Result<Self, JointDefError> {
        validate_range(lower, upper)?;
        self.enable_limit = enabled;
        self.lower_translation = lower;
        self.upper_translation = upper;
        Ok(self)
    }

    /// Configures the linear motor.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-finite speed or negative/non-finite force.
    pub fn with_motor(
        mut self,
        enabled: bool,
        speed: f32,
        max_force: f32,
    ) -> Result<Self, JointDefError> {
        validate_scalar(speed)?;
        validate_non_negative(max_force)?;
        self.enable_motor = enabled;
        self.motor_speed = speed;
        self.max_motor_force = max_force;
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
    /// Returns the normalized local axis on body A.
    #[must_use]
    pub const fn local_axis_a(self) -> crate::math::Vec2 {
        self.local_axis_a
    }
    /// Returns the reference angle.
    #[must_use]
    pub const fn reference_angle(self) -> f32 {
        self.reference_angle
    }
    /// Returns whether translation limits are enabled.
    #[must_use]
    pub const fn is_limit_enabled(self) -> bool {
        self.enable_limit
    }
    /// Returns the lower translation limit.
    #[must_use]
    pub const fn lower_translation(self) -> f32 {
        self.lower_translation
    }
    /// Returns the upper translation limit.
    #[must_use]
    pub const fn upper_translation(self) -> f32 {
        self.upper_translation
    }
    /// Returns whether the motor is enabled.
    #[must_use]
    pub const fn is_motor_enabled(self) -> bool {
        self.enable_motor
    }
    /// Returns target motor speed.
    #[must_use]
    pub const fn motor_speed(self) -> f32 {
        self.motor_speed
    }
    /// Returns maximum motor force.
    #[must_use]
    pub const fn max_motor_force(self) -> f32 {
        self.max_motor_force
    }
    pub(crate) const fn bodies(self) -> [BodyId; 2] {
        [self.body_a, self.body_b]
    }
    pub(crate) const fn collide_connected(self) -> bool {
        self.collide_connected
    }
}
