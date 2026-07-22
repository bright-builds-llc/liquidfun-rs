//! Engine-neutral structural, numeric, occurrence, and set observations.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{FloatBits, MathProbePolicyPath, ScenarioId, codec::BoundedVec};

pub(super) const CHECKPOINT_MAXIMUM_SET_MEMBERS: usize = 4_096;

/// Closed exact structural values accepted by canonical checkpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum StructuralValue {
    /// One exact boolean state.
    Presence {
        /// Exact presence state.
        present: bool,
    },
    /// One exact non-negative count.
    Count(u64),
    /// One exact public flag mask.
    FlagBits(u64),
    /// One stable semantic identity.
    Identity {
        /// Stable semantic identity.
        semantic_id: ScenarioId,
    },
    /// One exact bounded UTF-8-free status code represented as a stable ID.
    Status {
        /// Stable closed status identity.
        status_id: ScenarioId,
    },
}

/// One exact structural observation keyed by a stable semantic path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralObservation {
    observation_id: ScenarioId,
    value: StructuralValue,
}

impl StructuralObservation {
    /// Creates one exact structural observation.
    #[must_use]
    pub const fn new(observation_id: ScenarioId, value: StructuralValue) -> Self {
        Self {
            observation_id,
            value,
        }
    }

    /// Returns the stable semantic observation identity.
    #[must_use]
    pub const fn observation_id(&self) -> &ScenarioId {
        &self.observation_id
    }

    /// Returns the exact structural value.
    #[must_use]
    pub const fn value(&self) -> &StructuralValue {
        &self.value
    }
}

/// One exact floating observation bound to the closed Phase 4 policy registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericObservation {
    observation_id: ScenarioId,
    value_bits: FloatBits,
    policy_path: MathProbePolicyPath,
}

impl NumericObservation {
    /// Creates one exact-bit observation with an explicit reviewed policy path.
    #[must_use]
    pub const fn new(
        observation_id: ScenarioId,
        value_bits: FloatBits,
        policy_path: MathProbePolicyPath,
    ) -> Self {
        Self {
            observation_id,
            value_bits,
            policy_path,
        }
    }

    /// Returns the stable semantic observation identity.
    #[must_use]
    pub const fn observation_id(&self) -> &ScenarioId {
        &self.observation_id
    }

    /// Returns the exact IEEE-754 bits.
    #[must_use]
    pub const fn value_bits(&self) -> FloatBits {
        self.value_bits
    }

    /// Returns the closed reviewed Phase 4 policy path.
    #[must_use]
    pub const fn policy_path(&self) -> MathProbePolicyPath {
        self.policy_path
    }
}

/// Closed source-significant occurrence kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OccurrenceKind {
    /// A rigid contact began.
    ContactBegin,
    /// A rigid contact persisted.
    ContactPersist,
    /// A rigid contact ended.
    ContactEnd,
    /// A particle-particle contact began.
    ParticleContactBegin,
    /// A particle-particle contact ended.
    ParticleContactEnd,
    /// A particle-body contact began.
    BodyContactBegin,
    /// A particle-body contact ended.
    BodyContactEnd,
    /// A semantic object was destroyed.
    Destruction,
    /// A public callback occurrence was observed.
    Callback,
    /// A typed mutation was accepted.
    Mutation,
}

/// One source-significant occurrence with stable identity and owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderedOccurrence {
    occurrence_id: ScenarioId,
    kind: OccurrenceKind,
    owner_id: ScenarioId,
}

impl OrderedOccurrence {
    /// Creates one source-significant occurrence.
    #[must_use]
    pub const fn new(
        occurrence_id: ScenarioId,
        kind: OccurrenceKind,
        owner_id: ScenarioId,
    ) -> Self {
        Self {
            occurrence_id,
            kind,
            owner_id,
        }
    }

    /// Returns the stable occurrence identity.
    #[must_use]
    pub const fn occurrence_id(&self) -> &ScenarioId {
        &self.occurrence_id
    }

    /// Returns the closed occurrence kind.
    #[must_use]
    pub const fn kind(&self) -> OccurrenceKind {
        self.kind
    }

    /// Returns the stable semantic owner.
    #[must_use]
    pub const fn owner_id(&self) -> &ScenarioId {
        &self.owner_id
    }
}

/// One explicitly unordered set canonicalized by stable semantic identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckpointSet {
    set_id: ScenarioId,
    members: Box<[ScenarioId]>,
}

impl CheckpointSet {
    /// Sorts and validates one explicitly unordered semantic set.
    ///
    /// # Errors
    ///
    /// Returns [`super::CheckpointValidationError`] for duplicate or excessive members.
    pub fn new(
        set_id: ScenarioId,
        mut members: Vec<ScenarioId>,
    ) -> Result<Self, super::CheckpointValidationError> {
        if members.len() > CHECKPOINT_MAXIMUM_SET_MEMBERS {
            return Err(super::validation(
                super::CheckpointErrorKind::BoundaryLimitExceeded,
            ));
        }
        members.sort_unstable();
        if members.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(super::validation(
                super::CheckpointErrorKind::DuplicateSemanticId,
            ));
        }
        Ok(Self {
            set_id,
            members: members.into_boxed_slice(),
        })
    }

    /// Returns the stable set identity.
    #[must_use]
    pub const fn set_id(&self) -> &ScenarioId {
        &self.set_id
    }

    /// Returns members in canonical stable-ID order.
    #[must_use]
    pub fn members(&self) -> &[ScenarioId] {
        &self.members
    }
}

impl<'de> Deserialize<'de> for CheckpointSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawCheckpointSet {
            set_id: ScenarioId,
            members: BoundedVec<ScenarioId, CHECKPOINT_MAXIMUM_SET_MEMBERS>,
        }

        let raw = RawCheckpointSet::deserialize(deserializer)?;
        let members = raw.members.into_vec();
        if members.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(serde::de::Error::custom(
                "unordered set members must be unique and canonically ordered",
            ));
        }
        Ok(Self {
            set_id: raw.set_id,
            members: members.into_boxed_slice(),
        })
    }
}
