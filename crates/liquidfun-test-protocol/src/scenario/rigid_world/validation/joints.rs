use super::{
    HashMap, HashSet, RIGID_WORLD_MAXIMUM_ROPE_VERTICES, RawRopeDeclaration, RigidJointDeclaration,
    RigidJointDefinition, RigidJointKind, RigidJointMutation, RigidRopeDeclaration,
    RigidWorldDecodeError, RigidWorldErrorKind, ScenarioId, validate_finite, validate_nonnegative,
    validate_nonzero_vector, validate_positive, validate_unit_interval, validate_vec2, validation,
};

pub(super) fn validate_joints(
    joints: Vec<RigidJointDeclaration>,
    body_ids: &HashSet<ScenarioId>,
) -> Result<Vec<RigidJointDeclaration>, RigidWorldDecodeError> {
    let mut ids = HashSet::with_capacity(joints.len());
    let mut kinds = HashMap::with_capacity(joints.len());
    let mut endpoints: HashMap<ScenarioId, [ScenarioId; 2]> = HashMap::with_capacity(joints.len());
    for joint in &joints {
        if ids.contains(&joint.joint_id) {
            return Err(validation(RigidWorldErrorKind::DuplicateJointId));
        }
        if joint.body_a_id == joint.body_b_id
            || !body_ids.contains(&joint.body_a_id)
            || !body_ids.contains(&joint.body_b_id)
        {
            return Err(validation(RigidWorldErrorKind::InvalidOwner));
        }
        validate_joint_definition(&joint.definition)?;
        if let RigidJointDefinition::Gear {
            joint_a_id,
            joint_b_id,
            ..
        } = &joint.definition
        {
            let maybe_source_a = endpoints.get(joint_a_id);
            let maybe_source_b = endpoints.get(joint_b_id);
            if joint_a_id == joint_b_id
                || !matches!(
                    kinds.get(joint_a_id),
                    Some(RigidJointKind::Revolute | RigidJointKind::Prismatic)
                )
                || !matches!(
                    kinds.get(joint_b_id),
                    Some(RigidJointKind::Revolute | RigidJointKind::Prismatic)
                )
                || !matches!(
                    (maybe_source_a, maybe_source_b),
                    (Some([_, moving_a]), Some([_, moving_b]))
                        if moving_a != moving_b
                            && moving_a == &joint.body_a_id
                            && moving_b == &joint.body_b_id
                )
            {
                return Err(validation(RigidWorldErrorKind::InvalidJointDependency));
            }
        }
        ids.insert(joint.joint_id.clone());
        kinds.insert(joint.joint_id.clone(), joint.definition.joint_kind());
        endpoints.insert(
            joint.joint_id.clone(),
            [joint.body_a_id.clone(), joint.body_b_id.clone()],
        );
    }
    Ok(joints)
}

pub(super) fn remove_joint_cascade(
    joint_id: &ScenarioId,
    live_joints: &mut HashSet<ScenarioId>,
    gear_dependents: &HashMap<ScenarioId, Vec<ScenarioId>>,
) {
    if let Some(dependents) = gear_dependents.get(joint_id) {
        for dependent in dependents.iter().rev() {
            live_joints.remove(dependent);
        }
    }
    live_joints.remove(joint_id);
}

