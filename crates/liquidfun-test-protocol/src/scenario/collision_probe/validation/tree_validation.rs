use super::{
    CollisionProbeDecodeError, CollisionProbeErrorKind, CollisionTreeCommand, RawTreeCommand,
    validate_aabb, validate_vec2, validation,
};

pub(super) fn validate_tree_command(
    raw: RawTreeCommand,
) -> Result<CollisionTreeCommand, CollisionProbeDecodeError> {
    let command = match raw {
        RawTreeCommand::Create {
            payload_id,
            lower,
            upper,
        } => {
            validate_aabb(lower, upper)?;
            CollisionTreeCommand::Create {
                payload_id,
                lower,
                upper,
            }
        }
        RawTreeCommand::Move {
            payload_id,
            lower,
            upper,
            displacement,
        } => {
            validate_aabb(lower, upper)?;
            validate_vec2(displacement)?;
            CollisionTreeCommand::Move {
                payload_id,
                lower,
                upper,
                displacement,
            }
        }
        RawTreeCommand::Touch { payload_id } => CollisionTreeCommand::Touch { payload_id },
        RawTreeCommand::Destroy { payload_id } => CollisionTreeCommand::Destroy { payload_id },
        RawTreeCommand::Query { lower, upper } => {
            validate_aabb(lower, upper)?;
            CollisionTreeCommand::Query { lower, upper }
        }
        RawTreeCommand::Ray {
            start,
            end,
            max_fraction_bits,
        } => {
            validate_vec2(start)?;
            validate_vec2(end)?;
            let fraction = max_fraction_bits.to_f32();
            if !(0.0..=1.0).contains(&fraction) {
                return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
            }
            CollisionTreeCommand::Ray {
                start,
                end,
                max_fraction_bits,
            }
        }
        RawTreeCommand::Refilter {
            payload_id,
            category_bits,
            mask_bits,
            group_index,
        } => CollisionTreeCommand::Refilter {
            payload_id,
            category_bits,
            mask_bits,
            group_index,
        },
        RawTreeCommand::UpdatePairs => CollisionTreeCommand::UpdatePairs,
        RawTreeCommand::Metrics => CollisionTreeCommand::Metrics,
    };
    Ok(command)
}
