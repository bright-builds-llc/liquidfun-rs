use super::{BitAnd, BitAndAssign, BitOr, BitOrAssign, PRIVATE_GROUP_FLAG_MASK};

/// Public particle-group behavior flags with the pinned upstream bit values.
///
/// Unknown public bits are retained by [`Self::from_bits_retain`] for the same
/// forward-compatible round-trip policy as [`crate::ParticleFlags`]. The two
/// upstream-private lifecycle/cache bits are always removed and cannot cross
/// this public boundary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ParticleGroupFlags(u32);

impl ParticleGroupFlags {
    /// Prevent other particles from overlapping or leaking through the group.
    pub const SOLID: Self = Self(0x0001);
    /// Preserve the group's shape through rigid particle motion.
    pub const RIGID: Self = Self(0x0002);
    /// Retain the group identity when its final particle is removed.
    pub const CAN_BE_EMPTY: Self = Self(0x0004);

    /// Returns a value with no public group behavior enabled.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns the union of all named public group behaviors.
    #[must_use]
    pub const fn all() -> Self {
        Self(Self::SOLID.0 | Self::RIGID.0 | Self::CAN_BE_EMPTY.0)
    }

    /// Creates flags while retaining unknown public bits.
    ///
    /// Upstream-private lifecycle/cache bits are stripped even when present in
    /// the input, so they are neither constructible nor inspectable here.
    #[must_use]
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits & !PRIVATE_GROUP_FLAG_MASK)
    }

    /// Returns exact known and retained unknown public bits.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns whether no public or retained unknown bits are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns whether every bit in `other` is present.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for ParticleGroupFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_bits_retain(self.0 | rhs.0)
    }
}

impl BitOrAssign for ParticleGroupFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitAnd for ParticleGroupFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_bits_retain(self.0 & rhs.0)
    }
}

impl BitAndAssign for ParticleGroupFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}
