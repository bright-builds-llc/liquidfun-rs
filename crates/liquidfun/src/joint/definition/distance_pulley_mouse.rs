use super::{
    BodyId, JointDefError, validate_bodies, validate_non_negative, validate_positive,
    validate_scalar, validate_vec,
};

/// Definition of a fixed or soft distance joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistanceJointDef {
    body_a: BodyId,
    body_b: BodyId,
    collide_connected: bool,
    local_anchor_a: crate::math::Vec2,
    local_anchor_b: crate::math::Vec2,
    length: f32,
    frequency: f32,
    damping_ratio: f32,
}

impl DistanceJointDef {
    /// Creates the pinned default distance-joint configuration.
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
            length: 1.0,
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

    /// Sets the two local anchors.
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

    /// Sets the positive natural length.
    ///
    /// # Errors
    ///
    /// Returns an error when `length` is non-finite or not positive.
    pub fn with_length(mut self, length: f32) -> Result<Self, JointDefError> {
        validate_positive(length)?;
        self.length = length;
        Ok(self)
    }

    /// Sets the non-negative frequency in hertz.
    ///
    /// # Errors
    ///
    /// Returns an error when `frequency` is non-finite or negative.
    pub fn with_frequency(mut self, frequency: f32) -> Result<Self, JointDefError> {
        validate_non_negative(frequency)?;
        self.frequency = frequency;
        Ok(self)
    }

    /// Sets the non-negative damping ratio.
    ///
    /// # Errors
    ///
    /// Returns an error when `damping_ratio` is non-finite or negative.
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
    /// Returns the natural length.
    #[must_use]
    pub const fn length(self) -> f32 {
        self.length
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

/// Definition of a pulley joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PulleyJointDef {
    body_a: BodyId,
    body_b: BodyId,
    collide_connected: bool,
    ground_anchor_a: crate::math::Vec2,
    ground_anchor_b: crate::math::Vec2,
    local_anchor_a: crate::math::Vec2,
    local_anchor_b: crate::math::Vec2,
    length_a: f32,
    length_b: f32,
    ratio: f32,
}

impl PulleyJointDef {
    /// Creates a checked source-shaped pulley configuration.
    ///
    /// # Errors
    ///
    /// Returns [`JointDefError::SameBody`] for identical endpoints.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Result<Self, JointDefError> {
        validate_bodies(body_a, body_b)?;
        Ok(Self {
            body_a,
            body_b,
            collide_connected: true,
            ground_anchor_a: crate::math::Vec2::new(-1.0, 1.0),
            ground_anchor_b: crate::math::Vec2::new(1.0, 1.0),
            local_anchor_a: crate::math::Vec2::new(-1.0, 0.0),
            local_anchor_b: crate::math::Vec2::new(1.0, 0.0),
            length_a: 1.0,
            length_b: 1.0,
            ratio: 1.0,
        })
    }

    /// Chooses whether the connected bodies may collide.
    #[must_use]
    pub const fn with_collide_connected(mut self, value: bool) -> Self {
        self.collide_connected = value;
        self
    }

