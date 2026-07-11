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
    maybe_phase4: Option<Phase4BuildIdentityFields>,
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
            maybe_phase4: None,
        }
    }

    /// Adds the complete Phase 4 compiler, platform, and runtime floating identity.
    #[must_use]
    pub fn with_phase4(mut self, phase4: Phase4BuildIdentityFields) -> Self {
        self.maybe_phase4 = Some(phase4);
        self
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

/// Raw Phase 4 build fields accepted together or rejected together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase4BuildIdentityFields {
    compile_command_sha256: String,
    compiler_id: String,
    compiler_version: String,
    target_triple: String,
    target_cpu: String,
    target_features: String,
    sdk_or_sysroot: String,
    optimization: String,
    fp_model: String,
    fp_contract: String,
    denormal_mode: String,
    feature_set: String,
    os: String,
    libc: String,
    libm: String,
    rounding_mode: String,
    gradual_underflow: bool,
}

impl Phase4BuildIdentityFields {
    /// Collects the exact 17-field Phase 4 identity extension.
    #[allow(
        clippy::too_many_arguments,
        reason = "the Phase 4 protocol has seventeen fixed wire fields"
    )]
    pub fn new(
        compile_command_sha256: impl Into<String>,
        compiler_id: impl Into<String>,
        compiler_version: impl Into<String>,
        target_triple: impl Into<String>,
        target_cpu: impl Into<String>,
        target_features: impl Into<String>,
        sdk_or_sysroot: impl Into<String>,
        optimization: impl Into<String>,
        fp_model: impl Into<String>,
        fp_contract: impl Into<String>,
        denormal_mode: impl Into<String>,
        feature_set: impl Into<String>,
        os: impl Into<String>,
        libc: impl Into<String>,
        libm: impl Into<String>,
        rounding_mode: impl Into<String>,
        gradual_underflow: bool,
    ) -> Self {
        Self {
            compile_command_sha256: compile_command_sha256.into(),
            compiler_id: compiler_id.into(),
            compiler_version: compiler_version.into(),
            target_triple: target_triple.into(),
            target_cpu: target_cpu.into(),
            target_features: target_features.into(),
            sdk_or_sysroot: sdk_or_sysroot.into(),
            optimization: optimization.into(),
            fp_model: fp_model.into(),
            fp_contract: fp_contract.into(),
            denormal_mode: denormal_mode.into(),
            feature_set: feature_set.into(),
            os: os.into(),
            libc: libc.into(),
            libm: libm.into(),
            rounding_mode: rounding_mode.into(),
            gradual_underflow,
        }
    }

    /// Replaces the feature description for forbidden-flag regression tests.
    #[must_use]
    pub fn with_feature_set(mut self, value: impl Into<String>) -> Self {
        self.feature_set = value.into();
        self
    }

    /// Replaces the SDK/sysroot witness for completeness regression tests.
    #[must_use]
    pub fn with_sdk_or_sysroot(mut self, value: impl Into<String>) -> Self {
        self.sdk_or_sysroot = value.into();
        self
    }
}

/// Evidence authority derived from exact compiler, target, and floating witnesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildEvidenceTier {
    /// Pinned Linux x86_64 compiler and complete IEEE runtime witnesses.
    D1Canonical,
    /// Supported baseline platform with complete identity, but no promotion authority.
    D2Supported,
    /// Exploratory or explicitly noncanonical build.
    D3Exploratory,
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
    /// The complete Phase 4 identity extension is present but malformed.
    #[error("Phase 4 build identity field `{0}` is invalid or empty")]
    InvalidPhase4Field(&'static str),
    /// A would-be canonical identity contains a prohibited optimization or CPU flag.
    #[error("canonical build identity contains forbidden floating or native tuning flags")]
    CanonicalForbiddenFlags,
    /// A would-be canonical identity lacks the required IEEE runtime witnesses.
    #[error("canonical build identity lacks round-to-nearest or gradual-underflow proof")]
    CanonicalRuntimeWitness,
}

/// Validated Phase 4 build and floating-point identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase4BuildIdentity {
    fields: Phase4BuildIdentityFields,
}

impl Phase4BuildIdentity {
    /// Returns the effective compile-command SHA-256.
    #[must_use]
    pub fn compile_command_sha256(&self) -> &str {
        &self.fields.compile_command_sha256
    }

    /// Returns the exact compiler identifier.
    #[must_use]
    pub fn compiler_id(&self) -> &str {
        &self.fields.compiler_id
    }

