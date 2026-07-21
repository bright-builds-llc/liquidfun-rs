//! Typed witness-observation comparison.

use liquidfun_test_protocol::Phase10WitnessObservation;

use super::check;
use crate::rigid_world::phase10::comparator::{
    Phase10ComparatorError, Phase10Mismatch, mismatch_if, numeric, numeric_vec,
};

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive match proves every closed witness observation variant"
)]
pub(crate) fn compare_witness(
    scenario: &str,
    index: usize,
    expected: &Phase10WitnessObservation,
    actual: &Phase10WitnessObservation,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    let entity = format!("witness:{index}");
    match (expected, actual) {
        (
            Phase10WitnessObservation::ControlUnchanged,
            Phase10WitnessObservation::ControlUnchanged,
        ) => Ok(None),
        (
            Phase10WitnessObservation::FlagActivated { flags_bits: left },
            Phase10WitnessObservation::FlagActivated { flags_bits: right },
        ) => Ok(mismatch_if(
            scenario,
            "witness",
            &entity,
            index,
            "phase10.witness.flags",
            left,
            right,
        )),
        (
            Phase10WitnessObservation::ParticleVelocity {
                particle_id: left_id,
                before: left_before,
                after: left_after,
            },
            Phase10WitnessObservation::ParticleVelocity {
                particle_id: right_id,
                before: right_before,
                after: right_after,
            },
        ) => {
            check!(
                scenario,
                "witness",
                &entity,
                index,
                "phase10.witness.kind",
                left_id,
                right_id
            );
            numeric_vec(
                scenario,
                &entity,
                index,
                "phase10.witness.velocity",
                *left_before,
                *right_before,
            )?
            .map_or_else(
                || {
                    numeric_vec(
                        scenario,
                        &entity,
                        index,
                        "phase10.witness.velocity",
                        *left_after,
                        *right_after,
                    )
                },
                |found| Ok(Some(found)),
            )
        }
        (
            Phase10WitnessObservation::Scalar { value_bits: left },
            Phase10WitnessObservation::Scalar { value_bits: right },
        ) => numeric(
            scenario,
            &entity,
            index,
            "phase10.witness.scalar",
            *left,
            *right,
        ),
        (
            Phase10WitnessObservation::Count { value: left },
            Phase10WitnessObservation::Count { value: right },
        ) => Ok(mismatch_if(
            scenario,
            "witness",
            &entity,
            index,
            "phase10.witness.count",
            left,
            right,
        )),
        (
            Phase10WitnessObservation::Occurrence {
                event_ordinal: left,
            },
            Phase10WitnessObservation::Occurrence {
                event_ordinal: right,
            },
        ) => Ok(mismatch_if(
            scenario,
            "witness",
            &entity,
            index,
            "phase10.witness.occurrence",
            left,
            right,
        )),
        (
            Phase10WitnessObservation::Topology {
                pair_count: left_pairs,
                triad_count: left_triads,
            },
            Phase10WitnessObservation::Topology {
                pair_count: right_pairs,
                triad_count: right_triads,
            },
        ) => Ok(mismatch_if(
            scenario,
            "witness",
            &entity,
            index,
            "phase10.witness.topology",
            &(left_pairs, left_triads),
            &(right_pairs, right_triads),
        )),
        _ => Ok(mismatch_if(
            scenario,
            "witness",
            &entity,
            index,
            "phase10.witness.kind",
            &witness_kind(expected),
            &witness_kind(actual),
        )),
    }
}

const fn witness_kind(value: &Phase10WitnessObservation) -> &'static str {
    match value {
        Phase10WitnessObservation::ControlUnchanged => "control_unchanged",
        Phase10WitnessObservation::FlagActivated { .. } => "flag_activated",
        Phase10WitnessObservation::ParticleVelocity { .. } => "particle_velocity",
        Phase10WitnessObservation::Scalar { .. } => "scalar",
        Phase10WitnessObservation::Count { .. } => "count",
        Phase10WitnessObservation::Occurrence { .. } => "occurrence",
        Phase10WitnessObservation::Topology { .. } => "topology",
    }
}
