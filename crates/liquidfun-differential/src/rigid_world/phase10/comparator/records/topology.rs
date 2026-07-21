//! Pair, triad, and contact snapshot comparison.

use liquidfun_test_protocol::{
    Phase10BodyContact, Phase10PairSnapshot, Phase10ParticleContact, Phase10TriadSnapshot,
};

use super::check;
use crate::rigid_world::phase10::comparator::{
    Phase10ComparatorError, Phase10Mismatch, numeric, numeric_vec,
};

pub(crate) fn compare_pair(
    scenario: &str,
    index: usize,
    expected: &Phase10PairSnapshot,
    actual: &Phase10PairSnapshot,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    let entity = format!("pair:{}:{}", expected.particle_a_id, expected.particle_b_id);
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.pair.ordinal",
        expected.ordinal,
        actual.ordinal
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.pair.identity",
        (&expected.particle_a_id, &expected.particle_b_id),
        (&actual.particle_a_id, &actual.particle_b_id)
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.pair.flags",
        expected.flags_bits,
        actual.flags_bits
    );
    numeric(
        scenario,
        &entity,
        index,
        "phase10.pair.strength",
        expected.strength_bits,
        actual.strength_bits,
    )?
    .map_or_else(
        || {
            numeric(
                scenario,
                &entity,
                index,
                "phase10.pair.distance",
                expected.distance_bits,
                actual.distance_bits,
            )
        },
        |found| Ok(Some(found)),
    )
}

pub(crate) fn compare_triad(
    scenario: &str,
    index: usize,
    expected: &Phase10TriadSnapshot,
    actual: &Phase10TriadSnapshot,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    let entity = format!(
        "triad:{}:{}:{}",
        expected.particle_a_id, expected.particle_b_id, expected.particle_c_id
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.triad.ordinal",
        expected.ordinal,
        actual.ordinal
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.triad.identity",
        (
            &expected.particle_a_id,
            &expected.particle_b_id,
            &expected.particle_c_id
        ),
        (
            &actual.particle_a_id,
            &actual.particle_b_id,
            &actual.particle_c_id
        )
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.triad.flags",
        expected.flags_bits,
        actual.flags_bits
    );
    if let Some(found) = numeric(
        scenario,
        &entity,
        index,
        "phase10.triad.strength",
        expected.strength_bits,
        actual.strength_bits,
    )? {
        return Ok(Some(found));
    }
    for (left, right) in [
        (expected.pa, actual.pa),
        (expected.pb, actual.pb),
        (expected.pc, actual.pc),
    ] {
        if let Some(found) = numeric_vec(
            scenario,
            &entity,
            index,
            "phase10.triad.offset",
            left,
            right,
        )? {
            return Ok(Some(found));
        }
    }
    for (left, right) in [
        (expected.ka_bits, actual.ka_bits),
        (expected.kb_bits, actual.kb_bits),
        (expected.kc_bits, actual.kc_bits),
        (expected.s_bits, actual.s_bits),
    ] {
        if let Some(found) = numeric(
            scenario,
            &entity,
            index,
            "phase10.triad.coefficient",
            left,
            right,
        )? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

pub(crate) fn compare_particle_contact(
    scenario: &str,
    index: usize,
    expected: &Phase10ParticleContact,
    actual: &Phase10ParticleContact,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    let entity = format!(
        "particle-contact:{}:{}",
        expected.particle_a_id, expected.particle_b_id
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.contact.ordinal",
        expected.ordinal,
        actual.ordinal
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.contact.identity",
        (
            &expected.system_id,
            &expected.particle_a_id,
            &expected.particle_b_id
        ),
        (
            &actual.system_id,
            &actual.particle_a_id,
            &actual.particle_b_id
        )
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.contact.flags",
        expected.flags_bits,
        actual.flags_bits
    );
    numeric(
        scenario,
        &entity,
        index,
        "phase10.contact.weight",
        expected.weight_bits,
        actual.weight_bits,
    )?
    .map_or_else(
        || {
            numeric_vec(
                scenario,
                &entity,
                index,
                "phase10.contact.normal",
                expected.normal,
                actual.normal,
            )
        },
        |found| Ok(Some(found)),
    )
}

pub(crate) fn compare_body_contact(
    scenario: &str,
    index: usize,
    expected: &Phase10BodyContact,
    actual: &Phase10BodyContact,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    let entity = format!("body-contact:{}:{}", expected.particle_id, expected.body_id);
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.contact.ordinal",
        expected.ordinal,
        actual.ordinal
    );
    check!(
        scenario,
        "state",
        &entity,
        index,
        "phase10.contact.identity",
        (
            &expected.system_id,
            &expected.particle_id,
            &expected.body_id,
            &expected.fixture_id
        ),
        (
            &actual.system_id,
            &actual.particle_id,
            &actual.body_id,
            &actual.fixture_id
        )
    );
    numeric(
        scenario,
        &entity,
        index,
        "phase10.contact.weight",
        expected.weight_bits,
        actual.weight_bits,
    )?
    .or(numeric_vec(
        scenario,
        &entity,
        index,
        "phase10.contact.normal",
        expected.normal,
        actual.normal,
    )?)
    .map_or_else(
        || {
            numeric(
                scenario,
                &entity,
                index,
                "phase10.body_contact.mass",
                expected.mass_bits,
                actual.mass_bits,
            )
        },
        |found| Ok(Some(found)),
    )
}
