//! Closed Phase 4 policy bindings for canonical checkpoint numerics.

use liquidfun_test_protocol::{
    FieldComparison, FloatBits, MathProbePolicyPath, Phase4PolicyProfile,
};

use crate::{ComparisonError, ComparisonState, float_values_match_with_policy};

pub(super) fn compare_numeric(
    rust: FloatBits,
    oracle: FloatBits,
    path: MathProbePolicyPath,
    profile: &Phase4PolicyProfile,
) -> Result<ComparisonState, ComparisonError> {
    let path_text = path.as_str();
    reject_open_path(path_text)?;
    let field = profile
        .field(path_text)
        .ok_or(ComparisonError::InvalidPolicyBinding)?;
    if !matches!(field.comparison(), FieldComparison::Float { .. }) {
        return Err(ComparisonError::InvalidPolicyBinding);
    }
    if rust == oracle {
        return Ok(ComparisonState::ExactMatch);
    }
    Ok(if float_values_match_with_policy(oracle, rust, field) {
        ComparisonState::WithinPolicy
    } else {
        ComparisonState::PhysicsMismatch
    })
}

pub(super) fn reject_open_path(path: &str) -> Result<(), ComparisonError> {
    let has_open_segment = path.split('.').any(|segment| {
        matches!(
            segment.to_ascii_lowercase().as_str(),
            "*" | "**" | "any" | "all" | "default" | "fallback" | "private"
        )
    });
    if path.is_empty()
        || path.len() > 256
        || path
            .chars()
            .any(|value| matches!(value, '*' | '?' | '[' | ']'))
        || has_open_segment
    {
        return Err(ComparisonError::InvalidSemanticPath);
    }
    Ok(())
}
