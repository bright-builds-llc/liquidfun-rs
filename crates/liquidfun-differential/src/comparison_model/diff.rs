//! Exhaustive canonical checkpoint field walk.

mod builder;
mod collections;
mod header;
mod primitives;

use liquidfun_test_protocol::{CanonicalCheckpoint, Phase4PolicyProfile};

use self::{
    builder::EntryBuilder,
    collections::{compare_occurrences, compare_profiles, compare_sets},
    header::{
        compare_header, compare_numeric_observations, compare_structural_observations,
        ensure_identity,
    },
    primitives::compare_primitives,
};
use super::{ComparisonError, ComparisonLimits, ComparisonModel, overall_state};

/// Compares two compatible canonical checkpoints through one exhaustive semantic walk.
///
/// # Errors
///
/// Returns a harness error for incompatible identities, unbound numeric paths, invalid generated
/// paths, or an entry-limit overflow. Physics differences remain successful model entries.
pub fn compare_canonical_checkpoints(
    rust: &CanonicalCheckpoint,
    oracle: &CanonicalCheckpoint,
    policies: &Phase4PolicyProfile,
    limits: ComparisonLimits,
) -> Result<ComparisonModel, ComparisonError> {
    ensure_identity(rust, oracle)?;
    let mut builder = EntryBuilder::new(limits, rust.checkpoint_id());
    compare_header(&mut builder, rust, oracle, policies)?;
    compare_structural_observations(&mut builder, rust.observations(), oracle.observations())?;
    compare_numeric_observations(
        &mut builder,
        rust.numeric_observations(),
        oracle.numeric_observations(),
        policies,
    )?;
    compare_occurrences(
        &mut builder,
        rust.ordered_occurrences(),
        oracle.ordered_occurrences(),
    )?;
    compare_sets(&mut builder, rust.unordered_sets(), oracle.unordered_sets())?;
    compare_primitives(
        &mut builder,
        rust.debug_primitives(),
        oracle.debug_primitives(),
        policies,
    )?;
    compare_profiles(&mut builder, rust.profile_names(), oracle.profile_names())?;

    let entries = builder.finish();
    Ok(ComparisonModel {
        request_id: rust.request_id().clone(),
        resolved_sha256: rust.resolved_sha256().clone(),
        checkpoint_id: rust.checkpoint_id().clone(),
        state: overall_state(&entries),
        entries,
    })
}