#[allow(
    clippy::too_many_lines,
    reason = "closed joint definitions are audited exhaustively"
)]
pub(super) fn validate_joint_definition(
    definition: &RigidJointDefinition,
) -> Result<(), RigidWorldDecodeError> {
    let invalid = || validation(RigidWorldErrorKind::InvalidJointDefinition);
    match definition {
        RigidJointDefinition::Revolute {
            local_anchor_a,
            local_anchor_b,
            reference_angle_bits,
            lower_angle_bits,
            upper_angle_bits,
            motor_speed_bits,
            max_motor_torque_bits,
            ..
        } => {
            validate_vec2(*local_anchor_a)?;
            validate_vec2(*local_anchor_b)?;
            for bits in [
                *reference_angle_bits,
                *lower_angle_bits,
                *upper_angle_bits,
                *motor_speed_bits,
            ] {
                validate_finite(bits, RigidWorldErrorKind::InvalidJointDefinition)?;
            }
            validate_nonnegative(*max_motor_torque_bits).map_err(|_| invalid())?;
            if lower_angle_bits.to_f32() > upper_angle_bits.to_f32() {
                return Err(invalid());
            }
        }
        RigidJointDefinition::Prismatic {
            local_anchor_a,
            local_anchor_b,
            local_axis_a,
            reference_angle_bits,
            lower_translation_bits,
            upper_translation_bits,
            motor_speed_bits,
            max_motor_force_bits,
            ..
        } => {
            for vector in [*local_anchor_a, *local_anchor_b, *local_axis_a] {
                validate_vec2(vector)?;
            }
            validate_nonzero_vector(*local_axis_a, RigidWorldErrorKind::InvalidJointDefinition)?;
            for bits in [
                *reference_angle_bits,
                *lower_translation_bits,
                *upper_translation_bits,
                *motor_speed_bits,
            ] {
                validate_finite(bits, RigidWorldErrorKind::InvalidJointDefinition)?;
            }
            validate_nonnegative(*max_motor_force_bits).map_err(|_| invalid())?;
            if lower_translation_bits.to_f32() > upper_translation_bits.to_f32() {
                return Err(invalid());
            }
        }
        RigidJointDefinition::Distance {
            local_anchor_a,
            local_anchor_b,
            length_bits,
            frequency_bits,
            damping_ratio_bits,
        } => {
            validate_vec2(*local_anchor_a)?;
            validate_vec2(*local_anchor_b)?;
            validate_positive(*length_bits).map_err(|_| invalid())?;
            validate_nonnegative(*frequency_bits).map_err(|_| invalid())?;
            validate_unit_interval(*damping_ratio_bits)?;
        }
        RigidJointDefinition::Pulley {
            ground_anchor_a,
            ground_anchor_b,
            local_anchor_a,
            local_anchor_b,
            length_a_bits,
            length_b_bits,
            ratio_bits,
        } => {
            for vector in [
                *ground_anchor_a,
                *ground_anchor_b,
                *local_anchor_a,
                *local_anchor_b,
            ] {
                validate_vec2(vector)?;
            }
            for bits in [*length_a_bits, *length_b_bits, *ratio_bits] {
                validate_positive(bits).map_err(|_| invalid())?;
            }
        }
        RigidJointDefinition::Mouse {
            target,
            max_force_bits,
            frequency_bits,
            damping_ratio_bits,
        } => {
            validate_vec2(*target)?;
            validate_nonnegative(*max_force_bits).map_err(|_| invalid())?;
            validate_nonnegative(*frequency_bits).map_err(|_| invalid())?;
            validate_unit_interval(*damping_ratio_bits)?;
        }
        RigidJointDefinition::Gear { ratio_bits, .. } => {
            validate_finite(*ratio_bits, RigidWorldErrorKind::InvalidJointDefinition)?;
        }
        RigidJointDefinition::Wheel {
            local_anchor_a,
            local_anchor_b,
            local_axis_a,
            motor_speed_bits,
            max_motor_torque_bits,
            frequency_bits,
            damping_ratio_bits,
            ..
        } => {
            for vector in [*local_anchor_a, *local_anchor_b, *local_axis_a] {
                validate_vec2(vector)?;
            }
            validate_nonzero_vector(*local_axis_a, RigidWorldErrorKind::InvalidJointDefinition)?;
            validate_finite(
                *motor_speed_bits,
                RigidWorldErrorKind::InvalidJointDefinition,
            )?;
            validate_nonnegative(*max_motor_torque_bits).map_err(|_| invalid())?;
            validate_nonnegative(*frequency_bits).map_err(|_| invalid())?;
            validate_unit_interval(*damping_ratio_bits)?;
        }
        RigidJointDefinition::Weld {
            local_anchor_a,
            local_anchor_b,
            reference_angle_bits,
            frequency_bits,
            damping_ratio_bits,
        } => {
            validate_vec2(*local_anchor_a)?;
            validate_vec2(*local_anchor_b)?;
            validate_finite(
                *reference_angle_bits,
                RigidWorldErrorKind::InvalidJointDefinition,
            )?;
            validate_nonnegative(*frequency_bits).map_err(|_| invalid())?;
            validate_unit_interval(*damping_ratio_bits)?;
        }
        RigidJointDefinition::Friction {
            local_anchor_a,
            local_anchor_b,
            max_force_bits,
            max_torque_bits,
        } => {
            validate_vec2(*local_anchor_a)?;
            validate_vec2(*local_anchor_b)?;
            validate_nonnegative(*max_force_bits).map_err(|_| invalid())?;
            validate_nonnegative(*max_torque_bits).map_err(|_| invalid())?;
        }
        RigidJointDefinition::Rope {
            local_anchor_a,
            local_anchor_b,
            max_length_bits,
        } => {
            validate_vec2(*local_anchor_a)?;
            validate_vec2(*local_anchor_b)?;
            validate_positive(*max_length_bits).map_err(|_| invalid())?;
        }
        RigidJointDefinition::Motor {
            linear_offset,
            angular_offset_bits,
            max_force_bits,
            max_torque_bits,
            correction_factor_bits,
        } => {
            validate_vec2(*linear_offset)?;
            validate_finite(
                *angular_offset_bits,
                RigidWorldErrorKind::InvalidJointDefinition,
            )?;
            validate_nonnegative(*max_force_bits).map_err(|_| invalid())?;
            validate_nonnegative(*max_torque_bits).map_err(|_| invalid())?;
            validate_unit_interval(*correction_factor_bits)?;
        }
    }
    Ok(())
}

