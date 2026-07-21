//! Group and particle snapshot comparison.

use liquidfun_test_protocol::{FloatBits, Phase10GroupSnapshot, Phase10ParticleSnapshot};

use super::check;
use crate::rigid_world::phase10::comparator::{
    Phase10ComparatorError, Phase10Mismatch, mismatch_if, numeric, numeric_transform, numeric_vec,
};

pub(crate) fn compare_group(
    scenario: &str,
    index: usize,
    expected: &Phase10GroupSnapshot,
    actual: &Phase10GroupSnapshot,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    let entity = format!("group:{}", expected.group_id);
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.group.ordinal",
        expected.ordinal,
        actual.ordinal
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.group.identity",
        (&expected.group_id, &expected.system_id),
        (&actual.group_id, &actual.system_id)
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.group.membership",
        expected.member_ids,
        actual.member_ids
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.group.flags",
        expected.group_flags_bits,
        actual.group_flags_bits
    );
    numeric_transform(
        scenario,
        &entity,
        index,
        "phase10.group.transform",
        expected.transform,
        actual.transform,
    )?
    .or(numeric_vec(
        scenario,
        &entity,
        index,
        "phase10.group.center",
        expected.center,
        actual.center,
    )?)
    .or(numeric_vec(
        scenario,
        &entity,
        index,
        "phase10.group.linear_velocity",
        expected.linear_velocity,
        actual.linear_velocity,
    )?)
    .or(numeric(
        scenario,
        &entity,
        index,
        "phase10.group.angular_velocity",
        expected.angular_velocity_bits,
        actual.angular_velocity_bits,
    )?)
    .or(numeric(
        scenario,
        &entity,
        index,
        "phase10.group.mass",
        expected.mass_bits,
        actual.mass_bits,
    )?)
    .or(numeric(
        scenario,
        &entity,
        index,
        "phase10.group.inertia",
        expected.inertia_bits,
        actual.inertia_bits,
    )?)
    .map_or_else(
        || {
            compare_optional_bits(
                scenario,
                &entity,
                index,
                expected.maybe_depths_bits.as_deref(),
                actual.maybe_depths_bits.as_deref(),
            )
        },
        |found| Ok(Some(found)),
    )
}

pub(crate) fn compare_particle(
    scenario: &str,
    index: usize,
    expected: &Phase10ParticleSnapshot,
    actual: &Phase10ParticleSnapshot,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    let entity = format!("particle:{}", expected.particle_id);
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.particle.identity",
        (
            &expected.particle_id,
            &expected.system_id,
            &expected.group_id
        ),
        (&actual.particle_id, &actual.system_id, &actual.group_id)
    );
    if let Some(found) = numeric_vec(
        scenario,
        &entity,
        index,
        "phase10.particle.position",
        expected.position,
        actual.position,
    )?
    .or(numeric_vec(
        scenario,
        &entity,
        index,
        "phase10.particle.velocity",
        expected.velocity,
        actual.velocity,
    )?) {
        return Ok(Some(found));
    }
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.particle.flags",
        expected.flags_bits,
        actual.flags_bits
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.particle.color",
        expected.color,
        actual.color
    );
    numeric(
        scenario,
        &entity,
        index,
        "phase10.particle.weight",
        expected.weight_bits,
        actual.weight_bits,
    )
}

fn compare_optional_bits(
    scenario: &str,
    entity: &str,
    index: usize,
    expected: Option<&[FloatBits]>,
    actual: Option<&[FloatBits]>,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    let (Some(expected), Some(actual)) = (expected, actual) else {
        return Ok(mismatch_if(
            scenario,
            "state",
            entity,
            index,
            "phase10.group.depth",
            &expected.map(<[FloatBits]>::len),
            &actual.map(<[FloatBits]>::len),
        ));
    };
    if let Some(found) = mismatch_if(
        scenario,
        "state",
        entity,
        index,
        "phase10.group.depth",
        &expected.len(),
        &actual.len(),
    ) {
        return Ok(Some(found));
    }
    for (offset, (left, right)) in expected.iter().zip(actual).enumerate() {
        if let Some(found) = numeric(
            scenario,
            entity,
            offset,
            "phase10.group.depth",
            *left,
            *right,
        )? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}
