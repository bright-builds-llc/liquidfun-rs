//! Identity, header, and typed observation comparison.

use liquidfun_test_protocol::{
    CanonicalCheckpoint, CheckpointPosition, MathProbePolicyPath, NumericObservation,
    Phase4PolicyProfile, StructuralObservation, StructuralValue,
};

use super::super::{ComparisonError, ComparisonKind, ComparisonState};
use super::builder::EntryBuilder;

pub(super) fn ensure_identity(
    rust: &CanonicalCheckpoint,
    oracle: &CanonicalCheckpoint,
) -> Result<(), ComparisonError> {
    let identities = [
        (
            rust.protocol_version() == oracle.protocol_version(),
            "protocol_version",
        ),
        (rust.record_kind() == oracle.record_kind(), "record_kind"),
        (
            rust.schema_version() == oracle.schema_version(),
            "checkpoint_schema_version",
        ),
        (rust.request_id() == oracle.request_id(), "request_id"),
        (
            rust.resolved_sha256() == oracle.resolved_sha256(),
            "resolved_sha256",
        ),
        (
            rust.checkpoint_id() == oracle.checkpoint_id(),
            "checkpoint_id",
        ),
    ];
    if let Some((_, field)) = identities.into_iter().find(|(matches, _)| !matches) {
        return Err(ComparisonError::IdentityMismatch { field });
    }
    Ok(())
}

pub(super) fn compare_header(
    builder: &mut EntryBuilder<'_>,
    rust: &CanonicalCheckpoint,
    oracle: &CanonicalCheckpoint,
    policies: &Phase4PolicyProfile,
) -> Result<(), ComparisonError> {
    builder.exact(
        "checkpoint.protocol_version",
        ComparisonKind::Identity,
        &rust.protocol_version(),
        &oracle.protocol_version(),
        "closed protocol identity",
    )?;
    builder.exact(
        "checkpoint.record_kind",
        ComparisonKind::Kind,
        &rust.record_kind(),
        &oracle.record_kind(),
        "closed checkpoint record kind",
    )?;
    builder.exact(
        "checkpoint.schema_version",
        ComparisonKind::Identity,
        &rust.schema_version(),
        &oracle.schema_version(),
        "closed checkpoint schema identity",
    )?;
    builder.exact(
        "checkpoint.request_id",
        ComparisonKind::Identity,
        rust.request_id(),
        oracle.request_id(),
        "stable run identity",
    )?;
    builder.exact(
        "checkpoint.resolved_sha256",
        ComparisonKind::Identity,
        rust.resolved_sha256(),
        oracle.resolved_sha256(),
        "exact resolved scenario identity",
    )?;
    builder.exact(
        "checkpoint.checkpoint_id",
        ComparisonKind::Identity,
        rust.checkpoint_id(),
        oracle.checkpoint_id(),
        "stable checkpoint identity",
    )?;
    compare_position(builder, rust.position(), oracle.position())?;
    builder.numeric(
        "checkpoint.simulation_time",
        rust.simulation_time_bits(),
        oracle.simulation_time_bits(),
        MathProbePolicyPath::MathOperationAbs,
        policies,
        None,
        "exact accumulated logical time; wall-clock duration excluded",
    )
}

fn compare_position(
    builder: &mut EntryBuilder<'_>,
    rust: &CheckpointPosition,
    oracle: &CheckpointPosition,
) -> Result<(), ComparisonError> {
    builder.exact(
        "checkpoint.position.kind",
        ComparisonKind::Kind,
        &position_kind(rust),
        &position_kind(oracle),
        "closed semantic checkpoint boundary",
    )?;
    match (rust, oracle) {
        (
            CheckpointPosition::Action {
                after_action_id: rust_id,
                ordinal: rust_ordinal,
            },
            CheckpointPosition::Action {
                after_action_id: oracle_id,
                ordinal: oracle_ordinal,
            },
        ) => {
            builder.exact(
                "checkpoint.position.after_action_id",
                ComparisonKind::Identity,
                rust_id,
                oracle_id,
                "stable action boundary identity",
            )?;
            builder.exact(
                "checkpoint.position.ordinal",
                ComparisonKind::Count,
                rust_ordinal,
                oracle_ordinal,
                "source-significant action ordinal",
            )
        }
        (
            CheckpointPosition::LogicalStep {
                ordinal: rust_ordinal,
            },
            CheckpointPosition::LogicalStep {
                ordinal: oracle_ordinal,
            },
        ) => builder.exact(
            "checkpoint.position.ordinal",
            ComparisonKind::Count,
            rust_ordinal,
            oracle_ordinal,
            "logical-step ordinal",
        ),
        _ => Ok(()),
    }
}

const fn position_kind(position: &CheckpointPosition) -> &'static str {
    match position {
        CheckpointPosition::Action { .. } => "action",
        CheckpointPosition::LogicalStep { .. } => "logical_step",
    }
}