pub(super) fn validate_ropes(
    raw_ropes: Vec<RawRopeDeclaration>,
) -> Result<Vec<RigidRopeDeclaration>, RigidWorldDecodeError> {
    let ropes = raw_ropes
        .into_iter()
        .map(|raw| RigidRopeDeclaration {
            rope_id: raw.rope_id,
            vertices: raw.vertices.into_vec().into_boxed_slice(),
            masses_bits: raw.masses_bits.into_vec().into_boxed_slice(),
            gravity: raw.gravity,
            damping_bits: raw.damping_bits,
            stretch_stiffness_bits: raw.stretch_stiffness_bits,
            bend_stiffness_bits: raw.bend_stiffness_bits,
        })
        .collect::<Vec<_>>();
    let mut ids = HashSet::with_capacity(ropes.len());
    for rope in &ropes {
        if !ids.insert(rope.rope_id.clone()) {
            return Err(validation(RigidWorldErrorKind::DuplicateRopeId));
        }
        if rope.vertices.len() < 3
            || rope.vertices.len() > RIGID_WORLD_MAXIMUM_ROPE_VERTICES
            || rope.vertices.len() != rope.masses_bits.len()
        {
            return Err(validation(RigidWorldErrorKind::InvalidRopeDefinition));
        }
        for vertex in &rope.vertices {
            validate_vec2(*vertex)?;
        }
        for mass in &rope.masses_bits {
            validate_nonnegative(*mass)
                .map_err(|_| validation(RigidWorldErrorKind::InvalidRopeDefinition))?;
        }
        validate_vec2(rope.gravity)?;
        validate_nonnegative(rope.damping_bits)
            .map_err(|_| validation(RigidWorldErrorKind::InvalidRopeDefinition))?;
        validate_unit_interval(rope.stretch_stiffness_bits)?;
        validate_unit_interval(rope.bend_stiffness_bits)?;
    }
    Ok(ropes)
}

pub(super) fn joint_mutation_changes_definition(
    definition: &RigidJointDefinition,
    mutation: RigidJointMutation,
) -> bool {
    if let Some(changed) = limit_or_motor_mutation_changes_definition(definition, mutation) {
        return changed;
    }

    match (definition, mutation) {
        (
            RigidJointDefinition::Distance { length_bits, .. },
            RigidJointMutation::Length {
                length_bits: mutation_bits,
            },
        ) => mutation_bits != *length_bits,
        (
            RigidJointDefinition::Distance { frequency_bits, .. }
            | RigidJointDefinition::Mouse { frequency_bits, .. }
            | RigidJointDefinition::Wheel { frequency_bits, .. }
            | RigidJointDefinition::Weld { frequency_bits, .. },
            RigidJointMutation::Frequency {
                frequency_bits: mutation_bits,
            },
        ) => mutation_bits != *frequency_bits,
        (
            RigidJointDefinition::Distance {
                damping_ratio_bits, ..
            }
            | RigidJointDefinition::Mouse {
                damping_ratio_bits, ..
            }
            | RigidJointDefinition::Wheel {
                damping_ratio_bits, ..
            }
            | RigidJointDefinition::Weld {
                damping_ratio_bits, ..
            },
            RigidJointMutation::DampingRatio {
                damping_ratio_bits: mutation_bits,
            },
        ) => mutation_bits != *damping_ratio_bits,
        (
            RigidJointDefinition::Mouse { target, .. },
            RigidJointMutation::MouseTarget {
                target: mutation_target,
            },
        ) => mutation_target != *target,
        (
            RigidJointDefinition::Mouse { max_force_bits, .. }
            | RigidJointDefinition::Friction { max_force_bits, .. }
            | RigidJointDefinition::Motor { max_force_bits, .. },
            RigidJointMutation::MaxForce { force_bits },
        ) => force_bits != *max_force_bits,
        (
            RigidJointDefinition::Friction {
                max_torque_bits, ..
            }
            | RigidJointDefinition::Motor {
                max_torque_bits, ..
            },
            RigidJointMutation::MaxTorque { torque_bits },
        ) => torque_bits != *max_torque_bits,
        (
            RigidJointDefinition::Gear { ratio_bits, .. },
            RigidJointMutation::GearRatio {
                ratio_bits: mutation_bits,
            },
        ) => mutation_bits != *ratio_bits,
        (
            RigidJointDefinition::Rope {
                max_length_bits, ..
            },
            RigidJointMutation::RopeMaxLength {
                max_length_bits: mutation_bits,
            },
        ) => mutation_bits != *max_length_bits,
        (
            RigidJointDefinition::Motor { linear_offset, .. },
            RigidJointMutation::LinearOffset { offset },
        ) => offset != *linear_offset,
        (
            RigidJointDefinition::Motor {
                angular_offset_bits,
                ..
            },
            RigidJointMutation::AngularOffset { offset_bits },
        ) => offset_bits != *angular_offset_bits,
        (
            RigidJointDefinition::Motor {
                correction_factor_bits,
                ..
            },
            RigidJointMutation::CorrectionFactor { factor_bits },
        ) => factor_bits != *correction_factor_bits,
        _ => false,
    }
}

