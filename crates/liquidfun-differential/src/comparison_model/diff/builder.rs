//! Bounded comparison entry construction and stable mismatch signatures.

use std::fmt::Debug;

use liquidfun_test_protocol::{
    DebugPrimitiveKey, FloatBits, MathProbePolicyPath, Phase4PolicyProfile,
};

use crate::{MismatchKind, report::checkpoint_mismatch_signature_sha256};

use super::super::{
    ComparisonEntry, ComparisonError, ComparisonKind, ComparisonLimits, ComparisonState,
    policy::{compare_numeric, reject_open_path},
};

pub(super) struct EntryBuilder<'a> {
    limits: ComparisonLimits,
    checkpoint_id: &'a liquidfun_test_protocol::CheckpointId,
    entries: Vec<ComparisonEntry>,
}

impl<'a> EntryBuilder<'a> {
    pub(super) fn new(
        limits: ComparisonLimits,
        checkpoint_id: &'a liquidfun_test_protocol::CheckpointId,
    ) -> Self {
        Self {
            limits,
            checkpoint_id,
            entries: Vec::new(),
        }
    }

    pub(super) fn finish(self) -> Box<[ComparisonEntry]> {
        self.entries.into_boxed_slice()
    }

    pub(super) fn exact<T: Debug + PartialEq + ?Sized>(
        &mut self,
        path: &str,
        kind: ComparisonKind,
        rust: &T,
        oracle: &T,
        context: &str,
    ) -> Result<(), ComparisonError> {
        self.exact_with_key(path, kind, rust, oracle, None, context)
    }

    pub(super) fn exact_with_key<T: Debug + PartialEq + ?Sized>(
        &mut self,
        path: &str,
        kind: ComparisonKind,
        rust: &T,
        oracle: &T,
        maybe_key: Option<DebugPrimitiveKey>,
        context: &str,
    ) -> Result<(), ComparisonError> {
        let state = if rust == oracle {
            ComparisonState::ExactMatch
        } else {
            ComparisonState::PhysicsMismatch
        };
        self.push(
            path,
            kind,
            state,
            None,
            Some(format!("{rust:?}")),
            Some(format!("{oracle:?}")),
            maybe_key,
            context,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "one numeric entry binds both values, policy, focus key, and context"
    )]
    pub(super) fn numeric(
        &mut self,
        path: &str,
        rust: FloatBits,
        oracle: FloatBits,
        policy_path: MathProbePolicyPath,
        policies: &Phase4PolicyProfile,
        maybe_key: Option<DebugPrimitiveKey>,
        context: &str,
    ) -> Result<(), ComparisonError> {
        let state = compare_numeric(rust, oracle, policy_path, policies)?;
        self.push(
            path,
            ComparisonKind::Numeric,
            state,
            Some(policy_path),
            Some(format!("0x{:08x}", rust.bits())),
            Some(format!("0x{:08x}", oracle.bits())),
            maybe_key,
            context,
        )
    }

    pub(super) fn only<T: Debug + ?Sized>(
        &mut self,
        path: &str,
        kind: ComparisonKind,
        state: ComparisonState,
        rust: Option<&T>,
        oracle: Option<&T>,
        context: &str,
    ) -> Result<(), ComparisonError> {
        self.only_with_key(path, kind, state, rust, oracle, None, context)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "one missing-side entry binds values, focus key, and bounded context"
    )]
    pub(super) fn only_with_key<T: Debug + ?Sized>(
        &mut self,
        path: &str,
        kind: ComparisonKind,
        state: ComparisonState,
        rust: Option<&T>,
        oracle: Option<&T>,
        maybe_key: Option<DebugPrimitiveKey>,
        context: &str,
    ) -> Result<(), ComparisonError> {
        self.push(
            path,
            kind,
            state,
            None,
            rust.map(|value| format!("{value:?}")),
            oracle.map(|value| format!("{value:?}")),
            maybe_key,
            context,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the authoritative entry owns path, state, policy, values, focus, and context"
    )]
    pub(super) fn push(
        &mut self,
        path: &str,
        kind: ComparisonKind,
        state: ComparisonState,
        maybe_policy_path: Option<MathProbePolicyPath>,
        maybe_rust_value: Option<String>,
        maybe_oracle_value: Option<String>,
        maybe_primitive_key: Option<DebugPrimitiveKey>,
        context: &str,
    ) -> Result<(), ComparisonError> {
        reject_open_path(path)?;
        if self.entries.len() >= self.limits.entry_count {
            return Err(ComparisonError::EntryLimitExceeded);
        }
        let maybe_signature_sha256 = (state != ComparisonState::ExactMatch
            && state != ComparisonState::WithinPolicy)
            .then(|| {
                checkpoint_mismatch_signature_sha256(
                    self.checkpoint_id,
                    path,
                    mismatch_kind(kind, state),
                )
            });
        self.entries.push(ComparisonEntry {
            semantic_path: path.into(),
            kind,
            state,
            maybe_policy_path,
            maybe_rust_value: maybe_rust_value
                .map(|value| bound_text(value, self.limits.value_bytes)),
            maybe_oracle_value: maybe_oracle_value
                .map(|value| bound_text(value, self.limits.value_bytes)),
            maybe_primitive_key,
            context: bound_text(context.to_owned(), self.limits.context_bytes),
            maybe_signature_sha256,
        });
        Ok(())
    }
}

fn mismatch_kind(kind: ComparisonKind, state: ComparisonState) -> MismatchKind {
    match state {
        ComparisonState::RustOnly => MismatchKind::Unexpected,
        ComparisonState::OracleOnly => MismatchKind::Missing,
        ComparisonState::PhysicsMismatch if kind == ComparisonKind::Numeric => {
            MismatchKind::Numeric
        }
        ComparisonState::PhysicsMismatch if kind == ComparisonKind::Order => MismatchKind::Order,
        ComparisonState::PhysicsMismatch
        | ComparisonState::ExactMatch
        | ComparisonState::WithinPolicy => MismatchKind::Exact,
    }
}

fn bound_text(mut value: String, maximum_bytes: usize) -> Box<str> {
    if value.len() <= maximum_bytes {
        return value.into_boxed_str();
    }
    let suffix = "…";
    let target = maximum_bytes.saturating_sub(suffix.len());
    let boundary = value
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= target)
        .last()
        .unwrap_or(0);
    value.truncate(boundary);
    value.push_str(suffix);
    value.into_boxed_str()
}
