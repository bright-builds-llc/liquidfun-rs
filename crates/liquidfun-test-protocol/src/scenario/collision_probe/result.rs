use std::collections::HashSet;

use serde::Serialize;

use super::types::{
    CollisionProbeDecodeError, CollisionProbeErrorKind, CollisionProbeHorizon,
    CollisionProbeOperation, CollisionRejectionCategory, CollisionRejectionField, validation,
};
use crate::{FloatBits, tolerance::CollectionPolicy};

const MAXIMUM_RESULT_FIELDS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionProbeNumericValue {
    field: Box<str>,
    bits: FloatBits,
}

impl CollisionProbeNumericValue {
    #[must_use]
    pub fn new(field: impl Into<Box<str>>, bits: FloatBits) -> Self {
        Self {
            field: field.into(),
            bits,
        }
    }
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }
    #[must_use]
    pub const fn bits(&self) -> FloatBits {
        self.bits
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionProbeDiscreteValue {
    field: Box<str>,
    value: Box<str>,
}

impl CollisionProbeDiscreteValue {
    #[must_use]
    pub fn new(field: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        Self {
            field: field.into(),
            value: value.into(),
        }
    }
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollisionProbeResultOutcome {
    Accepted {
        numeric: Box<[CollisionProbeNumericValue]>,
        discrete: Box<[CollisionProbeDiscreteValue]>,
        payload_ids: Box<[u32]>,
    },
    Rejected {
        category: CollisionRejectionCategory,
        field: CollisionRejectionField,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionProbeResult {
    case_id: Box<str>,
    operation: CollisionProbeOperation,
    policy_path: Box<str>,
    horizon: CollisionProbeHorizon,
    collection_policy: CollectionPolicy,
    outcome: CollisionProbeResultOutcome,
}

impl CollisionProbeResult {
    /// Creates one bounded result aligned to the operation's closed metadata.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionProbeDecodeError`] when aggregate fields exceed the
    /// reviewed limit or a set-like payload collection contains duplicates.
    pub fn new(
        case_id: impl Into<Box<str>>,
        operation: CollisionProbeOperation,
        numeric: Vec<CollisionProbeNumericValue>,
        discrete: Vec<CollisionProbeDiscreteValue>,
        payload_ids: Vec<u32>,
    ) -> Result<Self, CollisionProbeDecodeError> {
        if numeric.len() + discrete.len() + payload_ids.len() > MAXIMUM_RESULT_FIELDS {
            return Err(validation(CollisionProbeErrorKind::AggregateLimitExceeded));
        }
        if operation.expected_collection_policy() == CollectionPolicy::Set {
            let unique: HashSet<_> = payload_ids.iter().copied().collect();
            if unique.len() != payload_ids.len() {
                return Err(validation(CollisionProbeErrorKind::DuplicateSetPayload));
            }
        }
        Ok(Self {
            case_id: case_id.into(),
            operation,
            policy_path: operation.policy_path().into(),
            horizon: operation.expected_horizon(),
            collection_policy: operation.expected_collection_policy(),
            outcome: CollisionProbeResultOutcome::Accepted {
                numeric: numeric.into_boxed_slice(),
                discrete: discrete.into_boxed_slice(),
                payload_ids: payload_ids.into_boxed_slice(),
            },
        })
    }

    #[must_use]
    pub fn rejected(
        case_id: impl Into<Box<str>>,
        operation: CollisionProbeOperation,
        category: CollisionRejectionCategory,
        field: CollisionRejectionField,
    ) -> Self {
        Self {
            case_id: case_id.into(),
            operation,
            policy_path: operation.policy_path().into(),
            horizon: operation.expected_horizon(),
            collection_policy: operation.expected_collection_policy(),
            outcome: CollisionProbeResultOutcome::Rejected { category, field },
        }
    }

    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
    }
    #[must_use]
    pub const fn operation(&self) -> CollisionProbeOperation {
        self.operation
    }
    #[must_use]
    pub fn policy_path(&self) -> &str {
        &self.policy_path
    }
    #[must_use]
    pub const fn horizon(&self) -> CollisionProbeHorizon {
        self.horizon
    }
    #[must_use]
    pub const fn collection_policy(&self) -> CollectionPolicy {
        self.collection_policy
    }
    #[must_use]
    pub fn numeric(&self) -> &[CollisionProbeNumericValue] {
        match &self.outcome {
            CollisionProbeResultOutcome::Accepted { numeric, .. } => numeric,
            CollisionProbeResultOutcome::Rejected { .. } => &[],
        }
    }
    #[must_use]
    pub fn discrete(&self) -> &[CollisionProbeDiscreteValue] {
        match &self.outcome {
            CollisionProbeResultOutcome::Accepted { discrete, .. } => discrete,
            CollisionProbeResultOutcome::Rejected { .. } => &[],
        }
    }
    #[must_use]
    pub fn payload_ids(&self) -> &[u32] {
        match &self.outcome {
            CollisionProbeResultOutcome::Accepted { payload_ids, .. } => payload_ids,
            CollisionProbeResultOutcome::Rejected { .. } => &[],
        }
    }
    #[must_use]
    pub const fn outcome(&self) -> &CollisionProbeResultOutcome {
        &self.outcome
    }
}
