use std::{fmt, marker::PhantomData};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};

use crate::HarnessLimits;

/// Selects the reviewed byte limit for one JSONL record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordLimit {
    /// A request written to an adapter.
    Input,
    /// A record emitted by an adapter.
    Output,
}

/// Stable failure categories for strict JSONL framing and decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecErrorKind {
    /// The record contains no JSON value.
    BlankRecord,
    /// The record is not valid UTF-8.
    InvalidUtf8,
    /// EOF arrived before the required final newline.
    PartialRecord,
    /// More bytes follow the one allowed JSON value.
    TrailingBytes,
    /// The record exceeds its active reviewed byte limit.
    RecordTooLarge,
    /// JSON nesting exceeds the active reviewed depth limit.
    NestingTooDeep,
    /// An object repeats a member name.
    DuplicateMember,
    /// An object contains a member outside its closed schema.
    UnknownField,
    /// A tagged record or payload uses an unknown kind.
    UnknownRecordKind,
    /// A protocol version axis is unsupported.
    UnsupportedVersion,
    /// A bounded string or collection exceeded its typed limit.
    BoundaryLimitExceeded,
    /// The complete record is otherwise malformed.
    MalformedRecord,
    /// A valid typed value could not be serialized.
    SerializationFailure,
}

/// Error returned by strict JSONL framing, decoding, or encoding.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("JSONL codec failure: {kind:?}: {message}")]
pub struct CodecError {
    kind: CodecErrorKind,
    message: Box<str>,
}

impl CodecError {
    fn new(kind: CodecErrorKind, message: impl Into<Box<str>>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Returns the stable codec failure category.
    #[must_use]
    pub const fn kind(&self) -> CodecErrorKind {
        self.kind
    }
}

/// Decodes exactly one strict newline-complete JSON record.
///
/// # Errors
///
/// Returns [`CodecError`] for framing, resource-limit, UTF-8, shape, or typed decode failure.
pub fn decode_jsonl<T: DeserializeOwned>(
    bytes: &[u8],
    limits: &HarnessLimits,
    record_limit: RecordLimit,
) -> Result<T, CodecError> {
    let maximum_bytes = match record_limit {
        RecordLimit::Input => limits.input_record_bytes(),
        RecordLimit::Output => limits.output_record_bytes(),
    };
    if bytes.len() > maximum_bytes {
        return Err(CodecError::new(
            CodecErrorKind::RecordTooLarge,
            "record exceeds reviewed byte limit",
        ));
    }
    if bytes.last() != Some(&b'\n') {
        return Err(CodecError::new(
            CodecErrorKind::PartialRecord,
            "record does not end with newline",
        ));
    }

    let payload = &bytes[..bytes.len() - 1];
    if payload.contains(&b'\n') {
        return Err(CodecError::new(
            CodecErrorKind::TrailingBytes,
            "more than one line was supplied",
        ));
    }
    let payload = std::str::from_utf8(payload)
        .map_err(|error| CodecError::new(CodecErrorKind::InvalidUtf8, error.to_string()))?;
    if payload.trim().is_empty() {
        return Err(CodecError::new(
            CodecErrorKind::BlankRecord,
            "record contains no JSON value",
        ));
    }
    validate_nesting_depth(payload.as_bytes(), limits.json_nesting_depth())?;

    let mut stream = serde_json::Deserializer::from_str(payload).into_iter::<T>();
    let value = stream
        .next()
        .ok_or_else(|| CodecError::new(CodecErrorKind::BlankRecord, "missing JSON value"))?
        .map_err(|error| classify_decode_error(&error))?;
    if stream.byte_offset() != payload.len() {
        return Err(CodecError::new(
            CodecErrorKind::TrailingBytes,
            "bytes follow the decoded JSON value",
        ));
    }
    Ok(value)
}

/// Serializes exactly one newline-complete JSON record under the reviewed limit.
///
/// # Errors
///
/// Returns [`CodecError`] when serialization fails or the encoded record is oversized.
pub fn encode_jsonl<T: Serialize>(
    value: &T,
    limits: &HarnessLimits,
    record_limit: RecordLimit,
) -> Result<Vec<u8>, CodecError> {
    let mut encoded = serde_json::to_vec(value).map_err(|error| {
        CodecError::new(CodecErrorKind::SerializationFailure, error.to_string())
    })?;
    encoded.push(b'\n');
    let maximum_bytes = match record_limit {
        RecordLimit::Input => limits.input_record_bytes(),
        RecordLimit::Output => limits.output_record_bytes(),
    };
    if encoded.len() > maximum_bytes {
        return Err(CodecError::new(
            CodecErrorKind::RecordTooLarge,
            "encoded record exceeds reviewed byte limit",
        ));
    }
    Ok(encoded)
}

fn classify_decode_error(error: &serde_json::Error) -> CodecError {
    let message = error.to_string();
    let normalized = message.to_ascii_lowercase();
    let kind = if normalized.contains("duplicate field") {
        CodecErrorKind::DuplicateMember
    } else if normalized.contains("unknown field") {
        CodecErrorKind::UnknownField
    } else if normalized.contains("unknown variant") {
        CodecErrorKind::UnknownRecordKind
    } else if normalized.contains("unsupported") && normalized.contains("version") {
        CodecErrorKind::UnsupportedVersion
    } else if normalized.contains("reviewed limit") {
        CodecErrorKind::BoundaryLimitExceeded
    } else {
        CodecErrorKind::MalformedRecord
    };
    CodecError::new(kind, message)
}

fn validate_nesting_depth(bytes: &[u8], maximum_depth: usize) -> Result<(), CodecError> {
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;
    for byte in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        if *byte == b'"' {
            in_string = true;
            continue;
        }
        if matches!(*byte, b'{' | b'[') {
            depth = depth.saturating_add(1);
            if depth > maximum_depth {
                return Err(CodecError::new(
                    CodecErrorKind::NestingTooDeep,
                    "JSON nesting exceeds reviewed limit",
                ));
            }
        } else if matches!(*byte, b'}' | b']') {
            depth = depth.saturating_sub(1);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundedString<const MAXIMUM: usize>(Box<str>);

impl<const MAXIMUM: usize> BoundedString<MAXIMUM> {
    pub(crate) fn into_string(self) -> String {
        self.0.into()
    }
}

impl<const MAXIMUM: usize> Serialize for BoundedString<MAXIMUM> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de, const MAXIMUM: usize> Deserialize<'de> for BoundedString<MAXIMUM> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<const MAXIMUM: usize>;

        impl<const MAXIMUM: usize> serde::de::Visitor<'_> for Visitor<MAXIMUM> {
            type Value = BoundedString<MAXIMUM>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "a UTF-8 string no longer than {MAXIMUM} bytes")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(value.to_owned())
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value.len() > MAXIMUM {
                    return Err(E::custom("string exceeds reviewed limit"));
                }
                Ok(BoundedString(value.into_boxed_str()))
            }
        }

        deserializer.deserialize_string(Visitor::<MAXIMUM>)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundedVec<T, const MAXIMUM: usize>(Vec<T>);

