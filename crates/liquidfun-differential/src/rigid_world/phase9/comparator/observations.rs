//! Typed Phase 9 observation comparators.

use super::{
    FloatBits, Location, PHASE9_BODY_MASS_ABSOLUTE, PHASE9_RAY_FRACTION_ABSOLUTE,
    Phase9BodyContactObservation, Phase9ComparatorError, Phase9Mismatch, Phase9Occurrence,
    Phase9ParticleContactObservation, Phase9ParticleSnapshot, Phase9StatisticsObservation,
    compare_absolute_relative, compare_dimensioned, compare_vec_exact, compare_vec_ulps, exact,
};

pub(super) fn compare_particle(
    location: Location,
    expected: &Phase9ParticleSnapshot,
    actual: &Phase9ParticleSnapshot,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    if let Some(found) = exact(
        location,
        "particle.storage.identity",
        &expected.particle_id,
        &actual.particle_id,
    )
    .or_else(|| {
        exact(
            location,
            "particle.storage.identity",
            &expected.system_id,
            &actual.system_id,
        )
    }) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_vec_ulps(
        location,
        "particle.position",
        expected.position,
        actual.position,
    )?
    .or(compare_vec_ulps(
        location,
        "particle.velocity",
        expected.velocity,
        actual.velocity,
    )?) {
        return Ok(Some(found));
    }
    if let Some(found) = exact(
        location,
        "particle.configuration.bits",
        &expected.flags_bits,
        &actual.flags_bits,
    )
    .or_else(|| {
        exact(
            location,
            "particle.configuration.bits",
            &expected.color,
            &actual.color,
        )
    }) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_absolute_relative(
        location,
        "particle.contact.weight",
        expected.weight_bits,
        actual.weight_bits,
    )? {
        return Ok(Some(found));
    }
    Ok(compare_vec_exact(
        location,
        "particle.force.range",
        expected.force,
        actual.force,
    )
    .or_else(|| {
        exact(
            location,
            "particle.zombie.lifecycle",
            &expected.pending_destruction,
            &actual.pending_destruction,
        )
    }))
}

pub(super) fn compare_occurrence(
    location: Location,
    expected: &Phase9Occurrence,
    actual: &Phase9Occurrence,
) -> Option<Phase9Mismatch> {
    exact(
        location,
        "particle.listener.occurrence",
        &expected.ordinal,
        &actual.ordinal,
    )
    .or_else(|| {
        exact(
            location,
            "particle.filter.decision",
            &expected.kind,
            &actual.kind,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.storage.identity",
            &expected.system_id,
            &actual.system_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.contact.identity",
            &expected.maybe_particle_id,
            &actual.maybe_particle_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.contact.identity",
            &expected.maybe_other_particle_id,
            &actual.maybe_other_particle_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.coupling.identity",
            &expected.maybe_fixture_id,
            &actual.maybe_fixture_id,
        )
    })
}

pub(super) fn compare_particle_contact(
    location: Location,
    expected: &Phase9ParticleContactObservation,
    actual: &Phase9ParticleContactObservation,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    if let Some(found) = exact(
        location,
        "particle.contact.identity",
        &expected.system_id,
        &actual.system_id,
    )
    .or_else(|| {
        exact(
            location,
            "particle.contact.identity",
            &expected.particle_a_id,
            &actual.particle_a_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.contact.identity",
            &expected.particle_b_id,
            &actual.particle_b_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.strict_contact.branch",
            &expected.flags_bits,
            &actual.flags_bits,
        )
    }) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_absolute_relative(
        location,
        "particle.contact.weight",
        expected.weight_bits,
        actual.weight_bits,
    )? {
        return Ok(Some(found));
    }
    compare_vec_ulps(
        location,
        "particle.contact.normal",
        expected.normal,
        actual.normal,
    )
}

pub(super) fn compare_body_contact(
    location: Location,
    expected: &Phase9BodyContactObservation,
    actual: &Phase9BodyContactObservation,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    if let Some(found) = exact(
        location,
        "particle.coupling.identity",
        &expected.system_id,
        &actual.system_id,
    )
    .or_else(|| {
        exact(
            location,
            "particle.coupling.identity",
            &expected.particle_id,
            &actual.particle_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.coupling.identity",
            &expected.body_id,
            &actual.body_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.coupling.identity",
            &expected.fixture_id,
            &actual.fixture_id,
        )
    }) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_absolute_relative(
        location,
        "particle.contact.weight",
        expected.weight_bits,
        actual.weight_bits,
    )?
    .or(compare_vec_ulps(
        location,
        "particle.contact.normal",
        expected.normal,
        actual.normal,
    )?) {
        return Ok(Some(found));
    }
    compare_dimensioned(
        location,
        "particle.body_contact.mass",
        expected.mass_bits,
        actual.mass_bits,
        PHASE9_BODY_MASS_ABSOLUTE,
    )
}

pub(super) fn compare_statistics(
    location: Location,
    expected: &Phase9StatisticsObservation,
    actual: &Phase9StatisticsObservation,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    if let Some(found) = exact(
        location,
        "particle.storage.identity",
        &expected.maybe_system_id,
        &actual.maybe_system_id,
    )
    .or_else(|| {
        exact(
            location,
            "particle.statistics.counts",
            &expected.system_count,
            &actual.system_count,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.statistics.counts",
            &expected.particle_count,
            &actual.particle_count,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.statistics.counts",
            &expected.pending_particle_count,
            &actual.pending_particle_count,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.statistics.counts",
            &expected.particle_contact_count,
            &actual.particle_contact_count,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.statistics.counts",
            &expected.body_contact_count,
            &actual.body_contact_count,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.lifetime.order",
            &expected.stuck_particle_ids,
            &actual.stuck_particle_ids,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.capacity.mode",
            &expected.declared_capacity,
            &actual.declared_capacity,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.capacity.mode",
            &expected.effective_capacity,
            &actual.effective_capacity,
        )
    }) {
        return Ok(Some(found));
    }
    compare_absolute_relative(
        location,
        "particle.statistics.collision_energy",
        expected.collision_energy_bits,
        actual.collision_energy_bits,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "ray structure is compared as one parallel record"
)]
pub(super) fn compare_ray(
    location: Location,
    expected_terminated: bool,
    expected_particles: &[liquidfun_test_protocol::ScenarioId],
    expected_fractions: &[FloatBits],
    actual_terminated: bool,
    actual_particles: &[liquidfun_test_protocol::ScenarioId],
    actual_fractions: &[FloatBits],
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    if expected_particles.len() != expected_fractions.len()
        || actual_particles.len() != actual_fractions.len()
    {
        return Err(Phase9ComparatorError::Structure {
            path: "particle.ray.fraction",
            expected: format!(
                "{} ids/{} fractions",
                expected_particles.len(),
                expected_fractions.len()
            )
            .into(),
            actual: format!(
                "{} ids/{} fractions",
                actual_particles.len(),
                actual_fractions.len()
            )
            .into(),
        });
    }
    if let Some(found) = exact(
        location,
        "particle.query.culling",
        &expected_terminated,
        &actual_terminated,
    )
    .or_else(|| {
        exact(
            location,
            "particle.query.order",
            &expected_particles,
            &actual_particles,
        )
    }) {
        return Ok(Some(found));
    }
    for (expected, actual) in expected_fractions.iter().zip(actual_fractions) {
        if let Some(found) = compare_dimensioned(
            location,
            "particle.ray.fraction",
            *expected,
            *actual,
            PHASE9_RAY_FRACTION_ABSOLUTE,
        )? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}
