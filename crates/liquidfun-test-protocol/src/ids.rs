use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

const MAX_ID_BYTES: usize = 128;

/// Broad category of a typed identifier validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdErrorKind {
    /// No identifier bytes were supplied.
    Empty,
    /// The identifier exceeds the reviewed byte limit.
    TooLong,
    /// The identifier contains non-ASCII input.
    NonAscii,
    /// The first byte is not a lowercase ASCII letter or digit.
    InvalidLeading,
    /// A later byte is outside the allowed identifier alphabet.
    InvalidCharacter,
}

/// Error returned when a typed protocol identifier is invalid.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid protocol identifier: {kind:?}")]
pub struct IdError {
    kind: IdErrorKind,
    maybe_index: Option<usize>,
}

impl IdError {
    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(&self) -> IdErrorKind {
        self.kind
    }

    /// Returns the invalid byte index when one is meaningful.
    #[must_use]
    pub const fn maybe_index(&self) -> Option<usize> {
        self.maybe_index
    }
}

fn validate_id(value: &str) -> Result<(), IdError> {
    if value.is_empty() {
        return Err(IdError {
            kind: IdErrorKind::Empty,
            maybe_index: None,
        });
    }
    if value.len() > MAX_ID_BYTES {
        return Err(IdError {
            kind: IdErrorKind::TooLong,
            maybe_index: None,
        });
    }
    if !value.is_ascii() {
        return Err(IdError {
            kind: IdErrorKind::NonAscii,
            maybe_index: value.bytes().position(|byte| !byte.is_ascii()),
        });
    }

    let bytes = value.as_bytes();
    if !is_lowercase_alphanumeric(bytes[0]) {
        return Err(IdError {
            kind: IdErrorKind::InvalidLeading,
            maybe_index: Some(0),
        });
    }

    if let Some((index, _)) = bytes
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, byte)| !is_id_byte(**byte))
    {
        return Err(IdError {
            kind: IdErrorKind::InvalidCharacter,
            maybe_index: Some(index),
        });
    }

    Ok(())
}

const fn is_lowercase_alphanumeric(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit()
}

const fn is_id_byte(byte: u8) -> bool {
    is_lowercase_alphanumeric(byte) || matches!(byte, b'.' | b'_' | b'-')
}

macro_rules! id_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Box<str>);

        impl $name {
            /// Parses and validates an owned protocol identifier.
            ///
            /// # Errors
            ///
            /// Returns [`IdError`] when the value violates the length or ASCII alphabet contract.
            pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                validate_id(&value)?;
                Ok(Self(value.into_boxed_str()))
            }

            /// Returns the validated identifier text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
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

id_type!(RequestId, "Validated identity of one harness request.");
id_type!(
    ScenarioId,
    "Validated stable identity of a semantic scenario."
);
id_type!(
    CheckpointId,
    "Validated identity of a requested checkpoint."
);

/// Semantic kind carried by an engine-neutral entity identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticEntityKind {
    /// A rigid body.
    Body,
    /// A fixture attached to a rigid body.
    Fixture,
    /// A joint between rigid bodies.
    Joint,
    /// A particle system.
    ParticleSystem,
    /// A particle group.
    ParticleGroup,
    /// A stable particle identity.
    Particle,
}

/// Engine-neutral semantic identity formed from a kind and deterministic ordinal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SemanticEntityId {
    kind: SemanticEntityKind,
    ordinal: u32,
}

impl SemanticEntityId {
    /// Creates a semantic identity without exposing engine storage details.
    #[must_use]
    pub const fn new(kind: SemanticEntityKind, ordinal: u32) -> Self {
        Self { kind, ordinal }
    }

    /// Returns the semantic entity kind.
    #[must_use]
    pub const fn kind(self) -> SemanticEntityKind {
        self.kind
    }

    /// Returns the deterministic ordinal within the semantic kind.
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.ordinal
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CheckpointId, IdErrorKind, RequestId, ScenarioId, SemanticEntityId, SemanticEntityKind,
    };

    #[test]
    fn typed_ids_accept_bounded_lowercase_ascii() {
        // Arrange
        let values = ["request-1", "scenario.empty_world", "0_checkpoint"];

        // Act
        let request = RequestId::new(values[0]);
        let scenario = ScenarioId::new(values[1]);
        let checkpoint = CheckpointId::new(values[2]);

        // Assert
        assert_eq!(request.as_ref().map(RequestId::as_str), Ok(values[0]));
        assert_eq!(scenario.as_ref().map(ScenarioId::as_str), Ok(values[1]));
        assert_eq!(checkpoint.as_ref().map(CheckpointId::as_str), Ok(values[2]));
    }

    #[test]
    fn typed_ids_reject_invalid_values() {
        // Arrange
        let cases = [
            ("", IdErrorKind::Empty),
            ("Uppercase", IdErrorKind::InvalidLeading),
            ("-leading", IdErrorKind::InvalidLeading),
            ("has/slash", IdErrorKind::InvalidCharacter),
            ("has\ncontrol", IdErrorKind::InvalidCharacter),
            ("nonascii-é", IdErrorKind::NonAscii),
        ];

        // Act and Assert
        for (value, expected_kind) in cases {
            let error = RequestId::new(value).expect_err("invalid ID should fail");
            assert_eq!(error.kind(), expected_kind);
        }
    }

    #[test]
    fn typed_ids_reject_values_over_128_bytes() {
        // Arrange
        let value = "a".repeat(129);

        // Act
        let error = RequestId::new(value).expect_err("oversized ID should fail");

        // Assert
        assert_eq!(error.kind(), IdErrorKind::TooLong);
    }

    #[test]
    fn semantic_entity_ids_preserve_kind_and_ordinal() {
        // Arrange
        let kind = SemanticEntityKind::Body;
        let ordinal = 42;

        // Act
        let id = SemanticEntityId::new(kind, ordinal);

        // Assert
        assert_eq!(id.kind(), kind);
        assert_eq!(id.ordinal(), ordinal);
    }
}