impl<T, const MAXIMUM: usize> BoundedVec<T, MAXIMUM> {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T: Serialize, const MAXIMUM: usize> Serialize for BoundedVec<T, MAXIMUM> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>, const MAXIMUM: usize> Deserialize<'de> for BoundedVec<T, MAXIMUM> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<T, const MAXIMUM: usize>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>, const MAXIMUM: usize> serde::de::Visitor<'de>
            for Visitor<T, MAXIMUM>
        {
            type Value = BoundedVec<T, MAXIMUM>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "a sequence with at most {MAXIMUM} elements")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let capacity = sequence.size_hint().unwrap_or(0).min(MAXIMUM);
                let mut values = Vec::with_capacity(capacity);
                while let Some(value) = sequence.next_element()? {
                    if values.len() == MAXIMUM {
                        return Err(serde::de::Error::custom(
                            "collection exceeds reviewed limit",
                        ));
                    }
                    values.push(value);
                }
                Ok(BoundedVec(values))
            }
        }

        deserializer.deserialize_seq(Visitor::<T, MAXIMUM>(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::{CodecErrorKind, RecordLimit, decode_jsonl};
    use crate::HarnessLimits;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct StrictRecord {
        value: u32,
    }

    #[test]
    fn codec_accepts_one_newline_complete_json_object() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();

        // Act
        let record = decode_jsonl::<StrictRecord>(b"{\"value\":7}\n", &limits, RecordLimit::Input)
            .expect("complete strict record should decode");

        // Assert
        assert_eq!(record.value, 7);
    }

    #[test]
    fn codec_classifies_framing_failures_distinctly() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();
        let cases: [(&[u8], CodecErrorKind); 5] = [
            (b"\n", CodecErrorKind::BlankRecord),
            (&[0xff, b'\n'], CodecErrorKind::InvalidUtf8),
            (b"{\"value\":7}", CodecErrorKind::PartialRecord),
            (b"{\"value\":7}\n{}\n", CodecErrorKind::TrailingBytes),
            (b"{\"value\":7} trailing\n", CodecErrorKind::TrailingBytes),
        ];

        // Act and Assert
        for (bytes, expected) in cases {
            let error = decode_jsonl::<StrictRecord>(bytes, &limits, RecordLimit::Input)
                .expect_err("invalid framing should fail");
            assert_eq!(error.kind(), expected);
        }
    }

    #[test]
    fn codec_rejects_nesting_above_the_reviewed_limit() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();
        let accepted = format!("{}0{}\n", "[".repeat(32), "]".repeat(32));
        let rejected = format!("{}0{}\n", "[".repeat(33), "]".repeat(33));

        // Act
        let accepted_result =
            decode_jsonl::<serde::de::IgnoredAny>(accepted.as_bytes(), &limits, RecordLimit::Input);
        let rejected_error =
            decode_jsonl::<serde::de::IgnoredAny>(rejected.as_bytes(), &limits, RecordLimit::Input)
                .expect_err("depth N + 1 should fail");

        // Assert
        assert!(accepted_result.is_ok());
        assert_eq!(rejected_error.kind(), CodecErrorKind::NestingTooDeep);
    }

    #[test]
    fn codec_enforces_input_record_bytes_at_n_and_n_plus_one() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();
        let accepted = format!("\"{}\"\n", "x".repeat(limits.input_record_bytes() - 3));
        let rejected = format!("\"{}\"\n", "x".repeat(limits.input_record_bytes() - 2));

        // Act
        let accepted_result =
            decode_jsonl::<serde::de::IgnoredAny>(accepted.as_bytes(), &limits, RecordLimit::Input);
        let rejected_error =
            decode_jsonl::<serde::de::IgnoredAny>(rejected.as_bytes(), &limits, RecordLimit::Input)
                .expect_err("record N + 1 should fail");

        // Assert
        assert!(accepted_result.is_ok());
        assert_eq!(rejected_error.kind(), CodecErrorKind::RecordTooLarge);
    }
}
