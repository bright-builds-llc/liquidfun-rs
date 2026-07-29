use super::Vec2;

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