    /// Returns the exact target triple.
    #[must_use]
    pub fn target_triple(&self) -> &str {
        &self.fields.target_triple
    }

    /// Returns the recorded feature set and effective flag summary.
    #[must_use]
    pub fn feature_set(&self) -> &str {
        &self.fields.feature_set
    }

    /// Returns whether gradual underflow was observed at runtime.
    #[must_use]
    pub const fn gradual_underflow(&self) -> bool {
        self.fields.gradual_underflow
    }
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
    maybe_phase4: Option<Phase4BuildIdentity>,
    evidence_tier: BuildEvidenceTier,
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
        let maybe_phase4 = fields
            .maybe_phase4
            .as_ref()
            .map(validate_phase4_identity)
            .transpose()?;
        let evidence_tier = classify_evidence_tier(maybe_phase4.as_ref())?;
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
            maybe_phase4,
            evidence_tier,
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

    /// Returns the validated Phase 4 extension when this is not a legacy identity.
    #[must_use]
    pub const fn maybe_phase4(&self) -> Option<&Phase4BuildIdentity> {
        self.maybe_phase4.as_ref()
    }

    /// Returns the derived evidence authority.
    #[must_use]
    pub const fn evidence_tier(&self) -> BuildEvidenceTier {
        self.evidence_tier
    }

