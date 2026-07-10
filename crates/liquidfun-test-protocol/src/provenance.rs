use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

/// Error returned when a hexadecimal SHA-256 identity is malformed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("SHA-256 identity must be exactly 64 lowercase hexadecimal characters")]
pub struct Sha256HexError;

/// Validated lowercase hexadecimal SHA-256 identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256Hex(Box<str>);

impl Sha256Hex {
    /// Validates a lowercase hexadecimal SHA-256 identity.
    ///
    /// # Errors
    ///
    /// Returns [`Sha256HexError`] unless the value is exactly 64 lowercase hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, Sha256HexError> {
        let value = value.into();
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(Sha256HexError);
        }

        Ok(Self(value.into_boxed_str()))
    }

    /// Encodes a raw SHA-256 digest as validated lowercase hexadecimal.
    #[must_use]
    pub fn from_digest(digest: [u8; 32]) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut encoded = String::with_capacity(64);
        for byte in digest {
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        Self(encoded.into_boxed_str())
    }

    /// Returns the validated lowercase hexadecimal representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for Sha256Hex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Sha256Hex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Raw build fields accepted at the provenance boundary before validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildIdentityFields {
    oracle_revision: String,
    adapter_revision: String,
    adapter_content_sha256: String,
    cmake_preset: String,
    compiler_id: String,
    compiler_version: String,
    target: String,
    build_type: String,
    effective_compile_flags: String,
    effective_link_flags: String,
    sanitizer_mode: String,
}

impl BuildIdentityFields {
    /// Collects raw build fields for validation as one identity.
    #[allow(
        clippy::too_many_arguments,
        reason = "the protocol identity has eleven fixed wire fields"
    )]
    pub fn new(
        oracle_revision: impl Into<String>,
        adapter_revision: impl Into<String>,
        adapter_content_sha256: impl Into<String>,
        cmake_preset: impl Into<String>,
        compiler_id: impl Into<String>,
        compiler_version: impl Into<String>,
        target: impl Into<String>,
        build_type: impl Into<String>,
        effective_compile_flags: impl Into<String>,
        effective_link_flags: impl Into<String>,
        sanitizer_mode: impl Into<String>,
    ) -> Self {
        Self {
            oracle_revision: oracle_revision.into(),
            adapter_revision: adapter_revision.into(),
            adapter_content_sha256: adapter_content_sha256.into(),
            cmake_preset: cmake_preset.into(),
            compiler_id: compiler_id.into(),
            compiler_version: compiler_version.into(),
            target: target.into(),
            build_type: build_type.into(),
            effective_compile_flags: effective_compile_flags.into(),
            effective_link_flags: effective_link_flags.into(),
            sanitizer_mode: sanitizer_mode.into(),
        }
    }

    /// Replaces the raw oracle revision for boundary-focused tests or adapters.
    #[must_use]
    pub fn with_oracle_revision(mut self, value: impl Into<String>) -> Self {
        self.oracle_revision = value.into();
        self
    }

    /// Replaces the raw compiler identifier for boundary-focused tests or adapters.
    #[must_use]
    pub fn with_compiler_id(mut self, value: impl Into<String>) -> Self {
        self.compiler_id = value.into();
        self
    }
}

/// Error returned when reported build provenance is incomplete or inconsistent.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BuildIdentityError {
    /// The oracle revision is not a full lowercase 40-hex Git object ID.
    #[error("oracle revision must be a full lowercase 40-hex Git object ID")]
    InvalidOracleRevision,
    /// The adapter content digest is malformed.
    #[error("adapter content SHA-256 is malformed")]
    InvalidAdapterContentSha256,
    /// A required identity field is empty.
    #[error("build identity field `{0}` must not be empty")]
    EmptyField(&'static str),
    /// The reported identity hash does not match the recomputed fields.
    #[error("reported build identity SHA-256 does not match its fields")]
    IdentityHashMismatch,
}

/// Validated, stable identity of one Rust or C++ harness build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildIdentity {
    oracle_revision: Box<str>,
    adapter_revision: Box<str>,
    adapter_content_sha256: Sha256Hex,
    cmake_preset: Box<str>,
    compiler_id: Box<str>,
    compiler_version: Box<str>,
    target: Box<str>,
    build_type: Box<str>,
    effective_compile_flags: Box<str>,
    effective_link_flags: Box<str>,
    sanitizer_mode: Box<str>,
    identity_sha256: Sha256Hex,
}

