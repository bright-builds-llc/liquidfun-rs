use crate::{
    CatalogDefinition, CatalogError, RigidJointKind, RigidJointMutation, RigidWorldAction,
    RigidWorldWitness, SemanticEntityKind,
};

use super::{bits, configured_steps, default_settings, definition, entity_id, vec2};

/// Returns one reviewed native scenario for each supported rigid joint kind.
///
/// # Errors
///
/// Returns [`CatalogError`] if a stable ID, schedule, setting, or coverage declaration is invalid.
pub fn definitions() -> Result<Vec<CatalogDefinition>, CatalogError> {
    RigidJointKind::ALL
        .into_iter()
        .map(joint_definition)
        .collect()
}

fn joint_definition(kind: RigidJointKind) -> Result<CatalogDefinition, CatalogError> {
    let settings = default_settings(1)?;
    let (entity_kinds, mut setup, joint_id) = if kind == RigidJointKind::Gear {
        gear_setup()?
    } else {
        standard_setup()?
    };
    if let Some(mutation) = mutation_for(kind) {
        setup.push(RigidWorldAction::MutateJoint {
            joint_id: joint_id.clone(),
            mutation,
        });
    }
    setup.push(RigidWorldAction::InspectJoint {
        joint_id: joint_id.clone(),
    });
    let slug = joint_slug(kind);
    definition(
        &format!("joint-{slug}-behavior"),
        &format!("{} Joint Behavior", joint_title(kind)),
        "native-joint-v1",
        &["joints", slug],
        &format!("joint-{slug}-test"),
        witnesses(kind),
        Some(kind),
        entity_kinds,
        setup,
        configured_steps(settings, 2),
        1,
    )
}

fn standard_setup() -> Result<
    (
        Vec<SemanticEntityKind>,
        Vec<RigidWorldAction>,
        crate::ScenarioId,
    ),
    CatalogError,
> {
    let body_a = entity_id(SemanticEntityKind::Body, 0)?;
    let body_b = entity_id(SemanticEntityKind::Body, 1)?;
    let joint = entity_id(SemanticEntityKind::Joint, 2)?;
    Ok((
        vec![
            SemanticEntityKind::Body,
            SemanticEntityKind::Body,
            SemanticEntityKind::Joint,
        ],
        vec![
            RigidWorldAction::CreateBody { body_id: body_a },
            RigidWorldAction::CreateBody { body_id: body_b },
            RigidWorldAction::CreateJoint {
                joint_id: joint.clone(),
            },
        ],
        joint,
    ))
}

fn gear_setup() -> Result<
    (
        Vec<SemanticEntityKind>,
        Vec<RigidWorldAction>,
        crate::ScenarioId,
    ),
    CatalogError,
> {
    let mut entity_kinds = Vec::with_capacity(7);
    let mut setup = Vec::with_capacity(7);
    for ordinal in 0..4 {
        entity_kinds.push(SemanticEntityKind::Body);
        setup.push(RigidWorldAction::CreateBody {
            body_id: entity_id(SemanticEntityKind::Body, ordinal)?,
        });
    }
    for ordinal in 4..7 {
        entity_kinds.push(SemanticEntityKind::Joint);
        setup.push(RigidWorldAction::CreateJoint {
            joint_id: entity_id(SemanticEntityKind::Joint, ordinal)?,
        });
    }
    Ok((
        entity_kinds,
        setup,
        entity_id(SemanticEntityKind::Joint, 6)?,
    ))
}

fn mutation_for(kind: RigidJointKind) -> Option<RigidJointMutation> {
    match kind {
        RigidJointKind::Revolute => Some(RigidJointMutation::MotorEnabled { enabled: true }),
        RigidJointKind::Prismatic => Some(RigidJointMutation::Limits {
            lower_bits: bits(-1.0),
            upper_bits: bits(1.0),
        }),
        RigidJointKind::Distance => Some(RigidJointMutation::Length {
            length_bits: bits(2.0),
        }),
        RigidJointKind::Pulley => None,
        RigidJointKind::Mouse => Some(RigidJointMutation::MouseTarget {
            target: vec2(1.0, 2.0),
        }),
        RigidJointKind::Gear => Some(RigidJointMutation::GearRatio {
            ratio_bits: bits(-2.0),
        }),
        RigidJointKind::Wheel => Some(RigidJointMutation::MaxMotorTorque {
            torque_bits: bits(5.0),
        }),
        RigidJointKind::Weld => Some(RigidJointMutation::Frequency {
            frequency_bits: bits(4.0),
        }),
        RigidJointKind::Friction => Some(RigidJointMutation::MaxForce {
            force_bits: bits(3.0),
        }),
        RigidJointKind::Rope => Some(RigidJointMutation::RopeMaxLength {
            max_length_bits: bits(3.0),
        }),
        RigidJointKind::Motor => Some(RigidJointMutation::CorrectionFactor {
            factor_bits: bits(0.5),
        }),
    }
}

