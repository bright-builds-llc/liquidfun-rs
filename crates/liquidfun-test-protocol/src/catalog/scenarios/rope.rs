use crate::{
    CatalogDefinition, CatalogError, RigidWorldAction, RigidWorldWitness, SemanticEntityKind,
};

use super::{bits, default_settings, definition, entity_id};

/// Returns the reviewed standalone rope scenario definition.
///
/// # Errors
///
/// Returns [`CatalogError`] if a stable ID, schedule, setting, or coverage declaration is invalid.
pub fn definitions() -> Result<Vec<CatalogDefinition>, CatalogError> {
    let rope_id = entity_id(SemanticEntityKind::Rope, 0)?;
    let settings = default_settings(8)?;
    Ok(vec![definition(
        "standalone-rope-evolution",
        "Standalone Rope Evolution",
        "native-rope-v1",
        &["rope", "standalone"],
        "standalone-rope-test",
        &[
            RigidWorldWitness::StandaloneRopePositiveStep,
            RigidWorldWitness::StandaloneRopeAngleObserved,
            RigidWorldWitness::StandaloneRopeVerticesObserved,
        ],
        None,
        vec![SemanticEntityKind::Rope],
        vec![
            RigidWorldAction::CreateRope {
                rope_id: rope_id.clone(),
            },
            RigidWorldAction::SetRopeAngle {
                rope_id: rope_id.clone(),
                angle_bits: bits(0.25),
            },
        ],
        vec![
            RigidWorldAction::StepRope {
                rope_id: rope_id.clone(),
                timestep_bits: settings.timestep_bits(),
                iterations: settings.particle_iterations(),
            },
            RigidWorldAction::StepRope {
                rope_id: rope_id.clone(),
                timestep_bits: settings.timestep_bits(),
                iterations: settings.particle_iterations(),
            },
            RigidWorldAction::InspectRope { rope_id },
        ],
        8,
    )?])
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use sha2::{Digest, Sha256};

    use crate::{
        CatalogErrorKind, FloatBits, ResolveRequest, RigidWorldAction, RunSettings, Sha256Hex,
        decode_resolved_scenario, resolve_catalog,
    };

    use super::definitions;

    #[test]
    fn standalone_rope_definition_resolves_rope_specific_steps_deterministically() {
        // Arrange
        let definitions = definitions().expect("rope definition should be valid");
        let definition = definitions
            .first()
            .expect("one rope definition is required");
        let settings = RunSettings::new(FloatBits::from_f32(1.0 / 30.0), 6, 2, 4)
            .expect("rope settings should be valid");
        let request = ResolveRequest::new(definition.slug().clone(), None, settings);

        // Act
        let first =
            resolve_catalog(&definitions, &request).expect("rope definition should resolve");
        let second = resolve_catalog(&definitions, &request)
            .expect("rope definition should resolve identically");
        let decoded =
            decode_resolved_scenario(first.canonical_bytes(), first.identity().content_sha256())
                .expect("resolved rope bytes should replay");
        let entity_ids = first
            .entities()
            .iter()
            .map(|entity| entity.scenario_id().as_str())
            .collect::<HashSet<_>>();
        let action_ids = first
            .actions()
            .iter()
            .map(|action| action.action_id().as_str())
            .collect::<HashSet<_>>();

        // Assert
        assert_eq!(definitions.len(), 1);
        assert_eq!(first.canonical_bytes(), second.canonical_bytes());
        assert_eq!(decoded, first);
        assert_eq!(entity_ids.len(), first.entities().len());
        assert_eq!(action_ids.len(), first.actions().len());
        assert_ne!(
            definition
                .metadata()
                .expect("rope metadata must be declared")
                .default_settings(),
            settings
        );
        assert!(
            first
                .actions()
                .iter()
                .any(|action| { matches!(action.action(), RigidWorldAction::CreateRope { .. }) })
        );
        assert!(
            first
                .actions()
                .iter()
                .any(|action| { matches!(action.action(), RigidWorldAction::StepRope { .. }) })
        );
        assert!(first.actions().iter().all(|action| {
            let RigidWorldAction::StepRope {
                timestep_bits,
                iterations,
                ..
            } = action.action()
            else {
                return true;
            };
            *timestep_bits == settings.timestep_bits()
                && *iterations == settings.particle_iterations()
        }));

        let mut tampered: serde_json::Value = serde_json::from_slice(first.canonical_bytes())
            .expect("canonical bytes should decode as JSON");
        tampered["actions"][0]["action"]["rope_id"] =
            serde_json::Value::String("entity-rope-9999".to_owned());
        let tampered_bytes = serde_json::to_vec(&tampered).expect("tampered JSON should encode");
        let tampered_hash = Sha256Hex::from_digest(Sha256::digest(&tampered_bytes).into());
        let error = decode_resolved_scenario(&tampered_bytes, &tampered_hash)
            .expect_err("unknown action entity must fail closed");
        assert_eq!(error.kind(), CatalogErrorKind::InvalidIdentifier);
    }
}
