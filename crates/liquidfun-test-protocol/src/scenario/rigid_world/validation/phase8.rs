use std::collections::{HashMap, HashSet};

use super::{
    RigidBodyDeclaration, RigidBodyKind, RigidFixtureDeclaration, RigidFixtureShape,
    RigidJointDeclaration, RigidJointDefinition, RigidJointKind, RigidJointMutation,
    RigidRopeDeclaration, RigidWorldAction, RigidWorldActionRecord, RigidWorldDecodeError,
    RigidWorldErrorKind, RigidWorldWitnessFamily, validation,
};

pub(super) fn validate_phase8_behavior(
    family: RigidWorldWitnessFamily,
    bodies: &[RigidBodyDeclaration],
    fixtures: &[RigidFixtureDeclaration],
    joints: &[RigidJointDeclaration],
    ropes: &[RigidRopeDeclaration],
    actions: &[RigidWorldActionRecord],
) -> Result<(), RigidWorldDecodeError> {
    use RigidWorldWitnessFamily as Family;

    match family {
        Family::JointDefinitionsAndMutations
        | Family::RevolutePrismaticLimitsAndMotors
        | Family::DistancePulleyMouseConstraints
        | Family::WheelWeldFrictionRopeMotorConstraints
        | Family::GearDependenciesAndFourBodySolver
        | Family::MixedJointIslandOrderAndCollisionSuppression => {
            validate_step_bearing_joint_timeline(family, joints, actions)?;
        }
        Family::StandaloneRopeEvolution => {
            let has_positive_step = actions.iter().any(|record| {
                matches!(
                    record.action(),
                    RigidWorldAction::StepRope { timestep_bits, .. }
                        if timestep_bits.to_f32() > 0.0
                )
            });
            let has_inspection_after_step = action_ordered_pair(
                actions,
                |action| matches!(action, RigidWorldAction::StepRope { .. }),
                |action| matches!(action, RigidWorldAction::InspectRope { .. }),
            );
            if ropes.len() != 1 || !has_positive_step || !has_inspection_after_step {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        Family::ContactFilterListenerAndPreSolveTiming => {
            validate_callback_timeline(bodies, fixtures, actions)?;
        }
        Family::DestructionListenerAndDependencyCascades => {
            validate_destruction_timeline(bodies, fixtures, joints, actions)?;
        }
        Family::DiagnosticReconstructionAndDumpOrder => {
            if !action_ordered_pair(
                actions,
                |action| matches!(action, RigidWorldAction::RequestReconstruction),
                |action| matches!(action, RigidWorldAction::RequestDiagnostics),
            ) {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_step_bearing_joint_timeline(
    family: RigidWorldWitnessFamily,
    joints: &[RigidJointDeclaration],
    actions: &[RigidWorldActionRecord],
) -> Result<(), RigidWorldDecodeError> {
    let step_positions = actions
        .iter()
        .enumerate()
        .filter_map(|(index, record)| is_positive_world_step(record.action()).then_some(index))
        .collect::<Vec<_>>();
    let Some(&first_step) = step_positions.first() else {
        return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
    };
    let has_nonzero_state_before_step = actions[..first_step]
        .iter()
        .any(|record| action_introduces_nonzero_state(record.action()));
    if !has_nonzero_state_before_step {
        return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
    }

    let created = actions[..first_step]
        .iter()
        .filter_map(|record| match record.action() {
            RigidWorldAction::CreateJoint { joint_id } => Some(joint_id),
            _ => None,
        })
        .collect::<HashSet<_>>();
    let inspected = actions[first_step + 1..]
        .iter()
        .filter_map(|record| match record.action() {
            RigidWorldAction::InspectJoint { joint_id } => Some(joint_id),
            _ => None,
        })
        .collect::<HashSet<_>>();
    if joints
        .iter()
        .any(|joint| !created.contains(&joint.joint_id) || !inspected.contains(&joint.joint_id))
    {
        return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
    }

    use RigidWorldWitnessFamily as Family;
    let valid_family_shape = match family {
        Family::JointDefinitionsAndMutations => {
            let kinds = joints
                .iter()
                .map(|joint| joint.definition.joint_kind())
                .collect::<HashSet<_>>();
            RigidJointKind::ALL.iter().all(|kind| kinds.contains(kind)) && step_positions.len() >= 2
        }
        Family::RevolutePrismaticLimitsAndMotors => {
            has_joint_kinds(
                joints,
                &[RigidJointKind::Revolute, RigidJointKind::Prismatic],
            ) && has_equal_limit(joints)
                && has_ranged_motor_limit(joints, RigidJointKind::Revolute)
                && has_ranged_motor_limit(joints, RigidJointKind::Prismatic)
        }
        Family::DistancePulleyMouseConstraints => {
            has_joint_kinds(
                joints,
                &[
                    RigidJointKind::Distance,
                    RigidJointKind::Pulley,
                    RigidJointKind::Mouse,
                ],
            ) && distance_modes_are_complete(joints)
        }
        Family::WheelWeldFrictionRopeMotorConstraints => {
            has_joint_kinds(
                joints,
                &[
                    RigidJointKind::Wheel,
                    RigidJointKind::Weld,
                    RigidJointKind::Friction,
                    RigidJointKind::Rope,
                    RigidJointKind::Motor,
                ],
            ) && weld_modes_are_complete(joints)
                && rope_modes_are_complete(joints)
        }
        Family::GearDependenciesAndFourBodySolver => gear_modes_are_complete(joints),
        Family::MixedJointIslandOrderAndCollisionSuppression => {
            joints.iter().any(|joint| !joint.collide_connected) && step_positions.len() >= 2
        }
        _ => false,
    };
    if !valid_family_shape {
        return Err(validation(RigidWorldErrorKind::InvalidJointDefinition));
    }
    Ok(())
}

fn is_positive_world_step(action: &RigidWorldAction) -> bool {
    match action {
        RigidWorldAction::Step { timestep_bits, .. }
        | RigidWorldAction::ConfiguredStep { timestep_bits, .. } => timestep_bits.to_f32() > 0.0,
        _ => false,
    }
}

fn action_introduces_nonzero_state(action: &RigidWorldAction) -> bool {
    match action {
        RigidWorldAction::MutateJoint { mutation, .. } => mutation_has_nonzero_value(*mutation),
        RigidWorldAction::SetLinearVelocity { velocity, .. } => {
            velocity.x_bits.to_f32() != 0.0 || velocity.y_bits.to_f32() != 0.0
        }
        RigidWorldAction::SetAngularVelocity {
            angular_velocity_bits,
            ..
        } => angular_velocity_bits.to_f32() != 0.0,
        _ => false,
    }
}

fn mutation_has_nonzero_value(mutation: RigidJointMutation) -> bool {
    match mutation {
        RigidJointMutation::LimitEnabled { enabled }
        | RigidJointMutation::MotorEnabled { enabled } => enabled,
        RigidJointMutation::Limits {
            lower_bits,
            upper_bits,
        } => lower_bits.to_f32() != 0.0 || upper_bits.to_f32() != 0.0,
        RigidJointMutation::MotorSpeed { speed_bits }
        | RigidJointMutation::MaxMotorForce {
            force_bits: speed_bits,
        }
        | RigidJointMutation::MaxMotorTorque {
            torque_bits: speed_bits,
        }
        | RigidJointMutation::Length {
            length_bits: speed_bits,
        }
        | RigidJointMutation::Frequency {
            frequency_bits: speed_bits,
        }
        | RigidJointMutation::DampingRatio {
            damping_ratio_bits: speed_bits,
        }
        | RigidJointMutation::MaxForce {
            force_bits: speed_bits,
        }
        | RigidJointMutation::MaxTorque {
            torque_bits: speed_bits,
        }
        | RigidJointMutation::GearRatio {
            ratio_bits: speed_bits,
        }
        | RigidJointMutation::RopeMaxLength {
            max_length_bits: speed_bits,
        }
        | RigidJointMutation::AngularOffset {
            offset_bits: speed_bits,
        }
        | RigidJointMutation::CorrectionFactor {
            factor_bits: speed_bits,
        } => speed_bits.to_f32() != 0.0,
        RigidJointMutation::MouseTarget { target }
        | RigidJointMutation::LinearOffset { offset: target } => {
            target.x_bits.to_f32() != 0.0 || target.y_bits.to_f32() != 0.0
        }
    }
}

fn has_joint_kinds(joints: &[RigidJointDeclaration], required: &[RigidJointKind]) -> bool {
    required.iter().all(|kind| {
        joints
            .iter()
            .any(|joint| joint.definition.joint_kind() == *kind)
    })
}

fn has_equal_limit(joints: &[RigidJointDeclaration]) -> bool {
    joints.iter().any(|joint| match &joint.definition {
        RigidJointDefinition::Revolute {
            lower_angle_bits,
            upper_angle_bits,
            ..
        }
        | RigidJointDefinition::Prismatic {
            lower_translation_bits: lower_angle_bits,
            upper_translation_bits: upper_angle_bits,
            ..
        } => lower_angle_bits == upper_angle_bits,
        _ => false,
    })
}

fn has_ranged_motor_limit(joints: &[RigidJointDeclaration], kind: RigidJointKind) -> bool {
    joints.iter().any(|joint| match (&joint.definition, kind) {
        (
            RigidJointDefinition::Revolute {
                lower_angle_bits,
                upper_angle_bits,
                limit_enabled,
                motor_enabled,
                ..
            },
            RigidJointKind::Revolute,
        )
        | (
            RigidJointDefinition::Prismatic {
                lower_translation_bits: lower_angle_bits,
                upper_translation_bits: upper_angle_bits,
                limit_enabled,
                motor_enabled,
                ..
            },
            RigidJointKind::Prismatic,
        ) => {
            *limit_enabled
                && *motor_enabled
                && lower_angle_bits.to_f32() < upper_angle_bits.to_f32()
        }
        _ => false,
    })
}

fn distance_modes_are_complete(joints: &[RigidJointDeclaration]) -> bool {
    let mut rigid = false;
    let mut soft = false;
    for joint in joints {
        if let RigidJointDefinition::Distance { frequency_bits, .. } = &joint.definition {
            rigid |= frequency_bits.to_f32() == 0.0;
            soft |= frequency_bits.to_f32() > 0.0;
        }
    }
    rigid && soft
}

fn weld_modes_are_complete(joints: &[RigidJointDeclaration]) -> bool {
    let mut rigid = false;
    let mut soft = false;
    for joint in joints {
        if let RigidJointDefinition::Weld { frequency_bits, .. } = &joint.definition {
            rigid |= frequency_bits.to_f32() == 0.0;
            soft |= frequency_bits.to_f32() > 0.0;
        }
    }
    rigid && soft
}

fn rope_modes_are_complete(joints: &[RigidJointDeclaration]) -> bool {
    let lengths = joints
        .iter()
        .filter_map(|joint| match &joint.definition {
            RigidJointDefinition::Rope {
                max_length_bits, ..
            } => Some(max_length_bits.to_f32()),
            _ => None,
        })
        .collect::<Vec<_>>();
    lengths.len() >= 2
        && lengths
            .iter()
            .any(|left| lengths.iter().any(|right| left < right))
}

fn gear_modes_are_complete(joints: &[RigidJointDeclaration]) -> bool {
    let kinds = joints
        .iter()
        .map(|joint| (joint.joint_id.clone(), joint.definition.joint_kind()))
        .collect::<HashMap<_, _>>();
    let mut combinations = HashSet::new();
    let mut has_negative = false;
    let mut has_zero = false;
    let mut has_positive = false;
    for joint in joints {
        let RigidJointDefinition::Gear {
            joint_a_id,
            joint_b_id,
            ratio_bits,
        } = &joint.definition
        else {
            continue;
        };
        let Some(kind_a) = kinds.get(joint_a_id) else {
            return false;
        };
        let Some(kind_b) = kinds.get(joint_b_id) else {
            return false;
        };
        combinations.insert((*kind_a, *kind_b));
        let ratio = ratio_bits.to_f32();
        has_negative |= ratio < 0.0;
        has_zero |= ratio == 0.0;
        has_positive |= ratio > 0.0;
    }
    combinations
        == HashSet::from([
            (RigidJointKind::Revolute, RigidJointKind::Revolute),
            (RigidJointKind::Revolute, RigidJointKind::Prismatic),
            (RigidJointKind::Prismatic, RigidJointKind::Revolute),
            (RigidJointKind::Prismatic, RigidJointKind::Prismatic),
        ])
        && has_negative
        && has_zero
        && has_positive
}

fn validate_callback_timeline(
    bodies: &[RigidBodyDeclaration],
    fixtures: &[RigidFixtureDeclaration],
    actions: &[RigidWorldActionRecord],
) -> Result<(), RigidWorldDecodeError> {
    if !has_eligible_touching_pair(bodies, fixtures) {
        return Err(validation(RigidWorldErrorKind::InvalidGeometry));
    }
    let filter_values = actions
        .iter()
        .filter_map(|record| match record.action() {
            RigidWorldAction::SetContactFilterDirective { should_collide, .. } => {
                Some(*should_collide)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let pre_solve_values = actions
        .iter()
        .filter_map(|record| match record.action() {
            RigidWorldAction::SetPreSolveDirective { directive, .. } => Some(*directive),
            _ => None,
        })
        .collect::<Vec<_>>();
    let steps = actions
        .iter()
        .filter(|record| is_positive_world_step(record.action()))
        .count();
    let ordered = action_ordered_pair(
        actions,
        |action| {
            matches!(
                action,
                RigidWorldAction::SetContactFilterDirective {
                    should_collide: false,
                    ..
                }
            )
        },
        |action| {
            matches!(
                action,
                RigidWorldAction::Step { .. } | RigidWorldAction::ConfiguredStep { .. }
            )
        },
    ) && action_ordered_pair(
        actions,
        |action| matches!(action, RigidWorldAction::SetPreSolveDirective { .. }),
        |action| matches!(action, RigidWorldAction::InspectBody { .. }),
    );
    if filter_values != [false, true]
        || steps < 3
        || !pre_solve_values.iter().any(|directive| !directive.enabled)
        || !pre_solve_values.iter().any(|directive| {
            directive.enabled
                && directive.maybe_friction_bits.is_some()
                && directive.maybe_restitution_bits.is_some()
                && directive.maybe_tangent_speed_bits.is_some()
        })
        || !ordered
    {
        return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
    }
    Ok(())
}

fn validate_destruction_timeline(
    bodies: &[RigidBodyDeclaration],
    fixtures: &[RigidFixtureDeclaration],
    joints: &[RigidJointDeclaration],
    actions: &[RigidWorldActionRecord],
) -> Result<(), RigidWorldDecodeError> {
    if !has_eligible_touching_pair(bodies, fixtures) || !gear_modes_are_complete_for_cascade(joints)
    {
        return Err(validation(RigidWorldErrorKind::InvalidGeometry));
    }
    let Some(step) = actions
        .iter()
        .position(|record| is_positive_world_step(record.action()))
    else {
        return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
    };
    let has_inspection = actions[step + 1..]
        .iter()
        .any(|record| matches!(record.action(), RigidWorldAction::InspectBody { .. }));
    let has_ordered_destruction = action_ordered_pair(
        actions,
        |action| matches!(action, RigidWorldAction::DestroyJoint { .. }),
        |action| matches!(action, RigidWorldAction::DestroyFixture { .. }),
    ) && action_ordered_pair(
        actions,
        |action| matches!(action, RigidWorldAction::DestroyFixture { .. }),
        |action| matches!(action, RigidWorldAction::DestroyBody { .. }),
    );
    let source_ids = joints
        .iter()
        .filter_map(|joint| match &joint.definition {
            RigidJointDefinition::Gear {
                joint_a_id,
                joint_b_id,
                ..
            } => Some([joint_a_id, joint_b_id]),
            _ => None,
        })
        .flatten()
        .collect::<HashSet<_>>();
    let first_destroyed_joint = actions.iter().find_map(|record| match record.action() {
        RigidWorldAction::DestroyJoint { joint_id } => Some(joint_id),
        _ => None,
    });
    let triggers_dependency_cascade =
        first_destroyed_joint.is_some_and(|joint_id| source_ids.contains(joint_id));
    if !has_inspection || !has_ordered_destruction || !triggers_dependency_cascade {
        return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
    }
    Ok(())
}

fn gear_modes_are_complete_for_cascade(joints: &[RigidJointDeclaration]) -> bool {
    joints
        .iter()
        .any(|joint| matches!(joint.definition, RigidJointDefinition::Gear { .. }))
        && joints
            .iter()
            .filter(|joint| {
                matches!(
                    joint.definition,
                    RigidJointDefinition::Revolute { .. } | RigidJointDefinition::Prismatic { .. }
                )
            })
            .count()
            >= 2
}

fn has_eligible_touching_pair(
    bodies: &[RigidBodyDeclaration],
    fixtures: &[RigidFixtureDeclaration],
) -> bool {
    let body_by_id = bodies
        .iter()
        .map(|body| (&body.body_id, body))
        .collect::<HashMap<_, _>>();
    fixtures.iter().enumerate().any(|(index, left)| {
        fixtures[index + 1..].iter().any(|right| {
            let Some(left_body) = body_by_id.get(&left.owner_body_id) else {
                return false;
            };
            let Some(right_body) = body_by_id.get(&right.owner_body_id) else {
                return false;
            };
            if left.owner_body_id == right.owner_body_id
                || left.sensor
                || right.sensor
                || (!matches!(left_body.body_kind, RigidBodyKind::Dynamic)
                    && !matches!(right_body.body_kind, RigidBodyKind::Dynamic))
            {
                return false;
            }
            let (
                RigidFixtureShape::Circle {
                    center: left_center,
                    radius_bits: left_radius,
                },
                RigidFixtureShape::Circle {
                    center: right_center,
                    radius_bits: right_radius,
                },
            ) = (&left.shape, &right.shape)
            else {
                return false;
            };
            let dx = left_body.transform.position.x_bits.to_f32() + left_center.x_bits.to_f32()
                - right_body.transform.position.x_bits.to_f32()
                - right_center.x_bits.to_f32();
            let dy = left_body.transform.position.y_bits.to_f32() + left_center.y_bits.to_f32()
                - right_body.transform.position.y_bits.to_f32()
                - right_center.y_bits.to_f32();
            let radius = left_radius.to_f32() + right_radius.to_f32();
            dx * dx + dy * dy <= radius * radius
        })
    })
}

fn action_ordered_pair(
    actions: &[RigidWorldActionRecord],
    first: impl Fn(&RigidWorldAction) -> bool,
    second: impl Fn(&RigidWorldAction) -> bool,
) -> bool {
    let Some(first_index) = actions.iter().position(|record| first(record.action())) else {
        return false;
    };
    actions[first_index + 1..]
        .iter()
        .any(|record| second(record.action()))
}
