use crate::{
    CatalogDefinition, CatalogError, CatalogEvidence, CatalogEvidenceId, Phase9ParticleAction,
    Phase9QueryControl, Phase9RayControl, SemanticEntityKind,
};

use super::{definition_with_evidence, entity_id, particles::particle_action, vec2};

/// Returns reviewed particle query, callback, and mutation definitions.
///
/// # Errors
///
/// Returns [`CatalogError`] if an action reference, evidence mapping, or bound is invalid.
pub fn definitions() -> Result<Vec<CatalogDefinition>, CatalogError> {
    Ok(vec![
        aabb_queries()?,
        ray_callbacks()?,
        lifecycle_callbacks()?,
        particle_mutations()?,
    ])
}

fn aabb_queries() -> Result<CatalogDefinition, CatalogError> {
    let system_id = entity_id(SemanticEntityKind::ParticleSystem, 0)?;
    particle_definition(
        "particle-aabb-query-controls",
        "Particle AABB Query Controls",
        &[
            "system_aabb",
            "world_aabb",
            "system_culling",
            "query_continue",
            "query_terminate",
        ],
        2,
        vec![
            Phase9ParticleAction::QueryAabb {
                system_id: Some(system_id.clone()),
                lower: vec2(-2.0, -1.0),
                upper: vec2(2.0, 1.0),
                control: Phase9QueryControl::Continue,
            },
            Phase9ParticleAction::QueryAabb {
                system_id: None,
                lower: vec2(-1.0, -1.0),
                upper: vec2(1.0, 1.0),
                control: Phase9QueryControl::Terminate,
            },
        ],
    )
}

fn ray_callbacks() -> Result<CatalogDefinition, CatalogError> {
    let system_id = entity_id(SemanticEntityKind::ParticleSystem, 0)?;
    particle_definition(
        "particle-ray-callback-controls",
        "Particle Ray Callback Controls",
        &[
            "system_ray",
            "world_ray",
            "ray_culling",
            "ray_start_inside_exclusion",
            "ray_ignore",
            "ray_continue",
            "ray_clip",
            "ray_terminate",
        ],
        2,
        [
            Phase9RayControl::Ignore,
            Phase9RayControl::Continue,
            Phase9RayControl::Clip,
            Phase9RayControl::Terminate,
        ]
        .into_iter()
        .zip([0.0, 0.25, 0.5, 0.75])
        .enumerate()
        .map(|(index, (control, height))| Phase9ParticleAction::RayCast {
            system_id: (index != 1).then(|| system_id.clone()),
            start: vec2(-3.0, height),
            end: vec2(3.0, height),
            control,
        })
        .collect(),
    )
}

fn lifecycle_callbacks() -> Result<CatalogDefinition, CatalogError> {
    particle_definition(
        "particle-lifecycle-callbacks",
        "Particle Lifecycle Callback Ordering",
        &[
            "requested_destruction_callback",
            "unrequested_destruction_callback",
            "contact_order",
            "contact_multiplicity",
        ],
        2,
        vec![
            Phase9ParticleAction::InspectOccurrence {
                occurrence_index: 0,
            },
            Phase9ParticleAction::MarkForDestruction {
                particle_id: entity_id(SemanticEntityKind::Particle, 1)?,
            },
            Phase9ParticleAction::Compact {
                system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
            },
            Phase9ParticleAction::InspectOccurrence {
                occurrence_index: 1,
            },
        ],
    )
}

fn particle_mutations() -> Result<CatalogDefinition, CatalogError> {
    particle_definition(
        "particle-mutations",
        "Particle Position and Velocity Mutations",
        &[
            "stable_ids_sort",
            "closed_policy_registry",
            "phase10_rejection",
        ],
        1,
        vec![
            Phase9ParticleAction::SetPosition {
                particle_id: entity_id(SemanticEntityKind::Particle, 1)?,
                position: vec2(0.75, -0.25),
            },
            Phase9ParticleAction::SetVelocity {
                particle_id: entity_id(SemanticEntityKind::Particle, 1)?,
                velocity: vec2(1.5, 0.5),
            },
            Phase9ParticleAction::InspectParticle {
                particle_id: entity_id(SemanticEntityKind::Particle, 1)?,
            },
        ],
    )
}

