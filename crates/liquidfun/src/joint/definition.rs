//! Checked, owned joint definitions.

use std::error::Error;
use std::fmt;

use crate::BodyId;

/// A failure while constructing a reusable joint definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JointDefError {
    /// A joint cannot connect a body to itself.
    SameBody,
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

macro_rules! basic_joint_definition {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            body_a: BodyId,
            body_b: BodyId,
            collide_connected: bool,
        }

        impl $name {
            /// Creates a checked definition with collision disabled between its bodies.
            ///
            /// # Errors
            ///
            /// Returns [`JointDefError::SameBody`] when both endpoints are identical.
            pub fn new(body_a: BodyId, body_b: BodyId) -> Result<Self, JointDefError> {
                if body_a == body_b {
                    return Err(JointDefError::SameBody);
                }
                Ok(Self {
                    body_a,
                    body_b,
                    collide_connected: false,
                })
            }

            /// Chooses whether the connected bodies may collide.
            #[must_use]
            pub const fn with_collide_connected(mut self, collide_connected: bool) -> Self {
                self.collide_connected = collide_connected;
                self
            }

            pub(crate) const fn bodies(self) -> [BodyId; 2] {
                [self.body_a, self.body_b]
            }

            pub(crate) const fn collide_connected(self) -> bool {
                self.collide_connected
            }
        }
    };
}

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
basic_joint_definition!(GearJointDef, "Definition of a gear joint.");

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
basic_joint_definition!(FrictionJointDef, "Definition of a friction joint.");
basic_joint_definition!(RopeJointDef, "Definition of a unilateral rope joint.");
basic_joint_definition!(MotorJointDef, "Definition of a motor joint.");

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
    /// Returns the connected body identities.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        match self {
            Self::Revolute(definition) => definition.bodies(),
            Self::Prismatic(definition) => definition.bodies(),
            Self::Distance(definition) => definition.bodies(),
            Self::Pulley(definition) => definition.bodies(),
            Self::Mouse(definition) => definition.bodies(),
            Self::Gear(definition) => definition.bodies(),
            Self::Wheel(definition) => definition.bodies(),
            Self::Weld(definition) => definition.bodies(),
            Self::Friction(definition) => definition.bodies(),
            Self::Rope(definition) => definition.bodies(),
            Self::Motor(definition) => definition.bodies(),
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