pub(super) fn limit_or_motor_mutation_changes_definition(
    definition: &RigidJointDefinition,
    mutation: RigidJointMutation,
) -> Option<bool> {
    let changed = match (definition, mutation) {
        (
            RigidJointDefinition::Revolute { limit_enabled, .. }
            | RigidJointDefinition::Prismatic { limit_enabled, .. },
            RigidJointMutation::LimitEnabled { enabled },
        ) => enabled != *limit_enabled,
        (
            RigidJointDefinition::Revolute {
                lower_angle_bits,
                upper_angle_bits,
                ..
            },
            RigidJointMutation::Limits {
                lower_bits,
                upper_bits,
            },
        ) => lower_bits != *lower_angle_bits || upper_bits != *upper_angle_bits,
        (
            RigidJointDefinition::Prismatic {
                lower_translation_bits,
                upper_translation_bits,
                ..
            },
            RigidJointMutation::Limits {
                lower_bits,
                upper_bits,
            },
        ) => lower_bits != *lower_translation_bits || upper_bits != *upper_translation_bits,
        (
            RigidJointDefinition::Revolute { motor_enabled, .. }
            | RigidJointDefinition::Prismatic { motor_enabled, .. }
            | RigidJointDefinition::Wheel { motor_enabled, .. },
            RigidJointMutation::MotorEnabled { enabled },
        ) => enabled != *motor_enabled,
        (
            RigidJointDefinition::Revolute {
                motor_speed_bits, ..
            }
            | RigidJointDefinition::Prismatic {
                motor_speed_bits, ..
            }
            | RigidJointDefinition::Wheel {
                motor_speed_bits, ..
            },
            RigidJointMutation::MotorSpeed { speed_bits },
        ) => speed_bits != *motor_speed_bits,
        (
            RigidJointDefinition::Prismatic {
                max_motor_force_bits,
                ..
            },
            RigidJointMutation::MaxMotorForce { force_bits },
        ) => force_bits != *max_motor_force_bits,
        (
            RigidJointDefinition::Revolute {
                max_motor_torque_bits,
                ..
            }
            | RigidJointDefinition::Wheel {
                max_motor_torque_bits,
                ..
            },
            RigidJointMutation::MaxMotorTorque { torque_bits },
        ) => torque_bits != *max_motor_torque_bits,
        _ => return None,
    };
    Some(changed)
}

