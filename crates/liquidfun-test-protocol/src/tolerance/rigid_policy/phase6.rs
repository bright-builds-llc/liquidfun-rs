use super::{
    CollectionPolicy, Deserialize, Digest, DivergenceHorizon, EvidenceTier, FLOAT_PATHS,
    FieldComparison, FieldPolicy, FloatPolicy, HashSet, MAXIMUM_JUSTIFICATION_BYTES,
    MAXIMUM_SEMANTIC_PATH_BYTES, NonFinitePolicy, STRUCTURAL_PATHS, Sha256, Sha256Hex,
    ToleranceProfileVersion, ZeroPolicy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase6PolicyProfile {
    profile_id: Box<str>,
    version: ToleranceProfileVersion,
    fields: Box<[FieldPolicy]>,
    profile_sha256: Sha256Hex,
}

impl Phase6PolicyProfile {
    /// Parses, validates, sorts, and hashes one closed Phase 6 profile.
    ///
    /// # Errors
    ///
    /// Returns [`Phase6PolicyError`] when identity, completeness, comparison,
    /// horizon, tier, collection, or edge-case metadata is not exact.
    pub fn parse_toml(input: &str) -> Result<Self, Phase6PolicyError> {
        let raw: RawProfile = toml::from_str(input).map_err(Phase6PolicyError::Toml)?;
        if raw.profile_id.as_ref() != "phase6-v1"
            || raw.version != ToleranceProfileVersion::CURRENT.get()
        {
            return Err(Phase6PolicyError::UnsupportedIdentity);
        }
        if raw.fields.len() != STRUCTURAL_PATHS.len() + FLOAT_PATHS.len() {
            return Err(Phase6PolicyError::IncompleteProfile);
        }

        let mut fields = raw.fields;
        let mut semantic_paths = HashSet::with_capacity(fields.len());
        for field in &fields {
            validate_field(field)?;
            if !semantic_paths.insert(field.semantic_path()) {
                return Err(Phase6PolicyError::DuplicateSemanticPath(
                    field.semantic_path().into(),
                ));
            }
        }
        if STRUCTURAL_PATHS
            .iter()
            .chain(FLOAT_PATHS)
            .any(|path| !semantic_paths.contains(path))
        {
            return Err(Phase6PolicyError::IncompleteProfile);
        }
        fields.sort_unstable_by(|left, right| left.semantic_path().cmp(right.semantic_path()));

        let canonical = serde_json::to_vec(&fields)
            .map_err(|error| Phase6PolicyError::Canonicalization(error.to_string()))?;
        let mut hasher = Sha256::new();
        update_hash_field(&mut hasher, raw.profile_id.as_bytes());
        update_hash_field(&mut hasher, &raw.version.to_be_bytes());
        update_hash_field(&mut hasher, &canonical);

        Ok(Self {
            profile_id: raw.profile_id,
            version: ToleranceProfileVersion::CURRENT,
            fields: fields.into_boxed_slice(),
            profile_sha256: Sha256Hex::from_digest(hasher.finalize().into()),
        })
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub const fn version(&self) -> ToleranceProfileVersion {
        self.version
    }

    #[must_use]
    pub fn fields(&self) -> &[FieldPolicy] {
        &self.fields
    }

    #[must_use]
    pub const fn profile_sha256(&self) -> &Sha256Hex {
        &self.profile_sha256
    }

    #[must_use]
    pub fn field(&self, semantic_path: &str) -> Option<&FieldPolicy> {
        self.fields
            .binary_search_by_key(&semantic_path, |field| field.semantic_path())
            .ok()
            .map(|index| &self.fields[index])
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Phase6PolicyError {
    #[error("phase6 policy TOML is invalid: {0}")]
    Toml(toml::de::Error),
    #[error("phase6 policy identity or version is unsupported")]
    UnsupportedIdentity,
    #[error("phase6 policy must classify every reviewed observable exactly once")]
    IncompleteProfile,
    #[error("semantic path is invalid or default-like")]
    InvalidSemanticPath,
    #[error("duplicate semantic path: {0}")]
    DuplicateSemanticPath(Box<str>),
    #[error("floating threshold must be finite and nonnegative")]
    InvalidThreshold,
    #[error("phase6 observables begin with exact comparison")]
    NonExactComparison,
    #[error("phase6 policy carries incompatible horizon, tier, collection, or edge metadata")]
    IncompatibleMetadata,
    #[error("justification must be nonempty and at most 512 bytes")]
    InvalidJustification,
    #[error("phase6 policy canonicalization failed: {0}")]
    Canonicalization(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawProfile {
    pub(super) profile_id: Box<str>,
    pub(super) version: u32,
    pub(super) fields: Vec<FieldPolicy>,
}

fn validate_field(field: &FieldPolicy) -> Result<(), Phase6PolicyError> {
    validate_path(field.semantic_path())?;
    if field.justification().is_empty() || field.justification().len() > MAXIMUM_JUSTIFICATION_BYTES
    {
        return Err(Phase6PolicyError::InvalidJustification);
    }
    if field.collection_policy() != CollectionPolicy::Ordered
        || field.horizon() != DivergenceHorizon::PhaseLocal
        || field.zero_policy() != ZeroPolicy::Distinct
        || field.non_finite_policy() != NonFinitePolicy::RejectArithmeticNaN
    {
        return Err(Phase6PolicyError::IncompatibleMetadata);
    }

    let path = field.semantic_path();
    if STRUCTURAL_PATHS.contains(&path) {
        if field.comparison() != FieldComparison::ExactDiscrete
            || field.evidence_tier() != EvidenceTier::D1Canonical
        {
            return Err(Phase6PolicyError::IncompatibleMetadata);
        }
        return Ok(());
    }
    if !FLOAT_PATHS.contains(&path) {
        return Err(Phase6PolicyError::InvalidSemanticPath);
    }

    let FieldComparison::Float { policy } = field.comparison() else {
        return Err(Phase6PolicyError::IncompatibleMetadata);
    };
    validate_float_thresholds(policy)?;
    if policy != FloatPolicy::ExactBits {
        return Err(Phase6PolicyError::NonExactComparison);
    }
    if field.evidence_tier() != EvidenceTier::D2Supported {
        return Err(Phase6PolicyError::IncompatibleMetadata);
    }
    Ok(())
}

pub(super) fn validate_path(path: &str) -> Result<(), Phase6PolicyError> {
    let default_like = path.split('.').any(|segment| {
        matches!(
            segment.to_ascii_lowercase().as_str(),
            "*" | "**" | "any" | "all" | "default" | "fallback"
        )
    });
    if path.is_empty()
        || path.len() > MAXIMUM_SEMANTIC_PATH_BYTES
        || path
            .chars()
            .any(|character| matches!(character, '*' | '?' | '[' | ']'))
        || default_like
    {
        return Err(Phase6PolicyError::InvalidSemanticPath);
    }
    Ok(())
}

pub(super) fn validate_float_thresholds(policy: FloatPolicy) -> Result<(), Phase6PolicyError> {
    let valid = |bits: crate::FloatBits| {
        let value = bits.to_f32();
        value.is_finite() && value >= 0.0
    };
    match policy {
        FloatPolicy::Absolute { max_bits } => valid(max_bits),
        FloatPolicy::AbsoluteRelative {
            absolute_bits,
            relative_bits,
        } => valid(absolute_bits) && valid(relative_bits),
        FloatPolicy::ExactBits | FloatPolicy::Ulps { .. } => true,
    }
    .then_some(())
    .ok_or(Phase6PolicyError::InvalidThreshold)
}

pub(super) fn update_hash_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

#[cfg(test)]
pub(super) fn render_phase6_policy_presentation(profile: &Phase6PolicyProfile) -> String {
    let mut rendered = format!(
        "profile_id = \"{}\"\nversion = {}\n\n",
        profile.profile_id(),
        profile.version().get()
    );
    for path in STRUCTURAL_PATHS.iter().chain(FLOAT_PATHS) {
        let field = profile
            .field(path)
            .expect("closed Phase 6 registry requires every rendered field");
        let comparison = match field.comparison() {
            FieldComparison::ExactDiscrete => "{ kind = \"exact_discrete\" }",
            FieldComparison::Float {
                policy: FloatPolicy::ExactBits,
            } => "{ kind = \"float\", policy = { kind = \"exact_bits\" } }",
            FieldComparison::Float { .. } => {
                unreachable!("validated Phase 6 policy begins with exact comparisons")
            }
        };
        let tier = match field.evidence_tier() {
            EvidenceTier::D1Canonical => "d1_canonical",
            EvidenceTier::D2Supported => "d2_supported",
            _ => unreachable!("validated Phase 6 policy carries only D1 or D2 authority"),
        };
        rendered.push_str(&format!(
            concat!(
                "[[fields]]\n",
                "semantic_path = \"{}\"\n",
                "comparison = {}\n",
                "zero_policy = \"distinct\"\n",
                "non_finite_policy = \"reject_arithmetic_nan\"\n",
                "collection_policy = \"ordered\"\n",
                "horizon = {{ kind = \"phase_local\" }}\n",
                "evidence_tier = \"{}\"\n",
                "justification = \"{}\"\n\n"
            ),
            path,
            comparison,
            tier,
            field.justification()
        ));
    }
    rendered.pop();
    rendered
}
