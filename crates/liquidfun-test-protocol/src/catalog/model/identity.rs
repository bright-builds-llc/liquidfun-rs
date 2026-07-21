use serde::{Deserialize, Deserializer, Serialize};

use super::{CATALOG_MAXIMUM_ITERATIONS, CatalogError, CatalogErrorKind};
use crate::{FloatBits, ScenarioId};

macro_rules! catalog_id_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(ScenarioId);

        impl $name {
            /// Parses a stable catalog identifier.
            ///
            /// # Errors
            ///
            /// Returns [`CatalogError`] when the value violates the protocol identifier contract.
            pub fn new(value: impl Into<String>) -> Result<Self, CatalogError> {
                ScenarioId::new(value)
                    .map(Self)
                    .map_err(|_| CatalogError::new(CatalogErrorKind::InvalidIdentifier))
            }

            /// Returns the validated identifier text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

catalog_id_type!(
    CatalogSlug,
    "Stable catalog lookup identity, independent of display text."
);
catalog_id_type!(
    GeneratorId,
    "Stable identity of a deterministic scenario generator."
);
catalog_id_type!(
    ScenarioActionId,
    "Stable identity of one ordered resolved action."
);

macro_rules! catalog_version_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(u32);

        impl $name {
            /// The only supported version.
            pub const CURRENT: Self = Self(1);

            /// Validates a raw version.
            ///
            /// # Errors
            ///
            /// Returns [`CatalogError`] unless `value` is exactly 1.
            pub const fn new(value: u32) -> Result<Self, CatalogError> {
                if value == 1 {
                    return Ok(Self(value));
                }
                Err(CatalogError::new(CatalogErrorKind::UnsupportedVersion))
            }

            /// Returns the validated numeric version.
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
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

catalog_version_type!(
    CatalogSchemaVersion,
    "Version of the resolved catalog byte schema."
);
catalog_version_type!(
    ScenarioVersion,
    "Version of one stable catalog scenario definition."
);
catalog_version_type!(
    GeneratorVersion,
    "Version of one deterministic generator algorithm."
);

/// Whether one catalog definition permits a caller-supplied seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioEligibility {
    /// The definition is named and rejects seeds.
    NamedOnly,
    /// The definition requires a seed.
    SeedRequired,
}

/// Exact settings shared by every execution backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RunSettings {
    timestep_bits: FloatBits,
    velocity_iterations: u32,
    position_iterations: u32,
    particle_iterations: u32,
}

impl RunSettings {
    /// Validates exact timestep bits and solver iteration counts.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] unless the timestep is finite and positive and every iteration
    /// count is in `1..=1024`.
    pub fn new(
        timestep_bits: FloatBits,
        velocity_iterations: u32,
        position_iterations: u32,
        particle_iterations: u32,
    ) -> Result<Self, CatalogError> {
        let timestep = timestep_bits.to_f32();
        let iterations = [
            velocity_iterations,
            position_iterations,
            particle_iterations,
        ];
        if !timestep.is_finite()
            || timestep <= 0.0
            || iterations
                .into_iter()
                .any(|value| !(1..=CATALOG_MAXIMUM_ITERATIONS).contains(&value))
        {
            return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
        }
        Ok(Self {
            timestep_bits,
            velocity_iterations,
            position_iterations,
            particle_iterations,
        })
    }

    /// Returns the exact timestep bits.
    #[must_use]
    pub const fn timestep_bits(self) -> FloatBits {
        self.timestep_bits
    }

    /// Returns the rigid velocity-iteration count.
    #[must_use]
    pub const fn velocity_iterations(self) -> u32 {
        self.velocity_iterations
    }

    /// Returns the rigid position-iteration count.
    #[must_use]
    pub const fn position_iterations(self) -> u32 {
        self.position_iterations
    }

    /// Returns the particle-iteration count.
    #[must_use]
    pub const fn particle_iterations(self) -> u32 {
        self.particle_iterations
    }
}

impl<'de> Deserialize<'de> for RunSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawRunSettings {
            timestep_bits: FloatBits,
            velocity_iterations: u32,
            position_iterations: u32,
            particle_iterations: u32,
        }

        let raw = RawRunSettings::deserialize(deserializer)?;
        Self::new(
            raw.timestep_bits,
            raw.velocity_iterations,
            raw.position_iterations,
            raw.particle_iterations,
        )
        .map_err(serde::de::Error::custom)
    }
}
