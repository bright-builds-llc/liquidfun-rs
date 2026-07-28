use super::phase6::{RawProfile, update_hash_field, validate_float_thresholds, validate_path};
use super::{
    CollectionPolicy, Digest, DivergenceHorizon, EvidenceTier, FieldComparison, FieldPolicy,
    FloatPolicy, HashSet, MAXIMUM_JUSTIFICATION_BYTES, NonFinitePolicy,
    PHASE7_ABSOLUTE_RELATIVE_PATHS, PHASE7_STRUCTURAL_PATHS, PHASE7_ULP_PATHS,
    RigidWorldWitnessFamily, Sha256, Sha256Hex, ToleranceProfileVersion, ZeroPolicy,
};

/// Strict closed comparison profile for Phase 7 rigid evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase7PolicyProfile {
    profile_id: Box<str>,
    version: ToleranceProfileVersion,
    fields: Box<[FieldPolicy]>,
    profile_sha256: Sha256Hex,
}

impl Phase7PolicyProfile {
    /// Parses and validates the complete Phase 7 field registry.
    ///
    /// # Errors
    ///
    /// Returns [`Phase7PolicyError`] for any unknown, missing, duplicate, or
    /// incorrectly classified semantic path.
    pub fn parse_toml(input: &str) -> Result<Self, Phase7PolicyError> {
        let raw: RawProfile = toml::from_str(input).map_err(Phase7PolicyError::Toml)?;
        if raw.profile_id.as_ref() != "phase7-v1"
            || raw.version != ToleranceProfileVersion::CURRENT.get()
        {
            return Err(Phase7PolicyError::UnsupportedIdentity);
        }
        let expected_count = PHASE7_STRUCTURAL_PATHS.len()
            + PHASE7_ABSOLUTE_RELATIVE_PATHS.len()
            + PHASE7_ULP_PATHS.len();
        if raw.fields.len() != expected_count {
            return Err(Phase7PolicyError::IncompleteProfile);
        }

        let mut fields = raw.fields;
        let mut semantic_paths = HashSet::with_capacity(fields.len());
        for field in &fields {
            validate_phase7_field(field)?;
            if !semantic_paths.insert(field.semantic_path()) {
                return Err(Phase7PolicyError::DuplicateSemanticPath(
                    field.semantic_path().into(),
                ));
            }
        }
        if PHASE7_STRUCTURAL_PATHS
            .iter()
            .chain(PHASE7_ABSOLUTE_RELATIVE_PATHS)
            .chain(PHASE7_ULP_PATHS)
            .any(|path| !semantic_paths.contains(path))
        {
            return Err(Phase7PolicyError::IncompleteProfile);
        }
        if RigidWorldWitnessFamily::PHASE7_REQUIRED
            .into_iter()
            .flat_map(phase7_witness_policy_paths)
            .any(|path| !semantic_paths.contains(path))
        {
            return Err(Phase7PolicyError::UnregisteredWitnessPolicy);
        }
        fields.sort_unstable_by(|left, right| left.semantic_path().cmp(right.semantic_path()));
        let canonical = serde_json::to_vec(&fields)
            .map_err(|error| Phase7PolicyError::Canonicalization(error.to_string()))?;
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

    /// Looks up an explicit policy. Unknown paths have no fallback.
    #[must_use]
    pub fn field(&self, semantic_path: &str) -> Option<&FieldPolicy> {
        self.fields
            .binary_search_by_key(&semantic_path, |field| field.semantic_path())
            .ok()
            .map(|index| &self.fields[index])
    }
}

fn phase7_witness_policy_paths(family: RigidWorldWitnessFamily) -> &'static [&'static str] {
    match family {
        RigidWorldWitnessFamily::BodyControlAndForcePolicy => &[
            "rigid_world.phase7.body.id",
            "rigid_world.phase7.body.awake",
            "rigid_world.phase7.body.linear_velocity.x",
        ],
        RigidWorldWitnessFamily::MultiContactIslandAndWarmStart => &[
            "rigid_world.phase7.island.body_order",
            "rigid_world.phase7.island.contact_order",
            "rigid_world.phase7.contact.normal_impulse",
        ],
        RigidWorldWitnessFamily::SleepingAndWaking => &[
            "rigid_world.phase7.body.awake",
            "rigid_world.phase7.step.completion",
        ],
        RigidWorldWitnessFamily::ContinuousCollisionAndSubStepping => &[
            "rigid_world.phase7.body.bullet",
            "rigid_world.phase7.body.transform.position.x",
            "rigid_world.phase7.step.completion",
        ],
        RigidWorldWitnessFamily::ContinuousBudgetResume => &[
            "rigid_world.phase7.step.outcome.kind",
            "rigid_world.phase7.step.partial_classification",
            "rigid_world.phase7.step.completion",
        ],
        RigidWorldWitnessFamily::WorldQueryAndRayCast => &[
            "rigid_world.phase7.query.occurrences.identity",
            "rigid_world.phase7.ray.hit.identity",
            "rigid_world.phase7.ray.completion",
            "rigid_world.phase7.ray.final_max_fraction",
        ],
        RigidWorldWitnessFamily::OriginShiftCovariance => &[
            "rigid_world.phase7.origin_shift.x",
            "rigid_world.phase7.origin_shift.y",
            "rigid_world.phase7.query.occurrences.identity",
            "rigid_world.phase7.ray.final_max_fraction",
            "rigid_world.phase7.ray.fraction",
        ],
        RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle
        | RigidWorldWitnessFamily::SingleContactLifecycle
        | RigidWorldWitnessFamily::JointDefinitionsAndMutations
        | RigidWorldWitnessFamily::RevolutePrismaticLimitsAndMotors
        | RigidWorldWitnessFamily::DistancePulleyMouseConstraints
        | RigidWorldWitnessFamily::WheelWeldFrictionRopeMotorConstraints
        | RigidWorldWitnessFamily::GearDependenciesAndFourBodySolver
        | RigidWorldWitnessFamily::MixedJointIslandOrderAndCollisionSuppression
        | RigidWorldWitnessFamily::StandaloneRopeEvolution
        | RigidWorldWitnessFamily::ContactFilterListenerAndPreSolveTiming
        | RigidWorldWitnessFamily::DestructionListenerAndDependencyCascades
        | RigidWorldWitnessFamily::DiagnosticReconstructionAndDumpOrder => &[],
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Phase7PolicyError {
    #[error("phase7 policy TOML is invalid: {0}")]
    Toml(toml::de::Error),
    #[error("phase7 policy identity or version is unsupported")]
    UnsupportedIdentity,
    #[error("phase7 policy must classify every observable exactly once")]
    IncompleteProfile,
    #[error("semantic path is invalid or unregistered")]
    InvalidSemanticPath,
    #[error("duplicate semantic path: {0}")]
    DuplicateSemanticPath(Box<str>),
    #[error("phase7 field policy has incompatible comparison metadata")]
    IncompatibleMetadata,
    #[error("phase7 witness family has no complete policy registration")]
    UnregisteredWitnessPolicy,
    #[error("phase7 policy canonicalization failed: {0}")]
    Canonicalization(String),
}

fn validate_phase7_field(field: &FieldPolicy) -> Result<(), Phase7PolicyError> {
    validate_path(field.semantic_path()).map_err(|_| Phase7PolicyError::InvalidSemanticPath)?;
    if field.justification().is_empty() || field.justification().len() > MAXIMUM_JUSTIFICATION_BYTES
    {
        return Err(Phase7PolicyError::IncompatibleMetadata);
    }
    if field.horizon() != DivergenceHorizon::PhaseLocal
        || field.zero_policy() != ZeroPolicy::Distinct
        || field.non_finite_policy() != NonFinitePolicy::RejectArithmeticNaN
    {
        return Err(Phase7PolicyError::IncompatibleMetadata);
    }
    let path = field.semantic_path();
    if PHASE7_STRUCTURAL_PATHS.contains(&path) {
        let expected_collection = if matches!(
            path,
            "rigid_world.phase7.query.occurrences.identity" | "rigid_world.phase7.ray.hit.identity"
        ) {
            CollectionPolicy::Multiset
        } else {
            CollectionPolicy::Ordered
        };
        if field.comparison() != FieldComparison::ExactDiscrete
            || field.evidence_tier() != EvidenceTier::D1Canonical
            || field.collection_policy() != expected_collection
        {
            return Err(Phase7PolicyError::IncompatibleMetadata);
        }
        return Ok(());
    }
    if field.collection_policy() != CollectionPolicy::Ordered
        || field.evidence_tier() != EvidenceTier::D2Supported
    {
        return Err(Phase7PolicyError::IncompatibleMetadata);
    }
    let FieldComparison::Float { policy } = field.comparison() else {
        return Err(Phase7PolicyError::IncompatibleMetadata);
    };
    validate_float_thresholds(policy).map_err(|_| Phase7PolicyError::IncompatibleMetadata)?;
    let expected = if PHASE7_ABSOLUTE_RELATIVE_PATHS.contains(&path) {
        matches!(policy, FloatPolicy::AbsoluteRelative { .. })
    } else if PHASE7_ULP_PATHS.contains(&path) {
        matches!(policy, FloatPolicy::Ulps { .. })
    } else {
        return Err(Phase7PolicyError::InvalidSemanticPath);
    };
    expected
        .then_some(())
        .ok_or(Phase7PolicyError::IncompatibleMetadata)
}
