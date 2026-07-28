use super::{
    BodyId, JointDefError, validate_bodies, validate_non_negative, validate_positive,
    validate_scalar, validate_vec,
};

/// Definition of a capped friction joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrictionJointDef {
    body_a: BodyId,
    body_b: BodyId,
    collide_connected: bool,
    local_anchor_a: crate::math::Vec2,
    local_anchor_b: crate::math::Vec2,
    max_force: f32,
    max_torque: f32,
}

impl FrictionJointDef {
    /// Creates the pinned default friction configuration.
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
            max_force: 0.0,
            max_torque: 0.0,
        })
    }
    /// Chooses whether the connected bodies may collide.
    #[must_use]
    pub const fn with_collide_connected(mut self, value: bool) -> Self {
        self.collide_connected = value;
        self
    }
    /// Sets both finite local anchors.
    ///
    /// # Errors
    ///
    /// Returns an error when either anchor is non-finite.
    pub fn with_anchors(
        mut self,
        local_anchor_a: crate::math::Vec2,
        local_anchor_b: crate::math::Vec2,
    ) -> Result<Self, JointDefError> {
        validate_vec(local_anchor_a)?;
        validate_vec(local_anchor_b)?;
        self.local_anchor_a = local_anchor_a;
        self.local_anchor_b = local_anchor_b;
        Ok(self)
    }
    /// Sets the non-negative force cap.
    ///
    /// # Errors
    ///
    /// Returns an error for a negative or non-finite force.
    pub fn with_max_force(mut self, value: f32) -> Result<Self, JointDefError> {
        validate_non_negative(value)?;
        self.max_force = value;
        Ok(self)
    }
    /// Sets the non-negative torque cap.
    ///
    /// # Errors
    ///
    /// Returns an error for a negative or non-finite torque.
    pub fn with_max_torque(mut self, value: f32) -> Result<Self, JointDefError> {
        validate_non_negative(value)?;
        self.max_torque = value;
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
    pub(crate) const fn bodies(self) -> [BodyId; 2] {
        [self.body_a, self.body_b]
    }
    pub(crate) const fn collide_connected(self) -> bool {
        self.collide_connected
    }
}

/// Definition of a unilateral rope joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RopeJointDef {
    body_a: BodyId,
    body_b: BodyId,
    collide_connected: bool,
    local_anchor_a: crate::math::Vec2,
    local_anchor_b: crate::math::Vec2,
    max_length: f32,
}

impl RopeJointDef {
    /// Creates a checked source-shaped rope-joint configuration.
    ///
    /// The checked Rust default uses a positive maximum length so invalid
    /// inert source defaults cannot enter a live world.
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
            local_anchor_a: crate::math::Vec2::new(-1.0, 0.0),
            local_anchor_b: crate::math::Vec2::new(1.0, 0.0),
            max_length: 1.0,
        })
    }
    /// Chooses whether the connected bodies may collide.
    #[must_use]
    pub const fn with_collide_connected(mut self, value: bool) -> Self {
        self.collide_connected = value;
        self
    }
    /// Sets both finite local anchors.
    ///
    /// # Errors
    ///
    /// Returns an error when either anchor is non-finite.
    pub fn with_anchors(
        mut self,
        local_anchor_a: crate::math::Vec2,
        local_anchor_b: crate::math::Vec2,
    ) -> Result<Self, JointDefError> {
        validate_vec(local_anchor_a)?;
        validate_vec(local_anchor_b)?;
        self.local_anchor_a = local_anchor_a;
        self.local_anchor_b = local_anchor_b;
        Ok(self)
    }
    /// Sets the strictly positive maximum length.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-positive or non-finite length.
    pub fn with_max_length(mut self, value: f32) -> Result<Self, JointDefError> {
        validate_positive(value)?;
        self.max_length = value;
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
    /// Returns the maximum length.
    #[must_use]
    pub const fn max_length(self) -> f32 {
        self.max_length
    }
    pub(crate) const fn bodies(self) -> [BodyId; 2] {
        [self.body_a, self.body_b]
    }
    pub(crate) const fn collide_connected(self) -> bool {
        self.collide_connected
    }
}

/// Definition of a capped relative-motion motor joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotorJointDef {
    body_a: BodyId,
    body_b: BodyId,
    collide_connected: bool,
    linear_offset: crate::math::Vec2,
    angular_offset: f32,
    max_force: f32,
    max_torque: f32,
    correction_factor: f32,
}

impl MotorJointDef {
    /// Creates the pinned default motor-joint configuration.
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
            linear_offset: crate::math::Vec2::ZERO,
            angular_offset: 0.0,
            max_force: 1.0,
            max_torque: 1.0,
            correction_factor: 0.3,
        })
    }
    /// Chooses whether the connected bodies may collide.
    #[must_use]
    pub const fn with_collide_connected(mut self, value: bool) -> Self {
        self.collide_connected = value;
        self
    }
    /// Sets the finite linear and angular offsets.
    ///
    /// # Errors
    ///
    /// Returns an error when either value is non-finite.
    pub fn with_offsets(
        mut self,
        linear: crate::math::Vec2,
        angular: f32,
    ) -> Result<Self, JointDefError> {
        validate_vec(linear)?;
        validate_scalar(angular)?;
        self.linear_offset = linear;
        self.angular_offset = angular;
        Ok(self)
    }
    /// Sets non-negative force and torque caps.
    ///
    /// # Errors
    ///
    /// Returns an error when either cap is negative or non-finite.
    pub fn with_caps(mut self, max_force: f32, max_torque: f32) -> Result<Self, JointDefError> {
        validate_non_negative(max_force)?;
        validate_non_negative(max_torque)?;
        self.max_force = max_force;
        self.max_torque = max_torque;
        Ok(self)
    }
    /// Sets the correction factor in the inclusive range zero through one.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is non-finite or outside the range.
    pub fn with_correction_factor(mut self, value: f32) -> Result<Self, JointDefError> {
        validate_scalar(value)?;
        if !(0.0..=1.0).contains(&value) {
            return Err(JointDefError::InvalidRange);
        }
        self.correction_factor = value;
        Ok(self)
    }
    /// Returns the linear offset in body A's frame.
    #[must_use]
    pub const fn linear_offset(self) -> crate::math::Vec2 {
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
    pub(crate) const fn bodies(self) -> [BodyId; 2] {
        [self.body_a, self.body_b]
    }
    pub(crate) const fn collide_connected(self) -> bool {
        self.collide_connected
    }
}
