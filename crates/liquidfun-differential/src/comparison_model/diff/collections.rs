//! Source-ordered occurrences, canonical sets, and profile-name comparison.

use liquidfun_test_protocol::{CheckpointProfileName, CheckpointSet, OrderedOccurrence};

use super::super::{ComparisonError, ComparisonKind, ComparisonState};
use super::builder::EntryBuilder;

pub(super) fn compare_occurrences(
    builder: &mut EntryBuilder<'_>,
    rust: &[OrderedOccurrence],
    oracle: &[OrderedOccurrence],
) -> Result<(), ComparisonError> {
    for index in 0..rust.len().max(oracle.len()) {
        let base = format!("ordered_occurrences.{index}");
        match (rust.get(index), oracle.get(index)) {
            (Some(rust_value), Some(oracle_value)) => {
                builder.exact(
                    &format!("{base}.occurrence_id"),
                    ComparisonKind::Order,
                    rust_value.occurrence_id(),
                    oracle_value.occurrence_id(),
                    "source-significant occurrence identity",
                )?;
                builder.exact(
                    &format!("{base}.kind"),
                    ComparisonKind::Kind,
                    &rust_value.kind(),
                    &oracle_value.kind(),
                    "closed occurrence kind",
                )?;
                builder.exact(
                    &format!("{base}.owner_id"),
                    ComparisonKind::Identity,
                    rust_value.owner_id(),
                    oracle_value.owner_id(),
                    "stable occurrence owner",
                )?;
            }
            (Some(value), None) => builder.only(
                &format!("{base}.presence"),
                ComparisonKind::Presence,
                ComparisonState::RustOnly,
                Some(value),
                None::<&OrderedOccurrence>,
                "source-significant occurrence missing from oracle",
            )?,
            (None, Some(value)) => builder.only(
                &format!("{base}.presence"),
                ComparisonKind::Presence,
                ComparisonState::OracleOnly,
                None::<&OrderedOccurrence>,
                Some(value),
                "source-significant occurrence missing from Rust",
            )?,
            (None, None) => {}
        }
    }
    Ok(())
}

pub(super) fn compare_sets(
    builder: &mut EntryBuilder<'_>,
    rust: &[CheckpointSet],
    oracle: &[CheckpointSet],
) -> Result<(), ComparisonError> {
    let mut rust_index = 0;
    let mut oracle_index = 0;
    while rust_index < rust.len() || oracle_index < oracle.len() {
        match (rust.get(rust_index), oracle.get(oracle_index)) {
            (Some(rust_value), Some(oracle_value)) => {
                match rust_value.set_id().cmp(oracle_value.set_id()) {
                    std::cmp::Ordering::Less => {
                        set_only(builder, rust_value, ComparisonState::RustOnly)?;
                        rust_index += 1;
                    }
                    std::cmp::Ordering::Greater => {
                        set_only(builder, oracle_value, ComparisonState::OracleOnly)?;
                        oracle_index += 1;
                    }
                    std::cmp::Ordering::Equal => {
                        let base = format!("unordered_sets.{}", rust_value.set_id().as_str());
                        builder.exact(
                            &format!("{base}.member_count"),
                            ComparisonKind::Count,
                            &rust_value.members().len(),
                            &oracle_value.members().len(),
                            "canonical unique-member count",
                        )?;
                        for member_index in
                            0..rust_value.members().len().max(oracle_value.members().len())
                        {
                            match (
                                rust_value.members().get(member_index),
                                oracle_value.members().get(member_index),
                            ) {
                                (Some(rust_member), Some(oracle_member)) => builder.exact(
                                    &format!("{base}.members.{member_index}"),
                                    ComparisonKind::Membership,
                                    rust_member,
                                    oracle_member,
                                    "declared unordered set in canonical semantic-ID order",
                                )?,
                                (Some(value), None) => builder.only(
                                    &format!("{base}.members.{member_index}"),
                                    ComparisonKind::Membership,
                                    ComparisonState::RustOnly,
                                    Some(value),
                                    None::<&liquidfun_test_protocol::ScenarioId>,
                                    "canonical set member missing from oracle",
                                )?,
                                (None, Some(value)) => builder.only(
                                    &format!("{base}.members.{member_index}"),
                                    ComparisonKind::Membership,
                                    ComparisonState::OracleOnly,
                                    None::<&liquidfun_test_protocol::ScenarioId>,
                                    Some(value),
                                    "canonical set member missing from Rust",
                                )?,
                                (None, None) => {}
                            }
                        }
                        rust_index += 1;
                        oracle_index += 1;
                    }
                }
            }
            (Some(value), None) => {
                set_only(builder, value, ComparisonState::RustOnly)?;
                rust_index += 1;
            }
            (None, Some(value)) => {
                set_only(builder, value, ComparisonState::OracleOnly)?;
                oracle_index += 1;
            }
            (None, None) => break,
        }
    }
    Ok(())
}

fn set_only(
    builder: &mut EntryBuilder<'_>,
    value: &CheckpointSet,
    state: ComparisonState,
) -> Result<(), ComparisonError> {
    builder.only(
        &format!("unordered_sets.{}.presence", value.set_id().as_str()),
        ComparisonKind::Presence,
        state,
        (state == ComparisonState::RustOnly).then_some(value),
        (state == ComparisonState::OracleOnly).then_some(value),
        "declared unordered set exists on only one backend",
    )
}

pub(super) fn compare_profiles(
    builder: &mut EntryBuilder<'_>,
    rust: &[CheckpointProfileName],
    oracle: &[CheckpointProfileName],
) -> Result<(), ComparisonError> {
    builder.exact(
        "profile_names.count",
        ComparisonKind::Count,
        &rust.len(),
        &oracle.len(),
        "instrumentation profile names only; duration values excluded",
    )?;
    for index in 0..rust.len().max(oracle.len()) {
        match (rust.get(index), oracle.get(index)) {
            (Some(rust_value), Some(oracle_value)) => builder.exact(
                &format!("profile_names.{index}"),
                ComparisonKind::Kind,
                rust_value,
                oracle_value,
                "closed profile name in canonical order",
            )?,
            (Some(value), None) => builder.only(
                &format!("profile_names.{index}"),
                ComparisonKind::Presence,
                ComparisonState::RustOnly,
                Some(value),
                None::<&CheckpointProfileName>,
                "profile name missing from oracle",
            )?,
            (None, Some(value)) => builder.only(
                &format!("profile_names.{index}"),
                ComparisonKind::Presence,
                ComparisonState::OracleOnly,
                None::<&CheckpointProfileName>,
                Some(value),
                "profile name missing from Rust",
            )?,
            (None, None) => {}
        }
    }
    Ok(())
}