    /// Sets all invariant-bearing pulley geometry.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite anchors or non-positive lengths or ratio.
    #[allow(
        clippy::too_many_arguments,
        reason = "the pinned pulley definition is one atomic geometry"
    )]
    pub fn with_geometry(
        mut self,
        ground_anchor_a: crate::math::Vec2,
        ground_anchor_b: crate::math::Vec2,
        local_anchor_a: crate::math::Vec2,
        local_anchor_b: crate::math::Vec2,
        length_a: f32,
        length_b: f32,
        ratio: f32,
    ) -> Result<Self, JointDefError> {
        validate_vec(ground_anchor_a)?;
        validate_vec(ground_anchor_b)?;
        validate_vec(local_anchor_a)?;
        validate_vec(local_anchor_b)?;
        validate_positive(length_a)?;
        validate_positive(length_b)?;
        validate_positive(ratio)?;
        let constant = length_a + ratio * length_b;
        validate_scalar(constant)?;
        self.ground_anchor_a = ground_anchor_a;
        self.ground_anchor_b = ground_anchor_b;
        self.local_anchor_a = local_anchor_a;
        self.local_anchor_b = local_anchor_b;
        self.length_a = length_a;
        self.length_b = length_b;
        self.ratio = ratio;
        Ok(self)
    }

    /// Returns the fixed world-space ground anchor on side A.
    #[must_use]
    pub const fn ground_anchor_a(self) -> crate::math::Vec2 {
        self.ground_anchor_a
    }
    /// Returns the fixed world-space ground anchor on side B.
    #[must_use]
    pub const fn ground_anchor_b(self) -> crate::math::Vec2 {
        self.ground_anchor_b
    }
    /// Returns the local body-A anchor.
    #[must_use]
    pub const fn local_anchor_a(self) -> crate::math::Vec2 {
        self.local_anchor_a
    }
    /// Returns the local body-B anchor.
    #[must_use]
    pub const fn local_anchor_b(self) -> crate::math::Vec2 {
        self.local_anchor_b
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
    /// Returns the pulley ratio.
    #[must_use]
    pub const fn ratio(self) -> f32 {
        self.ratio
    }
    /// Returns the source constant `length_a + ratio * length_b`.
    #[must_use]
    pub fn constant(self) -> f32 {
        self.length_a + self.ratio * self.length_b
    }
    pub(crate) const fn bodies(self) -> [BodyId; 2] {
        [self.body_a, self.body_b]
    }
    pub(crate) const fn collide_connected(self) -> bool {
        self.collide_connected
    }
}

/// Definition of a mouse joint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseJointDef {
    body_a: BodyId,
    body_b: BodyId,
    collide_connected: bool,
    target: crate::math::Vec2,
    max_force: f32,
    frequency: f32,
    damping_ratio: f32,
}

impl MouseJointDef {
    /// Creates the pinned default mouse-joint configuration.
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
            target: crate::math::Vec2::ZERO,
            max_force: 0.0,
            frequency: 5.0,
            damping_ratio: 0.7,
        })
    }

    /// Chooses whether the connected bodies may collide.
    #[must_use]
    pub const fn with_collide_connected(mut self, value: bool) -> Self {
        self.collide_connected = value;
        self
    }
    /// Sets the finite world-space target.
    ///
    /// # Errors
    ///
    /// Returns an error when the target is non-finite.
    pub fn with_target(mut self, target: crate::math::Vec2) -> Result<Self, JointDefError> {
        validate_vec(target)?;
        self.target = target;
        Ok(self)
    }
    /// Sets the non-negative force cap.
    ///
    /// # Errors
    ///
    /// Returns an error when the force is non-finite or negative.
    pub fn with_max_force(mut self, max_force: f32) -> Result<Self, JointDefError> {
        validate_non_negative(max_force)?;
        self.max_force = max_force;
        Ok(self)
    }
    /// Sets the non-negative frequency in hertz.
    ///
    /// # Errors
    ///
    /// Returns an error when the frequency is non-finite or negative.
    pub fn with_frequency(mut self, frequency: f32) -> Result<Self, JointDefError> {
        validate_non_negative(frequency)?;
        self.frequency = frequency;
        Ok(self)
    }
    /// Sets the non-negative damping ratio.
    ///
    /// # Errors
    ///
    /// Returns an error when the damping ratio is non-finite or negative.
    pub fn with_damping_ratio(mut self, damping_ratio: f32) -> Result<Self, JointDefError> {
        validate_non_negative(damping_ratio)?;
        self.damping_ratio = damping_ratio;
        Ok(self)
    }
    /// Returns the world-space target.
    #[must_use]
    pub const fn target(self) -> crate::math::Vec2 {
        self.target
    }
    /// Returns the force cap.
    #[must_use]
    pub const fn max_force(self) -> f32 {
        self.max_force
    }
    /// Returns the frequency in hertz.
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