    /// Returns whether this exact build may promote canonical fixtures.
    #[must_use]
    pub const fn can_promote_canonical_evidence(&self) -> bool {
        matches!(self.evidence_tier, BuildEvidenceTier::D1Canonical)
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

fn validate_phase4_identity(
    fields: &Phase4BuildIdentityFields,
) -> Result<Phase4BuildIdentity, BuildIdentityError> {
    Sha256Hex::new(fields.compile_command_sha256.clone())
        .map_err(|_| BuildIdentityError::InvalidPhase4Field("compile_command_sha256"))?;
    let required = [
        ("compiler_id", fields.compiler_id.as_str()),
        ("compiler_version", fields.compiler_version.as_str()),
        ("target_triple", fields.target_triple.as_str()),
        ("target_cpu", fields.target_cpu.as_str()),
        ("target_features", fields.target_features.as_str()),
        ("sdk_or_sysroot", fields.sdk_or_sysroot.as_str()),
        ("optimization", fields.optimization.as_str()),
        ("fp_model", fields.fp_model.as_str()),
        ("fp_contract", fields.fp_contract.as_str()),
        ("denormal_mode", fields.denormal_mode.as_str()),
        ("feature_set", fields.feature_set.as_str()),
        ("os", fields.os.as_str()),
        ("libc", fields.libc.as_str()),
        ("libm", fields.libm.as_str()),
        ("rounding_mode", fields.rounding_mode.as_str()),
    ];
    if let Some((name, _)) = required.iter().find(|(_, value)| value.trim().is_empty()) {
        return Err(BuildIdentityError::InvalidPhase4Field(name));
    }
    Ok(Phase4BuildIdentity {
        fields: fields.clone(),
    })
}

fn classify_evidence_tier(
    maybe_phase4: Option<&Phase4BuildIdentity>,
) -> Result<BuildEvidenceTier, BuildIdentityError> {
    let Some(phase4) = maybe_phase4 else {
        return Ok(BuildEvidenceTier::D3Exploratory);
    };
    let fields = &phase4.fields;
    let combined = format!(
        "{} {} {} {} {}",
        fields.optimization,
        fields.fp_model,
        fields.target_cpu,
        fields.target_features,
        fields.feature_set
    );
    let forbidden = [
        "-ffast-math",
        "-Ofast",
        "-fassociative-math",
        "-freciprocal-math",
        "-funsafe-math-optimizations",
        "-march=native",
        "-mcpu=native",
    ]
    .iter()
    .any(|flag| combined.split_ascii_whitespace().any(|word| word == *flag));
    let canonical_compiler = (fields.compiler_id == "Clang" && fields.compiler_version == "22.1.8")
        || (fields.compiler_id == "rustc" && fields.compiler_version.contains("1.97.0"));
    let canonical_target = fields.target_triple == "x86_64-unknown-linux-gnu"
        && fields.target_cpu == "baseline"
        && fields.os.eq_ignore_ascii_case("linux");
    let canonical_candidate = canonical_compiler && canonical_target;
    if canonical_candidate && forbidden {
        return Err(BuildIdentityError::CanonicalForbiddenFlags);
    }
    if canonical_candidate
        && (fields.fp_model != "precise"
            || fields.fp_contract != "off"
            || fields.denormal_mode != "ieee"
            || fields.rounding_mode != "nearest_ties_even"
            || !fields.gradual_underflow)
    {
        return Err(BuildIdentityError::CanonicalRuntimeWitness);
    }
    if canonical_candidate {
        return Ok(BuildEvidenceTier::D1Canonical);
    }
    if forbidden {
        return Ok(BuildEvidenceTier::D3Exploratory);
    }
    let supported_os = ["linux", "macos", "windows"]
        .iter()
        .any(|os| fields.os.eq_ignore_ascii_case(os));
    Ok(if supported_os {
        BuildEvidenceTier::D2Supported
    } else {
        BuildEvidenceTier::D3Exploratory
    })
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
    if let Some(phase4) = &fields.maybe_phase4 {
        let phase4_values = [
            (
                "compile_command_sha256",
                phase4.compile_command_sha256.as_str(),
            ),
            ("compiler_id", phase4.compiler_id.as_str()),
            ("compiler_version", phase4.compiler_version.as_str()),
            ("target_triple", phase4.target_triple.as_str()),
            ("target_cpu", phase4.target_cpu.as_str()),
            ("target_features", phase4.target_features.as_str()),
            ("sdk_or_sysroot", phase4.sdk_or_sysroot.as_str()),
            ("optimization", phase4.optimization.as_str()),
            ("fp_model", phase4.fp_model.as_str()),
            ("fp_contract", phase4.fp_contract.as_str()),
            ("denormal_mode", phase4.denormal_mode.as_str()),
            ("feature_set", phase4.feature_set.as_str()),
            ("os", phase4.os.as_str()),
            ("libc", phase4.libc.as_str()),
            ("libm", phase4.libm.as_str()),
            ("rounding_mode", phase4.rounding_mode.as_str()),
            (
                "gradual_underflow",
                if phase4.gradual_underflow {
                    "true"
                } else {
                    "false"
                },
            ),
        ];
        for (name, value) in phase4_values {
            hasher.update(name.len().to_be_bytes());
            hasher.update(name.as_bytes());
            hasher.update(value.len().to_be_bytes());
            hasher.update(value.as_bytes());
        }
    }
    Sha256Hex::from_digest(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::{
        BuildEvidenceTier, BuildIdentity, BuildIdentityError, BuildIdentityFields,
        Phase4BuildIdentityFields, Sha256Hex,
    };

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

    fn canonical_phase4() -> Phase4BuildIdentityFields {
        Phase4BuildIdentityFields::new(
            "11".repeat(32),
            "Clang",
            "22.1.8",
            "x86_64-unknown-linux-gnu",
            "baseline",
            "<none>",
            "<none>",
            "O3",
            "precise",
            "off",
            "ieee",
            "scalar baseline",
            "linux",
            "glibc",
            "libm",
            "nearest_ties_even",
            true,
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

    #[test]
    fn canonical_identity_rejects_forbidden_fp_flags() {
        // Arrange
        let fields = valid_fields()
            .with_phase4(canonical_phase4().with_feature_set("scalar baseline -ffast-math"));

        // Act
        let error = BuildIdentity::new(fields).expect_err("canonical fast math should fail");

        // Assert
        assert_eq!(error, BuildIdentityError::CanonicalForbiddenFlags);
    }

    #[test]
    fn canonical_identity_requires_all_fields() {
        // Arrange
        let fields = valid_fields().with_phase4(canonical_phase4().with_sdk_or_sysroot(""));

        // Act
        let error = BuildIdentity::new(fields).expect_err("missing canonical field should fail");

        // Assert
        assert_eq!(
            error,
            BuildIdentityError::InvalidPhase4Field("sdk_or_sysroot")
        );
    }

    #[test]
    fn supported_identity_cannot_promote_canonical_evidence() {
        // Arrange
        let supported = Phase4BuildIdentityFields::new(
            "22".repeat(32),
            "AppleClang",
            "21.0.0",
            "arm64-apple-darwin",
            "baseline",
            "<none>",
            "macos-sdk",
            "O0",
            "precise",
            "off",
            "ieee",
            "scalar baseline",
            "macos",
            "libSystem",
            "libSystem",
            "nearest_ties_even",
            true,
        );

        // Act
        let identity = BuildIdentity::new(valid_fields().with_phase4(supported))
            .expect("supported identity should validate");

        // Assert
        assert_eq!(identity.evidence_tier(), BuildEvidenceTier::D2Supported);
        assert!(!identity.can_promote_canonical_evidence());
    }
}