pub(super) fn compare_structural_observations(
    builder: &mut EntryBuilder<'_>,
    rust: &[StructuralObservation],
    oracle: &[StructuralObservation],
) -> Result<(), ComparisonError> {
    let mut rust_index = 0;
    let mut oracle_index = 0;
    while rust_index < rust.len() || oracle_index < oracle.len() {
        match (rust.get(rust_index), oracle.get(oracle_index)) {
            (Some(rust_value), Some(oracle_value)) => {
                match rust_value
                    .observation_id()
                    .cmp(oracle_value.observation_id())
                {
                    std::cmp::Ordering::Less => {
                        builder.only(
                            &format!(
                                "observations.{}.presence",
                                rust_value.observation_id().as_str()
                            ),
                            ComparisonKind::Presence,
                            ComparisonState::RustOnly,
                            Some(rust_value),
                            None::<&StructuralObservation>,
                            "structural observation missing from oracle",
                        )?;
                        rust_index += 1;
                    }
                    std::cmp::Ordering::Greater => {
                        builder.only(
                            &format!(
                                "observations.{}.presence",
                                oracle_value.observation_id().as_str()
                            ),
                            ComparisonKind::Presence,
                            ComparisonState::OracleOnly,
                            None::<&StructuralObservation>,
                            Some(oracle_value),
                            "structural observation missing from Rust",
                        )?;
                        oracle_index += 1;
                    }
                    std::cmp::Ordering::Equal => {
                        compare_structural_observation(builder, rust_value, oracle_value)?;
                        rust_index += 1;
                        oracle_index += 1;
                    }
                }
            }
            (Some(value), None) => {
                builder.only(
                    &format!("observations.{}.presence", value.observation_id().as_str()),
                    ComparisonKind::Presence,
                    ComparisonState::RustOnly,
                    Some(value),
                    None::<&StructuralObservation>,
                    "structural observation missing from oracle",
                )?;
                rust_index += 1;
            }
            (None, Some(value)) => {
                builder.only(
                    &format!("observations.{}.presence", value.observation_id().as_str()),
                    ComparisonKind::Presence,
                    ComparisonState::OracleOnly,
                    None::<&StructuralObservation>,
                    Some(value),
                    "structural observation missing from Rust",
                )?;
                oracle_index += 1;
            }
            (None, None) => break,
        }
    }
    Ok(())
}

fn compare_structural_observation(
    builder: &mut EntryBuilder<'_>,
    rust: &StructuralObservation,
    oracle: &StructuralObservation,
) -> Result<(), ComparisonError> {
    let base = format!("observations.{}", rust.observation_id().as_str());
    builder.exact(
        &format!("{base}.kind"),
        ComparisonKind::Kind,
        &structural_kind(rust.value()),
        &structural_kind(oracle.value()),
        "closed structural observation kind",
    )?;
    let kind = match rust.value() {
        StructuralValue::Presence { .. } => ComparisonKind::Presence,
        StructuralValue::Count(_) => ComparisonKind::Count,
        StructuralValue::FlagBits(_) => ComparisonKind::Flags,
        StructuralValue::Identity { .. } | StructuralValue::Status { .. } => {
            ComparisonKind::Identity
        }
    };
    builder.exact(
        &format!("{base}.value"),
        kind,
        rust.value(),
        oracle.value(),
        "exact structural observation value",
    )
}

const fn structural_kind(value: &StructuralValue) -> &'static str {
    match value {
        StructuralValue::Presence { .. } => "presence",
        StructuralValue::Count(_) => "count",
        StructuralValue::FlagBits(_) => "flag_bits",
        StructuralValue::Identity { .. } => "identity",
        StructuralValue::Status { .. } => "status",
    }
}

pub(super) fn compare_numeric_observations(
    builder: &mut EntryBuilder<'_>,
    rust: &[NumericObservation],
    oracle: &[NumericObservation],
    policies: &Phase4PolicyProfile,
) -> Result<(), ComparisonError> {
    let mut rust_index = 0;
    let mut oracle_index = 0;
    while rust_index < rust.len() || oracle_index < oracle.len() {
        match (rust.get(rust_index), oracle.get(oracle_index)) {
            (Some(rust_value), Some(oracle_value)) => {
                match rust_value
                    .observation_id()
                    .cmp(oracle_value.observation_id())
                {
                    std::cmp::Ordering::Less => {
                        numeric_only(builder, rust_value, ComparisonState::RustOnly)?;
                        rust_index += 1;
                    }
                    std::cmp::Ordering::Greater => {
                        numeric_only(builder, oracle_value, ComparisonState::OracleOnly)?;
                        oracle_index += 1;
                    }
                    std::cmp::Ordering::Equal => {
                        let base = format!(
                            "numeric_observations.{}",
                            rust_value.observation_id().as_str()
                        );
                        builder.exact(
                            &format!("{base}.policy_path"),
                            ComparisonKind::Identity,
                            &rust_value.policy_path(),
                            &oracle_value.policy_path(),
                            "closed numeric policy identity",
                        )?;
                        if rust_value.policy_path() == oracle_value.policy_path() {
                            builder.numeric(
                                &format!("{base}.value"),
                                rust_value.value_bits(),
                                oracle_value.value_bits(),
                                rust_value.policy_path(),
                                policies,
                                None,
                                "bounded semantic numeric observation",
                            )?;
                        }
                        rust_index += 1;
                        oracle_index += 1;
                    }
                }
            }
            (Some(value), None) => {
                numeric_only(builder, value, ComparisonState::RustOnly)?;
                rust_index += 1;
            }
            (None, Some(value)) => {
                numeric_only(builder, value, ComparisonState::OracleOnly)?;
                oracle_index += 1;
            }
            (None, None) => break,
        }
    }
    Ok(())
}

fn numeric_only(
    builder: &mut EntryBuilder<'_>,
    value: &NumericObservation,
    state: ComparisonState,
) -> Result<(), ComparisonError> {
    builder.only(
        &format!(
            "numeric_observations.{}.presence",
            value.observation_id().as_str()
        ),
        ComparisonKind::Presence,
        state,
        (state == ComparisonState::RustOnly).then_some(value),
        (state == ComparisonState::OracleOnly).then_some(value),
        "numeric observation exists on only one backend",
    )
}
