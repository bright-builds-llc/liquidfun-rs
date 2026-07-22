//! Complete renderer-neutral semantic checkpoint comparison model.

mod diff;
mod policy;

use liquidfun_test_protocol::{
    CheckpointId, DebugPrimitiveKey, MathProbePolicyPath, RequestId, Sha256Hex,
};

pub use diff::compare_canonical_checkpoints;

const MAXIMUM_COMPARISON_ENTRIES: usize = 131_072;
const MAXIMUM_VALUE_BYTES: usize = 256;
const MAXIMUM_CONTEXT_BYTES: usize = 512;

/// Reviewed resource limits for one comparison model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComparisonLimits {
    entry_count: usize,
    value_bytes: usize,
    context_bytes: usize,
}

impl ComparisonLimits {
    /// Returns the immutable Phase 11 comparison limits.
    #[must_use]
    pub const fn phase11_default() -> Self {
        Self {
            entry_count: MAXIMUM_COMPARISON_ENTRIES,
            value_bytes: MAXIMUM_VALUE_BYTES,
            context_bytes: MAXIMUM_CONTEXT_BYTES,
        }
    }

    /// Returns the default profile with a smaller entry ceiling for focused consumers and tests.
    ///
    /// # Errors
    ///
    /// Returns [`ComparisonError::InvalidLimits`] for zero or above-profile ceilings.
    pub const fn with_maximum_entries(maximum_entries: usize) -> Result<Self, ComparisonError> {
        if maximum_entries == 0 || maximum_entries > MAXIMUM_COMPARISON_ENTRIES {
            return Err(ComparisonError::InvalidLimits);
        }
        Ok(Self {
            entry_count: maximum_entries,
            value_bytes: MAXIMUM_VALUE_BYTES,
            context_bytes: MAXIMUM_CONTEXT_BYTES,
        })
    }
}

/// Complete presentation state of one semantic field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComparisonState {
    /// Both sides are exactly identical.
    ExactMatch,
    /// Different float bits remain inside the named reviewed policy.
    WithinPolicy,
    /// Both sides are present but disagree structurally or exceed numeric policy.
    PhysicsMismatch,
    /// Only the native Rust checkpoint contains the semantic field.
    RustOnly,
    /// Only the C++ oracle checkpoint contains the semantic field.
    OracleOnly,
}

/// Closed structural role of one compared field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonKind {
    /// Protocol, run, checkpoint, object, or primitive identity.
    Identity,
    /// Closed variant or occurrence kind.
    Kind,
    /// Exact public bit flags or colors.
    Flags,
    /// Exact count or ordinal.
    Count,
    /// Exact collection membership.
    Membership,
    /// Optional field or whole-record presence.
    Presence,
    /// Source-significant or declared canonical order.
    Order,
    /// Numeric value governed by one named Phase 4 policy.
    Numeric,
    /// Bounded inert semantic text.
    Text,
}

/// One bounded canonical-path comparison record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonEntry {
    semantic_path: Box<str>,
    kind: ComparisonKind,
    state: ComparisonState,
    maybe_policy_path: Option<MathProbePolicyPath>,
    maybe_rust_value: Option<Box<str>>,
    maybe_oracle_value: Option<Box<str>>,
    maybe_primitive_key: Option<DebugPrimitiveKey>,
    context: Box<str>,
    maybe_signature_sha256: Option<Sha256Hex>,
}

impl ComparisonEntry {
    /// Returns the canonical semantic path.
    #[must_use]
    pub fn semantic_path(&self) -> &str {
        &self.semantic_path
    }

    /// Returns the field's closed structural role.
    #[must_use]
    pub const fn kind(&self) -> ComparisonKind {
        self.kind
    }

    /// Returns the comparison state.
    #[must_use]
    pub const fn state(&self) -> ComparisonState {
        self.state
    }

    /// Returns the named numeric policy, when this is a numeric field.
    #[must_use]
    pub const fn maybe_policy_path(&self) -> Option<MathProbePolicyPath> {
        self.maybe_policy_path
    }

    /// Returns the bounded native Rust diagnostic value, when present.
    #[must_use]
    pub fn maybe_rust_value(&self) -> Option<&str> {
        self.maybe_rust_value.as_deref()
    }

    /// Returns the bounded C++ oracle diagnostic value, when present.
    #[must_use]
    pub fn maybe_oracle_value(&self) -> Option<&str> {
        self.maybe_oracle_value.as_deref()
    }

    /// Returns the stable primitive focus key, when both sides share one.
    #[must_use]
    pub const fn maybe_primitive_key(&self) -> Option<&DebugPrimitiveKey> {
        self.maybe_primitive_key.as_ref()
    }

    /// Returns bounded non-sensitive diagnostic context.
    #[must_use]
    pub fn context(&self) -> &str {
        &self.context
    }

    /// Returns the stable existing-report-compatible mismatch signature.
    #[must_use]
    pub const fn maybe_signature_sha256(&self) -> Option<&Sha256Hex> {
        self.maybe_signature_sha256.as_ref()
    }
}

/// Complete authoritative comparison consumed by headless and visual presentations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonModel {
    request_id: RequestId,
    resolved_sha256: Sha256Hex,
    checkpoint_id: CheckpointId,
    state: ComparisonState,
    entries: Box<[ComparisonEntry]>,
}

impl ComparisonModel {
    /// Returns the compared run request identity.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Returns the exact resolved-plan identity.
    #[must_use]
    pub const fn resolved_sha256(&self) -> &Sha256Hex {
        &self.resolved_sha256
    }

    /// Returns the compared checkpoint identity.
    #[must_use]
    pub const fn checkpoint_id(&self) -> &CheckpointId {
        &self.checkpoint_id
    }

    /// Returns the strongest state represented by the ordered entries.
    #[must_use]
    pub const fn state(&self) -> ComparisonState {
        self.state
    }

    /// Returns every field in canonical source order.
    #[must_use]
    pub fn entries(&self) -> &[ComparisonEntry] {
        &self.entries
    }
}

/// Fail-closed comparison error distinct from physics mismatch evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ComparisonError {
    /// Run, schema, or checkpoint identity differs before field comparison.
    #[error("checkpoint comparison identity mismatch: {field}")]
    IdentityMismatch {
        /// Closed identity field that disagreed.
        field: &'static str,
    },
    /// A numeric path was absent, open, private, or bound to a non-float policy.
    #[error("checkpoint comparison has an invalid numeric policy binding")]
    InvalidPolicyBinding,
    /// A configured limit is zero or exceeds the reviewed Phase 11 profile.
    #[error("checkpoint comparison limits are invalid")]
    InvalidLimits,
    /// The complete semantic walk exceeded its reviewed entry ceiling.
    #[error("checkpoint comparison entry limit exceeded")]
    EntryLimitExceeded,
    /// A generated path was open, private, wildcarded, or oversized.
    #[error("checkpoint comparison generated an invalid semantic path")]
    InvalidSemanticPath,
}

fn overall_state(entries: &[ComparisonEntry]) -> ComparisonState {
    entries
        .iter()
        .map(ComparisonEntry::state)
        .max_by_key(|state| match state {
            ComparisonState::ExactMatch => 0,
            ComparisonState::WithinPolicy => 1,
            ComparisonState::RustOnly | ComparisonState::OracleOnly => 2,
            ComparisonState::PhysicsMismatch => 3,
        })
        .unwrap_or(ComparisonState::ExactMatch)
}