fn particle_definition(
    slug: &str,
    title: &str,
    evidence: &[&str],
    particle_count: u32,
    actions: Vec<Phase9ParticleAction>,
) -> Result<CatalogDefinition, CatalogError> {
    let system_id = entity_id(SemanticEntityKind::ParticleSystem, 0)?;
    let mut kinds = vec![SemanticEntityKind::ParticleSystem];
    let mut setup = vec![particle_action(Phase9ParticleAction::CreateSystem {
        system_id: system_id.clone(),
    })];
    for ordinal in 1..=particle_count {
        kinds.push(SemanticEntityKind::Particle);
        setup.push(particle_action(Phase9ParticleAction::CreateParticle {
            particle_id: entity_id(SemanticEntityKind::Particle, ordinal)?,
        }));
    }
    let mut logical = actions.into_iter().map(particle_action).collect::<Vec<_>>();
    logical.push(particle_action(Phase9ParticleAction::DestroySystem {
        system_id,
    }));
    definition_with_evidence(
        slug,
        title,
        "native-particle-callback-v1",
        &["particle", "query-callback"],
        "particle-queries",
        evidence
            .iter()
            .map(|id| CatalogEvidenceId::new(*id).map(CatalogEvidence::Phase9))
            .collect::<Result<Vec<_>, _>>()?,
        None,
        kinds,
        setup,
        logical,
        2,
    )
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use crate::{
        CatalogErrorKind, FloatBits, Phase9RayControl, ResolveRequest, RigidWorldAction,
        RunSettings, Sha256Hex, decode_resolved_scenario, resolve_catalog,
    };

    use super::definitions;

    #[test]
    fn query_callback_and_mutation_definitions_resolve_deterministically() {
        // Arrange
        let definitions = definitions().expect("query and callback definitions should be valid");
        let settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 2)
            .expect("settings should be valid");

        // Act / Assert
        for definition in &definitions {
            let request = ResolveRequest::new(definition.slug().clone(), None, settings);
            let first = resolve_catalog(&definitions, &request).expect("definition resolves");
            let second = resolve_catalog(&definitions, &request).expect("definition repeats");
            assert_eq!(first.canonical_bytes(), second.canonical_bytes());
        }
        assert_eq!(definitions.len(), 4);
    }

    #[test]
    fn ray_catalog_carries_every_closed_callback_directive() {
        // Arrange
        let definitions = definitions().expect("query definitions should be valid");
        let definition = definitions
            .iter()
            .find(|definition| definition.slug().as_str() == "particle-ray-callback-controls")
            .expect("ray definition exists");
        let settings = definition
            .metadata()
            .expect("metadata exists")
            .default_settings();

        // Act
        let resolved = resolve_catalog(
            &definitions,
            &ResolveRequest::new(definition.slug().clone(), None, settings),
        )
        .expect("ray definition resolves");
        let controls = resolved
            .actions()
            .iter()
            .filter_map(|action| match action.action() {
                RigidWorldAction::Particle {
                    action: crate::Phase9ParticleAction::RayCast { control, .. },
                } => Some(*control),
                _ => None,
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(
            controls,
            vec![
                Phase9RayControl::Ignore,
                Phase9RayControl::Continue,
                Phase9RayControl::Clip,
                Phase9RayControl::Terminate,
            ]
        );
    }

    #[test]
    fn decoder_rejects_unknown_particle_mutation_identity() {
        // Arrange
        let definitions = definitions().expect("query definitions should be valid");
        let definition = definitions
            .iter()
            .find(|definition| definition.slug().as_str() == "particle-mutations")
            .expect("mutation definition exists");
        let settings = definition
            .metadata()
            .expect("metadata exists")
            .default_settings();
        let resolved = resolve_catalog(
            &definitions,
            &ResolveRequest::new(definition.slug().clone(), None, settings),
        )
        .expect("mutation definition resolves");
        let mut tampered: serde_json::Value =
            serde_json::from_slice(resolved.canonical_bytes()).expect("canonical bytes decode");
        tampered["actions"][2]["action"]["action"]["particle_id"] =
            serde_json::Value::String("entity-particle-9999".to_owned());
        let tampered_bytes = serde_json::to_vec(&tampered).expect("tampered JSON encodes");
        let tampered_hash = Sha256Hex::from_digest(Sha256::digest(&tampered_bytes).into());

        // Act
        let error = decode_resolved_scenario(&tampered_bytes, &tampered_hash)
            .expect_err("unknown mutation identity must fail closed");

        // Assert
        assert_eq!(error.kind(), CatalogErrorKind::InvalidIdentifier);
    }
}
