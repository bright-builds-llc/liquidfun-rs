//! Engine-neutral protocol support for `liquidfun-rs` differential testing.

#![forbid(unsafe_code)]

mod catalog;
mod codec;
mod failure;
mod float_bits;
mod ids;
mod limits;
mod provenance;
mod scenario;
#[cfg(test)]
mod schema;
mod tolerance;
mod trace;

pub use catalog::*;
pub use codec::*;
pub use failure::*;
pub use float_bits::FloatBits;
pub use ids::*;
pub use limits::HarnessLimits;
pub use provenance::*;
pub use scenario::*;
pub use tolerance::*;
pub use trace::*;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Error returned when a protocol contract receives an unsupported version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unsupported {axis} version {value}; supported version is 1")]
pub struct UnsupportedVersionError {
    axis: &'static str,
    value: u32,
}

impl UnsupportedVersionError {
    /// Returns the unsupported value without coercion.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.value
    }
}

macro_rules! version_type {
    ($name:ident, $axis:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            /// The only version supported by the phase-2 contract.
            pub const SUPPORTED: u32 = 1;

            /// The validated current phase-2 version.
            pub const CURRENT: Self = Self(Self::SUPPORTED);

            /// Validates a raw version value.
            ///
            /// # Errors
            ///
            /// Returns [`UnsupportedVersionError`] unless `value` is exactly version 1.
            pub const fn new(value: u32) -> Result<Self, UnsupportedVersionError> {
                if value == Self::SUPPORTED {
                    return Ok(Self(value));
                }

                Err(UnsupportedVersionError { axis: $axis, value })
            }

            /// Returns the validated integer version.
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }
        }

        impl TryFrom<u32> for $name {
            type Error = UnsupportedVersionError;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_u32(self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = u32::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

version_type!(
    ProtocolVersion,
    "protocol",
    "Validated transport and framing version."
);
version_type!(
    ScenarioSchemaVersion,
    "scenario schema",
    "Validated semantic scenario schema version."
);
version_type!(
    TraceSchemaVersion,
    "trace schema",
    "Validated semantic trace schema version."
);
version_type!(
    ToleranceProfileVersion,
    "tolerance profile",
    "Validated comparison tolerance profile version."
);

#[cfg(test)]
mod tests {
    use super::{
        ProtocolVersion, ScenarioSchemaVersion, ToleranceProfileVersion, TraceSchemaVersion,
    };

    #[test]
    fn version_types_accept_the_supported_version() {
        // Arrange
        let supported = 1;

        // Act
        let protocol = ProtocolVersion::new(supported);
        let scenario = ScenarioSchemaVersion::new(supported);
        let trace = TraceSchemaVersion::new(supported);
        let tolerance = ToleranceProfileVersion::new(supported);

        // Assert
        assert_eq!(protocol.map(ProtocolVersion::get), Ok(supported));
        assert_eq!(scenario.map(ScenarioSchemaVersion::get), Ok(supported));
        assert_eq!(trace.map(TraceSchemaVersion::get), Ok(supported));
        assert_eq!(tolerance.map(ToleranceProfileVersion::get), Ok(supported));
    }

    #[test]
    fn version_types_reject_unsupported_values_without_coercion() {
        // Arrange
        let unsupported = [0, 2, u32::MAX];

        // Act
        let results = unsupported.map(|value| {
            (
                ProtocolVersion::new(value),
                ScenarioSchemaVersion::new(value),
                TraceSchemaVersion::new(value),
                ToleranceProfileVersion::new(value),
            )
        });

        // Assert
        for (index, result) in results.into_iter().enumerate() {
            let expected_value = unsupported[index];
            assert_eq!(
                result.0.expect_err("protocol version should fail").value(),
                expected_value
            );
            assert_eq!(
                result.1.expect_err("scenario version should fail").value(),
                expected_value
            );
            assert_eq!(
                result.2.expect_err("trace version should fail").value(),
                expected_value
            );
            assert_eq!(
                result.3.expect_err("tolerance version should fail").value(),
                expected_value
            );
        }
    }
}
