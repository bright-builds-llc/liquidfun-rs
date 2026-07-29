use liquidfun_test_protocol::CanonicalCheckpoint;

use crate::{CatalogRunCapture, ComparisonEntry, ComparisonState, compare_canonical_checkpoints};

use super::{
    CatalogComparisonSurface, FailureBundleError, digest, encode_checkpoint, evidence_error,
    json_line,
};

pub(super) fn comparison_json(
    surface: CatalogComparisonSurface,
    native: &CatalogRunCapture,
    oracle: &CatalogRunCapture,
) -> Result<Vec<u8>, FailureBundleError> {
    let rows = native
        .checkpoints()
        .iter()
        .zip(oracle.checkpoints())
        .map(|(rust, cpp)| {
            let model = comparison_model(rust, cpp)?;
            let entries = projection_entries(surface, &model);
            let maybe_first = entries.iter().find(|entry| is_divergence(entry.state()));
            Ok(serde_json::json!({
                "checkpoint_id": rust.checkpoint_id(),
                "native_sha256": digest(&encode_checkpoint(rust)?),
                "oracle_sha256": digest(&encode_checkpoint(cpp)?),
                "state": maybe_first.map_or("match", |entry| state_name(entry.state())),
                "maybe_first_divergence_path": maybe_first.map(|entry| entry.semantic_path()),
            }))
        })
        .collect::<Result<Vec<_>, FailureBundleError>>()?;
    json_line(&serde_json::json!({
        "surface": surface,
        "checkpoints": rows,
    }))
}

pub(super) fn first_divergence_json(
    surface: CatalogComparisonSurface,
    native: &CatalogRunCapture,
    oracle: &CatalogRunCapture,
) -> Result<Option<Vec<u8>>, FailureBundleError> {
    for (rust, cpp) in native.checkpoints().iter().zip(oracle.checkpoints()) {
        let model = comparison_model(rust, cpp)?;
        if let Some(entry) = projection_entries(surface, &model)
            .into_iter()
            .find(|entry| is_divergence(entry.state()))
        {
            return json_line(&serde_json::json!({
                "surface": surface,
                "checkpoint_id": rust.checkpoint_id(),
                "path": entry.semantic_path(),
                "comparison_state": state_name(entry.state()),
                "native": {
                    "state": if entry.maybe_rust_value().is_some() { "present" } else { "absent" },
                    "value": entry.maybe_rust_value(),
                },
                "oracle": {
                    "state": if entry.maybe_oracle_value().is_some() { "present" } else { "absent" },
                    "value": entry.maybe_oracle_value(),
                },
                "native_sha256": digest(&encode_checkpoint(rust)?),
                "oracle_sha256": digest(&encode_checkpoint(cpp)?),
            }))
            .map(Some);
        }
    }
    Ok(None)
}

fn comparison_model(
    native: &CanonicalCheckpoint,
    oracle: &CanonicalCheckpoint,
) -> Result<crate::ComparisonModel, FailureBundleError> {
    let policies = liquidfun_test_protocol::Phase4PolicyProfile::parse_toml(include_str!(
        "../../../../../protocol/tolerances/phase4-v1.toml"
    ))
    .map_err(|error| evidence_error(&error.to_string()))?;
    compare_canonical_checkpoints(
        native,
        oracle,
        &policies,
        crate::ComparisonLimits::phase11_default(),
    )
    .map_err(|error| evidence_error(&error.to_string()))
}

fn projection_entries(
    surface: CatalogComparisonSurface,
    model: &crate::ComparisonModel,
) -> Vec<&ComparisonEntry> {
    model
        .entries()
        .iter()
        .filter(|entry| {
            surface == CatalogComparisonSurface::ExpandedCheckpointV1
                || (!entry.semantic_path().starts_with("debug_primitives.")
                    && !entry
                        .semantic_path()
                        .starts_with("observations.world-debug-primitive-count."))
        })
        .collect()
}

const fn is_divergence(state: ComparisonState) -> bool {
    matches!(
        state,
        ComparisonState::PhysicsMismatch | ComparisonState::RustOnly | ComparisonState::OracleOnly
    )
}

const fn state_name(state: ComparisonState) -> &'static str {
    match state {
        ComparisonState::ExactMatch => "exact_match",
        ComparisonState::WithinPolicy => "within_policy",
        ComparisonState::PhysicsMismatch => "physics_mismatch",
        ComparisonState::RustOnly => "native_only",
        ComparisonState::OracleOnly => "oracle_only",
    }
}