impl BuildIdentity {
    /// Validates build fields and computes their stable SHA-256 identity.
    ///
    /// # Errors
    ///
    /// Returns [`BuildIdentityError`] when a required field or digest is invalid.
    pub fn new(fields: BuildIdentityFields) -> Result<Self, BuildIdentityError> {
        validate_oracle_revision(&fields.oracle_revision)?;
        let adapter_content_sha256 = Sha256Hex::new(fields.adapter_content_sha256.clone())
            .map_err(|_| BuildIdentityError::InvalidAdapterContentSha256)?;
        validate_nonempty_fields(&fields)?;
        let identity_sha256 = hash_identity_fields(&fields);

        Ok(Self {
            oracle_revision: fields.oracle_revision.into_boxed_str(),
            adapter_revision: fields.adapter_revision.into_boxed_str(),
            adapter_content_sha256,
            cmake_preset: fields.cmake_preset.into_boxed_str(),
            compiler_id: fields.compiler_id.into_boxed_str(),
            compiler_version: fields.compiler_version.into_boxed_str(),
            target: fields.target.into_boxed_str(),
            build_type: fields.build_type.into_boxed_str(),
            effective_compile_flags: fields.effective_compile_flags.into_boxed_str(),
            effective_link_flags: fields.effective_link_flags.into_boxed_str(),
            sanitizer_mode: fields.sanitizer_mode.into_boxed_str(),
            identity_sha256,
        })
    }

    /// Validates fields and rejects a reported hash that does not recompute.
    ///
    /// # Errors
    ///
    /// Returns [`BuildIdentityError`] when fields are invalid or the reported hash differs.
    pub fn from_reported(
        fields: BuildIdentityFields,
        reported_identity_sha256: &Sha256Hex,
    ) -> Result<Self, BuildIdentityError> {
        let identity = Self::new(fields)?;
        if identity.identity_sha256 != *reported_identity_sha256 {
            return Err(BuildIdentityError::IdentityHashMismatch);
        }
        Ok(identity)
    }

    /// Returns the full pinned oracle revision.
    #[must_use]
    pub fn oracle_revision(&self) -> &str {
        &self.oracle_revision
    }

    /// Returns the adapter revision or digest label.
    #[must_use]
    pub fn adapter_revision(&self) -> &str {
        &self.adapter_revision
    }

    /// Returns the adapter content SHA-256.
    #[must_use]
    pub const fn adapter_content_sha256(&self) -> &Sha256Hex {
        &self.adapter_content_sha256
    }

    /// Returns the `CMake` preset identity.
    #[must_use]
    pub fn cmake_preset(&self) -> &str {
        &self.cmake_preset
    }

    /// Returns the compiler implementation identity.
    #[must_use]
    pub fn compiler_id(&self) -> &str {
        &self.compiler_id
    }

    /// Returns the complete compiler version.
    #[must_use]
    pub fn compiler_version(&self) -> &str {
        &self.compiler_version
    }

    /// Returns the target or platform triple.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Returns the build type.
    #[must_use]
    pub fn build_type(&self) -> &str {
        &self.build_type
    }

    /// Returns the effective compile flags.
    #[must_use]
    pub fn effective_compile_flags(&self) -> &str {
        &self.effective_compile_flags
    }

    /// Returns the effective link flags.
    #[must_use]
    pub fn effective_link_flags(&self) -> &str {
        &self.effective_link_flags
    }

    /// Returns the sanitizer mode.
    #[must_use]
    pub fn sanitizer_mode(&self) -> &str {
        &self.sanitizer_mode
    }

    /// Returns the recomputed stable identity hash.
    #[must_use]
    pub const fn identity_sha256(&self) -> &Sha256Hex {
        &self.identity_sha256
    }
}

fn validate_oracle_revision(value: &str) -> Result<(), BuildIdentityError> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    Err(BuildIdentityError::InvalidOracleRevision)
}

