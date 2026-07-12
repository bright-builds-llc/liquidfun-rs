use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{FloatBits, Sha256Hex, ToleranceProfileVersion};

mod collision_policy;
mod policy;
mod rigid_policy;

pub use collision_policy::*;
pub use policy::*;
pub use rigid_policy::*;

/// Closed policy set for authoritative floating-point observables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FloatPolicy {
    /// Compare the complete IEEE-754 bit pattern.
    ExactBits,
    /// Accept an absolute difference no larger than the bit-encoded threshold.
    Absolute {
        /// Authoritative `f32` bits for the maximum absolute difference.
        max_bits: FloatBits,
    },
    /// Accept when either absolute or relative difference is within its threshold.
    AbsoluteRelative {
        /// Authoritative `f32` bits for the maximum absolute difference.
        absolute_bits: FloatBits,
        /// Authoritative `f32` bits for the maximum relative difference.
        relative_bits: FloatBits,
    },
    /// Accept values separated by no more than the named ULP count.
    Ulps {
        /// Maximum representable-value distance.
        max: u32,
    },
}

/// Closed policy for discrete semantic fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscretePolicy {
    /// Require exact equality.
    Exact,
}

/// Closed policy for collection semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionPolicy {
    /// Preserve and compare solver-significant order.
    Ordered,
    /// Compare unique membership using stable semantic keys.
    Set,
    /// Compare membership and multiplicity using stable semantic keys.
    Multiset,
}

/// Versioned field-specific policy profile for the phase-2 trace schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToleranceProfile {
    profile_id: &'static str,
    version: ToleranceProfileVersion,
    simulation_time: FloatPolicy,
    world_counts: DiscretePolicy,
    checkpoints: CollectionPolicy,
    profile_sha256: Sha256Hex,
}

impl ToleranceProfile {
    /// Returns the immutable exact phase-2 policy profile.
    #[must_use]
    pub fn phase2_v1() -> Self {
        let version = ToleranceProfileVersion::CURRENT;
        let simulation_time = FloatPolicy::ExactBits;
        let world_counts = DiscretePolicy::Exact;
        let checkpoints = CollectionPolicy::Ordered;
        let canonical = concat!(
            "profile_id=phase2-v1\n",
            "version=1\n",
            "simulation_time=exact_bits\n",
            "world_counts=exact\n",
            "checkpoints=ordered\n"
        );
        Self {
            profile_id: "phase2-v1",
            version,
            simulation_time,
            world_counts,
            checkpoints,
            profile_sha256: Sha256Hex::from_digest(Sha256::digest(canonical).into()),
        }
    }

    /// Returns the stable profile identifier.
    #[must_use]
    pub const fn profile_id(&self) -> &'static str {
        self.profile_id
    }

    /// Returns the independently versioned policy schema.
    #[must_use]
    pub const fn version(&self) -> ToleranceProfileVersion {
        self.version
    }

    /// Returns the explicit simulation-time policy.
    #[must_use]
    pub const fn simulation_time(&self) -> FloatPolicy {
        self.simulation_time
    }

    /// Returns the explicit world-count policy.
    #[must_use]
    pub const fn world_counts(&self) -> DiscretePolicy {
        self.world_counts
    }

    /// Returns the explicit checkpoint collection semantics.
    #[must_use]
    pub const fn checkpoints(&self) -> CollectionPolicy {
        self.checkpoints
    }

    /// Returns the deterministic identity of every profile field.
    #[must_use]
    pub const fn profile_sha256(&self) -> &Sha256Hex {
        &self.profile_sha256
    }

    /// Returns typed non-authoritative examples used by comparator policy tests.
    #[must_use]
    pub fn synthetic_float_policies() -> [FloatPolicy; 3] {
        [
            FloatPolicy::Absolute {
                max_bits: FloatBits::new(1.0_f32.to_bits()),
            },
            FloatPolicy::AbsoluteRelative {
                absolute_bits: FloatBits::new(1.0_f32.to_bits()),
                relative_bits: FloatBits::new(0.25_f32.to_bits()),
            },
            FloatPolicy::Ulps { max: 4 },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{CollectionPolicy, DiscretePolicy, FloatPolicy, ToleranceProfile};
    use crate::FloatBits;

    #[test]
    fn phase2_tolerance_uses_exact_time_and_discrete_counts() {
        // Arrange and Act
        let profile = ToleranceProfile::phase2_v1();

        // Assert
        assert_eq!(profile.simulation_time(), FloatPolicy::ExactBits);
        assert_eq!(profile.world_counts(), DiscretePolicy::Exact);
        assert_eq!(profile.checkpoints(), CollectionPolicy::Ordered);
    }

    #[test]
    fn typed_float_policies_preserve_bit_encoded_thresholds() {
        // Arrange
        let absolute = FloatBits::new(1.0_f32.to_bits());
        let relative = FloatBits::new(0.25_f32.to_bits());

        // Act
        let policies = [
            FloatPolicy::Absolute { max_bits: absolute },
            FloatPolicy::AbsoluteRelative {
                absolute_bits: absolute,
                relative_bits: relative,
            },
            FloatPolicy::Ulps { max: 4 },
        ];

        // Assert
        assert!(matches!(policies[0], FloatPolicy::Absolute { .. }));
        assert!(matches!(policies[1], FloatPolicy::AbsoluteRelative { .. }));
        assert_eq!(policies[2], FloatPolicy::Ulps { max: 4 });
    }
}
