use super::{MAX_UPSTREAM_COUNT, ParticleCapacity, ParticleDefError, ParticleSystemDefError, Vec2};

pub(super) fn validate_positive(
    value: f32,
    non_finite: ParticleSystemDefError,
    non_positive: ParticleSystemDefError,
) -> Result<(), ParticleSystemDefError> {
    if !value.is_finite() {
        return Err(non_finite);
    }
    if value <= 0.0 {
        return Err(non_positive);
    }
    Ok(())
}

pub(super) fn validate_non_negative(
    value: f32,
    non_finite: ParticleSystemDefError,
    negative: ParticleSystemDefError,
) -> Result<(), ParticleSystemDefError> {
    if !value.is_finite() {
        return Err(non_finite);
    }
    if value < 0.0 {
        return Err(negative);
    }
    Ok(())
}

pub(super) fn validate_capacity_range(count: usize) -> Result<(), ParticleSystemDefError> {
    if count > MAX_UPSTREAM_COUNT {
        return Err(ParticleSystemDefError::CapacityOutOfRange);
    }
    Ok(())
}

pub(super) fn validate_maximum_capacity(
    maybe_maximum: Option<usize>,
    capacity: ParticleCapacity,
) -> Result<(), ParticleSystemDefError> {
    let (Some(maximum), Some(fixed_capacity)) = (maybe_maximum, capacity.maybe_fixed_limit())
    else {
        return Ok(());
    };
    if maximum > fixed_capacity {
        return Err(ParticleSystemDefError::MaximumExceedsFixedCapacity {
            maximum,
            capacity: fixed_capacity,
        });
    }
    Ok(())
}

pub(super) fn validate_vector(
    value: Vec2,
    invalid_x: ParticleDefError,
    invalid_y: ParticleDefError,
) -> Result<(), ParticleDefError> {
    if !value.x.is_finite() {
        return Err(invalid_x);
    }
    if !value.y.is_finite() {
        return Err(invalid_y);
    }
    Ok(())
}