pub(super) fn validate_joint_mutation(
    joint_kind: RigidJointKind,
    mutation: RigidJointMutation,
) -> Result<(), RigidWorldDecodeError> {
    if !joint_mutation_is_supported(joint_kind, mutation) {
        return Err(validation(RigidWorldErrorKind::InvalidJointDefinition));
    }

    match mutation {
        RigidJointMutation::LimitEnabled { .. } | RigidJointMutation::MotorEnabled { .. } => {}
        RigidJointMutation::Limits {
            lower_bits,
            upper_bits,
        } => {
            validate_finite(lower_bits, RigidWorldErrorKind::InvalidJointDefinition)?;
            validate_finite(upper_bits, RigidWorldErrorKind::InvalidJointDefinition)?;
            if lower_bits.to_f32() > upper_bits.to_f32() {
                return Err(validation(RigidWorldErrorKind::InvalidJointDefinition));
            }
        }
        RigidJointMutation::MotorSpeed { speed_bits }
        | RigidJointMutation::AngularOffset {
            offset_bits: speed_bits,
        }
        | RigidJointMutation::GearRatio {
            ratio_bits: speed_bits,
        } => validate_finite(speed_bits, RigidWorldErrorKind::InvalidJointDefinition)?,
        RigidJointMutation::MaxMotorForce { force_bits }
        | RigidJointMutation::MaxForce { force_bits }
        | RigidJointMutation::MaxMotorTorque {
            torque_bits: force_bits,
        }
        | RigidJointMutation::MaxTorque {
            torque_bits: force_bits,
        }
        | RigidJointMutation::Frequency {
            frequency_bits: force_bits,
        } => validate_nonnegative(force_bits)
            .map_err(|_| validation(RigidWorldErrorKind::InvalidJointDefinition))?,
        RigidJointMutation::Length { length_bits }
        | RigidJointMutation::RopeMaxLength {
            max_length_bits: length_bits,
        } => validate_positive(length_bits)
            .map_err(|_| validation(RigidWorldErrorKind::InvalidJointDefinition))?,
        RigidJointMutation::DampingRatio { damping_ratio_bits } => {
            validate_unit_interval(damping_ratio_bits)?;
        }
        RigidJointMutation::MouseTarget { target }
        | RigidJointMutation::LinearOffset { offset: target } => validate_vec2(target)?,
        RigidJointMutation::CorrectionFactor { factor_bits } => {
            validate_unit_interval(factor_bits)?;
        }
    }
    Ok(())
}

pub(super) fn joint_mutation_is_supported(
    joint_kind: RigidJointKind,
    mutation: RigidJointMutation,
) -> bool {
    match joint_kind {
        RigidJointKind::Revolute => matches!(
            mutation,
            RigidJointMutation::LimitEnabled { .. }
                | RigidJointMutation::Limits { .. }
                | RigidJointMutation::MotorEnabled { .. }
                | RigidJointMutation::MotorSpeed { .. }
                | RigidJointMutation::MaxMotorTorque { .. }
        ),
        RigidJointKind::Prismatic => matches!(
            mutation,
            RigidJointMutation::LimitEnabled { .. }
                | RigidJointMutation::Limits { .. }
                | RigidJointMutation::MotorEnabled { .. }
                | RigidJointMutation::MotorSpeed { .. }
                | RigidJointMutation::MaxMotorForce { .. }
        ),
        RigidJointKind::Distance => matches!(
            mutation,
            RigidJointMutation::Length { .. }
                | RigidJointMutation::Frequency { .. }
                | RigidJointMutation::DampingRatio { .. }
        ),
        RigidJointKind::Pulley => false,
        RigidJointKind::Mouse => matches!(
            mutation,
            RigidJointMutation::MouseTarget { .. }
                | RigidJointMutation::MaxForce { .. }
                | RigidJointMutation::Frequency { .. }
                | RigidJointMutation::DampingRatio { .. }
        ),
        RigidJointKind::Gear => matches!(mutation, RigidJointMutation::GearRatio { .. }),
        RigidJointKind::Wheel => matches!(
            mutation,
            RigidJointMutation::MotorEnabled { .. }
                | RigidJointMutation::MotorSpeed { .. }
                | RigidJointMutation::MaxMotorTorque { .. }
                | RigidJointMutation::Frequency { .. }
                | RigidJointMutation::DampingRatio { .. }
        ),
        RigidJointKind::Weld => matches!(
            mutation,
            RigidJointMutation::Frequency { .. } | RigidJointMutation::DampingRatio { .. }
        ),
        RigidJointKind::Friction => matches!(
            mutation,
            RigidJointMutation::MaxForce { .. } | RigidJointMutation::MaxTorque { .. }
        ),
        RigidJointKind::Rope => {
            matches!(mutation, RigidJointMutation::RopeMaxLength { .. })
        }
        RigidJointKind::Motor => matches!(
            mutation,
            RigidJointMutation::LinearOffset { .. }
                | RigidJointMutation::AngularOffset { .. }
                | RigidJointMutation::MaxForce { .. }
                | RigidJointMutation::MaxTorque { .. }
                | RigidJointMutation::CorrectionFactor { .. }
        ),
    }
}
