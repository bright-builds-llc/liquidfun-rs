use super::phase6::{RawProfile, update_hash_field, validate_float_thresholds, validate_path};
use super::{
    CollectionPolicy, Digest, DivergenceHorizon, EvidenceTier, FieldComparison, FieldPolicy,
    FloatPolicy, HashSet, MAXIMUM_JUSTIFICATION_BYTES, NonFinitePolicy, PHASE8_ABSOLUTE_PATHS,
    PHASE8_ABSOLUTE_RELATIVE_PATHS, PHASE8_EXACT_BITS_PATHS, PHASE8_STRUCTURAL_PATHS,
    PHASE8_ULP_PATHS, RigidWorldWitnessFamily, Sha256, Sha256Hex, ToleranceProfileVersion,
    ZeroPolicy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase8PolicyProfile {
    profile_id: Box<str>,
    version: ToleranceProfileVersion,
    fields: Box<[FieldPolicy]>,
    profile_sha256: Sha256Hex,
}

impl Phase8PolicyProfile {
    /// Parses and validates the complete Phase 8 field registry.
    ///
    /// # Errors
    ///
    /// Returns [`Phase8PolicyError`] for unknown, missing, duplicate, wildcard,
    /// or incorrectly classified semantic paths.
    pub fn parse_toml(input: &str) -> Result<Self, Phase8PolicyError> {
        let raw: RawProfile = toml::from_str(input).map_err(Phase8PolicyError::Toml)?;
        if raw.profile_id.as_ref() != "phase8-v1"
            || raw.version != ToleranceProfileVersion::CURRENT.get()
        {
            return Err(Phase8PolicyError::UnsupportedIdentity);
        }
        let expected_count = PHASE8_STRUCTURAL_PATHS.len()
            + PHASE8_EXACT_BITS_PATHS.len()
            + PHASE8_ABSOLUTE_RELATIVE_PATHS.len()
            + PHASE8_ULP_PATHS.len()
            + PHASE8_ABSOLUTE_PATHS.len();
        if raw.fields.len() != expected_count {
            return Err(Phase8PolicyError::IncompleteProfile);
        }

        let mut fields = raw.fields;
        let mut semantic_paths = HashSet::with_capacity(fields.len());
        for field in &fields {
            validate_phase8_field(field)?;
            if !semantic_paths.insert(field.semantic_path()) {
                return Err(Phase8PolicyError::DuplicateSemanticPath(
                    field.semantic_path().into(),
                ));
            }
        }
        if PHASE8_STRUCTURAL_PATHS
            .iter()
            .chain(PHASE8_EXACT_BITS_PATHS)
            .chain(PHASE8_ABSOLUTE_RELATIVE_PATHS)
            .chain(PHASE8_ULP_PATHS)
            .chain(PHASE8_ABSOLUTE_PATHS)
            .any(|path| !semantic_paths.contains(path))
        {
            return Err(Phase8PolicyError::IncompleteProfile);
        }
        if RigidWorldWitnessFamily::PHASE8_REQUIRED
            .into_iter()
            .flat_map(phase8_witness_policy_paths)
            .any(|path| !semantic_paths.contains(path))
        {
            return Err(Phase8PolicyError::UnregisteredWitnessPolicy);
        }
        fields.sort_unstable_by(|left, right| left.semantic_path().cmp(right.semantic_path()));
        let canonical = serde_json::to_vec(&fields)
            .map_err(|error| Phase8PolicyError::Canonicalization(error.to_string()))?;
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

#[derive(Debug, thiserror::Error)]
pub enum Phase8PolicyError {
    #[error("phase8 policy TOML is invalid: {0}")]
    Toml(toml::de::Error),
    #[error("phase8 policy identity or version is unsupported")]
    UnsupportedIdentity,
    #[error("phase8 policy must classify every observable exactly once")]
    IncompleteProfile,
    #[error("semantic path is invalid or unregistered")]
    InvalidSemanticPath,
    #[error("duplicate semantic path: {0}")]
    DuplicateSemanticPath(Box<str>),
    #[error("phase8 field policy has incompatible comparison metadata")]
    IncompatibleMetadata,
    #[error("phase8 witness family has no complete policy registration")]
    UnregisteredWitnessPolicy,
    #[error("phase8 policy canonicalization failed: {0}")]
    Canonicalization(String),
}

fn validate_phase8_field(field: &FieldPolicy) -> Result<(), Phase8PolicyError> {
    validate_path(field.semantic_path()).map_err(|_| Phase8PolicyError::InvalidSemanticPath)?;
    if field.justification().is_empty() || field.justification().len() > MAXIMUM_JUSTIFICATION_BYTES
    {
        return Err(Phase8PolicyError::IncompatibleMetadata);
    }
    if field.horizon() != DivergenceHorizon::PhaseLocal
        || field.zero_policy() != ZeroPolicy::Distinct
        || field.non_finite_policy() != NonFinitePolicy::RejectArithmeticNaN
        || field.collection_policy() != CollectionPolicy::Ordered
    {
        return Err(Phase8PolicyError::IncompatibleMetadata);
    }
    let path = field.semantic_path();
    if PHASE8_STRUCTURAL_PATHS.contains(&path) {
        if field.comparison() != FieldComparison::ExactDiscrete
            || field.evidence_tier() != EvidenceTier::D1Canonical
        {
            return Err(Phase8PolicyError::IncompatibleMetadata);
        }
        return Ok(());
    }
    let FieldComparison::Float { policy } = field.comparison() else {
        return Err(Phase8PolicyError::IncompatibleMetadata);
    };
    validate_float_thresholds(policy).map_err(|_| Phase8PolicyError::IncompatibleMetadata)?;
    let exact = PHASE8_EXACT_BITS_PATHS.contains(&path) && policy == FloatPolicy::ExactBits;
    let absolute_relative = PHASE8_ABSOLUTE_RELATIVE_PATHS.contains(&path)
        && phase8_absolute_relative_policy_matches(path, policy);
    let ulps = PHASE8_ULP_PATHS.contains(&path) && policy == FloatPolicy::Ulps { max: 4 };
    let absolute = PHASE8_ABSOLUTE_PATHS.contains(&path)
        && policy
            == FloatPolicy::Absolute {
                max_bits: crate::FloatBits::new(897_988_541),
            };
    if !(exact || absolute_relative || ulps || absolute)
        || field.evidence_tier()
            != if exact {
                EvidenceTier::D1Canonical
            } else {
                EvidenceTier::D2Supported
            }
    {
        return Err(Phase8PolicyError::IncompatibleMetadata);
    }
    Ok(())
}

fn phase8_absolute_relative_policy_matches(path: &str, policy: FloatPolicy) -> bool {
    let (absolute_bits, relative_bits) = match path {
        "rigid_world.phase8.joint.coordinate" | "rigid_world.phase8.joint.speed" => {
            (897_988_541, 981_668_463)
        }
        "rigid_world.phase8.joint.reaction_force.x"
        | "rigid_world.phase8.joint.reaction_force.y" => (973_279_855, 953_267_991),
        "rigid_world.phase8.joint.reaction_torque" => (897_988_541, 1_017_370_378),
        _ => (897_988_541, 953_267_991),
    };
    policy
        == FloatPolicy::AbsoluteRelative {
            absolute_bits: crate::FloatBits::new(absolute_bits),
            relative_bits: crate::FloatBits::new(relative_bits),
        }
}

fn phase8_witness_policy_paths(family: RigidWorldWitnessFamily) -> &'static [&'static str] {
    match family {
        RigidWorldWitnessFamily::JointDefinitionsAndMutations
        | RigidWorldWitnessFamily::RevolutePrismaticLimitsAndMotors
        | RigidWorldWitnessFamily::DistancePulleyMouseConstraints
        | RigidWorldWitnessFamily::WheelWeldFrictionRopeMotorConstraints
        | RigidWorldWitnessFamily::MixedJointIslandOrderAndCollisionSuppression => &[
            "rigid_world.phase8.joint.id",
            "rigid_world.phase8.joint.kind",
            "rigid_world.phase8.joint.body_ids",
            "rigid_world.phase8.joint.collide_connected",
            "rigid_world.phase8.joint.configuration.bits",
            "rigid_world.phase8.joint.anchor.x",
            "rigid_world.phase8.joint.anchor.y",
            "rigid_world.phase8.joint.coordinate",
            "rigid_world.phase8.joint.speed",
            "rigid_world.phase8.joint.branch_state",
            "rigid_world.phase8.joint.reaction_force.x",
            "rigid_world.phase8.joint.reaction_force.y",
            "rigid_world.phase8.joint.reaction_torque",
            "rigid_world.phase8.observations.order",
        ],
        RigidWorldWitnessFamily::GearDependenciesAndFourBodySolver => &[
            "rigid_world.phase8.joint.id",
            "rigid_world.phase8.joint.kind",
            "rigid_world.phase8.joint.body_ids",
            "rigid_world.phase8.joint.dependencies.order",
            "rigid_world.phase8.joint.anchor.x",
            "rigid_world.phase8.joint.anchor.y",
            "rigid_world.phase8.joint.coordinate",
            "rigid_world.phase8.joint.speed",
            "rigid_world.phase8.joint.branch_state",
            "rigid_world.phase8.joint.reaction_force.x",
            "rigid_world.phase8.joint.reaction_force.y",
            "rigid_world.phase8.joint.reaction_torque",
            "rigid_world.phase8.observations.order",
        ],
        RigidWorldWitnessFamily::StandaloneRopeEvolution => &[
            "rigid_world.phase8.rope.id",
            "rigid_world.phase8.rope.vertex_count",
            "rigid_world.phase8.rope.configuration.bits",
            "rigid_world.phase8.rope.vertex.x",
            "rigid_world.phase8.rope.vertex.y",
            "rigid_world.phase8.rope.angle",
            "rigid_world.phase8.observations.order",
        ],
        RigidWorldWitnessFamily::ContactFilterListenerAndPreSolveTiming => &[
            "rigid_world.phase8.lifecycle.order",
            "rigid_world.phase8.lifecycle.kind",
            "rigid_world.phase8.lifecycle.identity",
            "rigid_world.phase8.lifecycle.multiplicity",
            "rigid_world.phase8.filter.directive.bits",
            "rigid_world.phase8.pre_solve.friction.bits",
            "rigid_world.phase8.pre_solve.restitution.bits",
            "rigid_world.phase8.pre_solve.tangent_speed.bits",
        ],
        RigidWorldWitnessFamily::DestructionListenerAndDependencyCascades => &[
            "rigid_world.phase8.lifecycle.order",
            "rigid_world.phase8.lifecycle.kind",
            "rigid_world.phase8.lifecycle.identity",
            "rigid_world.phase8.lifecycle.multiplicity",
        ],
        RigidWorldWitnessFamily::DiagnosticReconstructionAndDumpOrder => &[
            "rigid_world.phase8.reconstruction.order",
            "rigid_world.phase8.reconstruction.kind",
            "rigid_world.phase8.reconstruction.support",
            "rigid_world.phase8.reconstruction.dependencies.order",
            "rigid_world.phase8.diagnostics.counts",
            "rigid_world.phase8.diagnostics.tree_quality",
            "rigid_world.phase8.dump.order",
            "rigid_world.phase8.field.presence",
        ],
        _ => &[],
    }
}