fn validate_nonempty_fields(fields: &BuildIdentityFields) -> Result<(), BuildIdentityError> {
    let required = [
        ("adapter_revision", fields.adapter_revision.as_str()),
        ("cmake_preset", fields.cmake_preset.as_str()),
        ("compiler_id", fields.compiler_id.as_str()),
        ("compiler_version", fields.compiler_version.as_str()),
        ("target", fields.target.as_str()),
        ("build_type", fields.build_type.as_str()),
        (
            "effective_compile_flags",
            fields.effective_compile_flags.as_str(),
        ),
        ("effective_link_flags", fields.effective_link_flags.as_str()),
        ("sanitizer_mode", fields.sanitizer_mode.as_str()),
    ];
    if let Some((name, _)) = required.iter().find(|(_, value)| value.trim().is_empty()) {
        return Err(BuildIdentityError::EmptyField(name));
    }
    Ok(())
}

fn hash_identity_fields(fields: &BuildIdentityFields) -> Sha256Hex {
    let values = [
        ("oracle_revision", fields.oracle_revision.as_str()),
        ("adapter_revision", fields.adapter_revision.as_str()),
        (
            "adapter_content_sha256",
            fields.adapter_content_sha256.as_str(),
        ),
        ("cmake_preset", fields.cmake_preset.as_str()),
        ("compiler_id", fields.compiler_id.as_str()),
        ("compiler_version", fields.compiler_version.as_str()),
        ("target", fields.target.as_str()),
        ("build_type", fields.build_type.as_str()),
        (
            "effective_compile_flags",
            fields.effective_compile_flags.as_str(),
        ),
        ("effective_link_flags", fields.effective_link_flags.as_str()),
        ("sanitizer_mode", fields.sanitizer_mode.as_str()),
    ];
    let mut hasher = Sha256::new();
    for (name, value) in values {
        hasher.update(name.len().to_be_bytes());
        hasher.update(name.as_bytes());
        hasher.update(value.len().to_be_bytes());
        hasher.update(value.as_bytes());
    }
    Sha256Hex::from_digest(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::{BuildIdentity, BuildIdentityError, BuildIdentityFields, Sha256Hex};

    fn valid_fields() -> BuildIdentityFields {
        BuildIdentityFields::new(
            "7f20402173fd143a3988c921bc384459c6a858f2",
            "adapter-v1",
            "c7f36eaf2f184a36b9c9a04636d3e22785d815c4948d55d0b3cbf44ee7245fc8",
            "oracle-debug",
            "Clang",
            "22.1.8",
            "aarch64-apple-darwin",
            "Debug",
            "-O0 -g",
            "-lc++",
            "none",
        )
    }

    #[test]
    fn build_identity_validates_and_hashes_all_fields_stably() {
        // Arrange
        let fields = valid_fields();
        let changed_fields = valid_fields().with_compiler_id("AppleClang");

        // Act
        let first = BuildIdentity::new(fields.clone()).expect("valid identity should build");
        let second = BuildIdentity::new(fields).expect("same identity should build");
        let changed = BuildIdentity::new(changed_fields).expect("changed identity should build");

        // Assert
        assert_eq!(first.identity_sha256(), second.identity_sha256());
        assert_ne!(first.identity_sha256(), changed.identity_sha256());
        assert_eq!(
            first.oracle_revision(),
            "7f20402173fd143a3988c921bc384459c6a858f2"
        );
    }

    #[test]
    fn build_identity_rejects_invalid_revision_and_empty_fields() {
        // Arrange
        let invalid_revision = valid_fields().with_oracle_revision("short");
        let empty_compiler = valid_fields().with_compiler_id("");

        // Act
        let revision_error =
            BuildIdentity::new(invalid_revision).expect_err("revision should fail");
        let compiler_error = BuildIdentity::new(empty_compiler).expect_err("field should fail");

        // Assert
        assert_eq!(revision_error, BuildIdentityError::InvalidOracleRevision);
        assert_eq!(
            compiler_error,
            BuildIdentityError::EmptyField("compiler_id")
        );
    }

    #[test]
    fn build_identity_rejects_a_mismatched_reported_hash() {
        // Arrange
        let reported = Sha256Hex::new("00".repeat(32)).expect("zero hash is syntactically valid");

        // Act
        let error = BuildIdentity::from_reported(valid_fields(), &reported)
            .expect_err("mismatched identity hash should fail");

        // Assert
        assert_eq!(error, BuildIdentityError::IdentityHashMismatch);
    }
}
