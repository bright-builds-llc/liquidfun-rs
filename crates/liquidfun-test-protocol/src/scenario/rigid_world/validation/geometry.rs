use super::MAXIMUM_POLYGON_VERTICES;
use crate::{FloatBits, TransformBits, Vec2Bits};

use crate::scenario::rigid_world::{
    RigidFixtureShape, RigidWorldDecodeError, RigidWorldErrorKind, validation,
};

pub(super) fn validate_shape(shape: &RigidFixtureShape) -> Result<(), RigidWorldDecodeError> {
    match shape {
        RigidFixtureShape::Circle {
            center,
            radius_bits,
        } => {
            validate_vec2(*center)?;
            validate_positive(*radius_bits)
        }
        RigidFixtureShape::Polygon { vertices } => {
            if !(3..=MAXIMUM_POLYGON_VERTICES).contains(&vertices.len()) {
                return Err(validation(RigidWorldErrorKind::InvalidGeometry));
            }
            for vertex in vertices.iter().copied() {
                validate_vec2(vertex)?;
            }
            if vertices.windows(2).any(|pair| pair.first() == pair.get(1)) {
                return Err(validation(RigidWorldErrorKind::InvalidGeometry));
            }
            Ok(())
        }
    }
}

pub(super) fn validate_transform(transform: TransformBits) -> Result<(), RigidWorldDecodeError> {
    validate_vec2(transform.position)?;
    validate_finite(transform.angle_bits)
}

pub(super) fn validate_vec2(value: Vec2Bits) -> Result<(), RigidWorldDecodeError> {
    validate_finite(value.x_bits)?;
    validate_finite(value.y_bits)
}

pub(super) fn validate_nonnegative(value: FloatBits) -> Result<(), RigidWorldDecodeError> {
    validate_finite(value)?;
    if value.to_f32() < 0.0 {
        return Err(validation(RigidWorldErrorKind::InvalidMaterial));
    }
    Ok(())
}

pub(super) fn validate_positive(value: FloatBits) -> Result<(), RigidWorldDecodeError> {
    validate_finite(value)?;
    if value.to_f32() <= 0.0 {
        return Err(validation(RigidWorldErrorKind::InvalidGeometry));
    }
    Ok(())
}

fn validate_finite(value: FloatBits) -> Result<(), RigidWorldDecodeError> {
    if !value.to_f32().is_finite() {
        return Err(validation(RigidWorldErrorKind::InvalidGeometry));
    }
    Ok(())
}