const fn joint_slug(kind: RigidJointKind) -> &'static str {
    match kind {
        RigidJointKind::Revolute => "revolute",
        RigidJointKind::Prismatic => "prismatic",
        RigidJointKind::Distance => "distance",
        RigidJointKind::Pulley => "pulley",
        RigidJointKind::Mouse => "mouse",
        RigidJointKind::Gear => "gear",
        RigidJointKind::Wheel => "wheel",
        RigidJointKind::Weld => "weld",
        RigidJointKind::Friction => "friction",
        RigidJointKind::Rope => "rope",
        RigidJointKind::Motor => "motor",
    }
}

const fn joint_title(kind: RigidJointKind) -> &'static str {
    match kind {
        RigidJointKind::Revolute => "Revolute",
        RigidJointKind::Prismatic => "Prismatic",
        RigidJointKind::Distance => "Distance",
        RigidJointKind::Pulley => "Pulley",
        RigidJointKind::Mouse => "Mouse",
        RigidJointKind::Gear => "Gear",
        RigidJointKind::Wheel => "Wheel",
        RigidJointKind::Weld => "Weld",
        RigidJointKind::Friction => "Friction",
        RigidJointKind::Rope => "Rope",
        RigidJointKind::Motor => "Motor",
    }
}

fn witnesses(kind: RigidJointKind) -> &'static [RigidWorldWitness] {
    match kind {
        RigidJointKind::Revolute => &[
            RigidWorldWitness::RevoluteLimitsAndMotorStepped,
            RigidWorldWitness::RevolutePrismaticReactionObserved,
        ],
        RigidJointKind::Prismatic => &[
            RigidWorldWitness::PrismaticLimitsAndMotorStepped,
            RigidWorldWitness::JointLowerLimitObserved,
        ],
        RigidJointKind::Distance => &[
            RigidWorldWitness::RigidDistanceStepped,
            RigidWorldWitness::SoftDistanceStepped,
        ],
        RigidJointKind::Pulley => &[RigidWorldWitness::PulleyRatioStepped],
        RigidJointKind::Mouse => &[RigidWorldWitness::MouseTargetAndForceStepped],
        RigidJointKind::Gear => &[
            RigidWorldWitness::GearFourBodyTopologyObserved,
            RigidWorldWitness::GearDependencyOrderObserved,
        ],
        RigidJointKind::Wheel => &[RigidWorldWitness::WheelSpringAndMotorStepped],
        RigidJointKind::Weld => &[
            RigidWorldWitness::RigidWeldStepped,
            RigidWorldWitness::SoftWeldStepped,
        ],
        RigidJointKind::Friction => &[RigidWorldWitness::FrictionCapsStepped],
        RigidJointKind::Rope => &[
            RigidWorldWitness::RopeJointInactiveStepped,
            RigidWorldWitness::RopeJointUpperLimitStepped,
        ],
        RigidJointKind::Motor => &[RigidWorldWitness::MotorCorrectionStepped],
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        FloatBits, ResolveRequest, RigidJointKind, RigidWorldAction, RunSettings, resolve_catalog,
    };

    use super::definitions;

    #[test]
    fn joint_definitions_cover_every_kind_with_stable_actions() {
        // Arrange
        let definitions = definitions().expect("joint definitions should be valid");
        let settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 1)
            .expect("default settings should be valid");
        let mut slugs = HashSet::new();
        let mut covered_kinds = HashSet::new();

        // Act
        for definition in &definitions {
            assert!(slugs.insert(definition.slug().as_str()));
            let metadata = definition
                .metadata()
                .expect("joint examples must declare metadata");
            assert_eq!(metadata.default_settings(), settings);
            covered_kinds.insert(metadata.joint_kind().expect("joint kind must be declared"));
            let request = ResolveRequest::new(definition.slug().clone(), None, settings);
            let resolved =
                resolve_catalog(&definitions, &request).expect("joint definition should resolve");
            let entity_ids = resolved
                .entities()
                .iter()
                .map(|entity| entity.scenario_id().as_str())
                .collect::<HashSet<_>>();
            let action_ids = resolved
                .actions()
                .iter()
                .map(|action| action.action_id().as_str())
                .collect::<HashSet<_>>();

            // Assert
            assert_eq!(entity_ids.len(), resolved.entities().len());
            assert_eq!(action_ids.len(), resolved.actions().len());
            assert!(
                resolved.actions().iter().any(|action| {
                    matches!(action.action(), RigidWorldAction::CreateJoint { .. })
                })
            );
            assert!(resolved.actions().iter().any(|action| {
                matches!(action.action(), RigidWorldAction::InspectJoint { .. })
            }));
        }
        assert_eq!(definitions.len(), RigidJointKind::ALL.len());
        assert_eq!(covered_kinds.len(), RigidJointKind::ALL.len());
    }
}
