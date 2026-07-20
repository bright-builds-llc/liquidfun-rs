//! The closed `phase10-pass-graph-v1` particle solver authority.

use std::error::Error;
use std::fmt;

use crate::particle::{ParticleFlags, ParticleGroupFlags};

use super::{PassDescriptor, PassGate, PassId};

pub(super) const PASS_GRAPH_VERSION: &str = "phase10-pass-graph-v1";
pub(super) const PASS_COUNT: usize = 31;
const OUTER_PASS_COUNT: usize = 5;
const PARTICLE_ITERATION_PASS_COUNT: usize = 26;

pub(super) const PASS_GRAPH: [PassDescriptor; PASS_COUNT] = [
    PassDescriptor::outer(PassId::Lifetime, PassGate::ExpirationLane),
    PassDescriptor::outer(
        PassId::ZombieCompaction,
        PassGate::AggregateParticleFlags(ParticleFlags::ZOMBIE),
    ),
    PassDescriptor::outer(PassId::RefreshParticleFlags, PassGate::DirtyParticleFlags),
    PassDescriptor::outer(PassId::RefreshGroupFlags, PassGate::DirtyGroupFlags),
    PassDescriptor::outer(PassId::PauseGate, PassGate::PauseTerminator),
    PassDescriptor::particle_iteration(PassId::ParticleContacts, PassGate::Always),
    PassDescriptor::particle_iteration(PassId::BodyContacts, PassGate::Always),
    PassDescriptor::particle_iteration(PassId::Weight, PassGate::Always),
    PassDescriptor::particle_iteration(PassId::SolidDepth, PassGate::NeedsGroupDepth),
    PassDescriptor::particle_iteration(
        PassId::ReactiveTopology,
        PassGate::AggregateParticleFlags(ParticleFlags::REACTIVE),
    ),
    PassDescriptor::particle_iteration(PassId::Force, PassGate::PendingForce),
    PassDescriptor::particle_iteration(
        PassId::Viscous,
        PassGate::AggregateParticleFlags(ParticleFlags::VISCOUS),
    ),
    PassDescriptor::particle_iteration(
        PassId::Repulsive,
        PassGate::AggregateParticleFlags(ParticleFlags::REPULSIVE),
    ),
    PassDescriptor::particle_iteration(
        PassId::Powder,
        PassGate::AggregateParticleFlags(ParticleFlags::POWDER),
    ),
    PassDescriptor::particle_iteration(
        PassId::Tensile,
        PassGate::AggregateParticleFlags(ParticleFlags::TENSILE),
    ),
    PassDescriptor::particle_iteration(
        PassId::Solid,
        PassGate::AggregateGroupFlags(ParticleGroupFlags::SOLID),
    ),
    PassDescriptor::particle_iteration(
        PassId::ColorMixing,
        PassGate::AggregateParticleFlags(ParticleFlags::COLOR_MIXING),
    ),
    PassDescriptor::particle_iteration(PassId::Gravity, PassGate::Always),
    PassDescriptor::particle_iteration(
        PassId::StaticPressure,
        PassGate::AggregateParticleFlags(ParticleFlags::STATIC_PRESSURE),
    ),
    PassDescriptor::particle_iteration(PassId::Pressure, PassGate::Always),
    PassDescriptor::particle_iteration(PassId::Damping, PassGate::Always),
    PassDescriptor::particle_iteration(
        PassId::ExtraDamping,
        PassGate::ExtraDampingAggregateFlags(ParticleFlags::STATIC_PRESSURE),
    ),
    PassDescriptor::particle_iteration(
        PassId::Elastic,
        PassGate::AggregateParticleFlags(ParticleFlags::ELASTIC),
    ),
    PassDescriptor::particle_iteration(
        PassId::Spring,
        PassGate::AggregateParticleFlags(ParticleFlags::SPRING),
    ),
    PassDescriptor::particle_iteration(PassId::LimitVelocity, PassGate::Always),
    PassDescriptor::particle_iteration(
        PassId::RigidDamping,
        PassGate::AggregateGroupFlags(ParticleGroupFlags::RIGID),
    ),
    PassDescriptor::particle_iteration(
        PassId::Barrier,
        PassGate::AggregateParticleFlags(ParticleFlags::BARRIER),
    ),
    PassDescriptor::particle_iteration(PassId::Collision, PassGate::Always),
    PassDescriptor::particle_iteration(
        PassId::Rigid,
        PassGate::AggregateGroupFlags(ParticleGroupFlags::RIGID),
    ),
    PassDescriptor::particle_iteration(
        PassId::Wall,
        PassGate::AggregateParticleFlags(ParticleFlags::WALL),
    ),
    PassDescriptor::particle_iteration(PassId::Integrate, PassGate::Always),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclaredPassId {
    Known(PassId),
    Unknown(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PassDeclaration {
    id: DeclaredPassId,
    descriptor: PassDescriptor,
}

impl From<PassDescriptor> for PassDeclaration {
    fn from(descriptor: PassDescriptor) -> Self {
        Self {
            id: DeclaredPassId::Known(descriptor.id),
            descriptor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ManifestValidationError {
    Unknown {
        index: usize,
        name: &'static str,
    },
    Missing {
        id: PassId,
    },
    Duplicate {
        id: PassId,
        first_index: usize,
        duplicate_index: usize,
    },
    Reordered {
        index: usize,
        expected: PassId,
        actual: PassId,
    },
    DescriptorMismatch {
        index: usize,
        id: PassId,
    },
}

impl fmt::Display for ManifestValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown { index, name } => {
                write!(formatter, "unknown pass {name:?} at index {index}")
            }
            Self::Missing { id } => write!(formatter, "missing pass {:?}", id.as_str()),
            Self::Duplicate {
                id,
                first_index,
                duplicate_index,
            } => write!(
                formatter,
                "duplicate pass {:?} at index {duplicate_index}; first declared at index {first_index}",
                id.as_str()
            ),
            Self::Reordered {
                index,
                expected,
                actual,
            } => write!(
                formatter,
                "reordered pass at index {index}: expected {:?}, found {:?}",
                expected.as_str(),
                actual.as_str()
            ),
            Self::DescriptorMismatch { index, id } => write!(
                formatter,
                "gate, scope, or multiplicity mismatch for pass {:?} at index {index}",
                id.as_str()
            ),
        }
    }
}

impl Error for ManifestValidationError {}

pub(super) fn validated_pass_graph()
-> Result<&'static [PassDescriptor; PASS_COUNT], ManifestValidationError> {
    let declarations = PASS_GRAPH.map(PassDeclaration::from);
    validate_manifest(&declarations)?;
    Ok(&PASS_GRAPH)
}

fn validate_manifest(declarations: &[PassDeclaration]) -> Result<(), ManifestValidationError> {
    for (index, declaration) in declarations.iter().enumerate() {
        if let DeclaredPassId::Unknown(name) = declaration.id {
            return Err(ManifestValidationError::Unknown { index, name });
        }
    }

    for (index, declaration) in declarations.iter().enumerate() {
        let DeclaredPassId::Known(id) = declaration.id else {
            unreachable!("unknown declarations returned before duplicate validation");
        };
        let maybe_first_index = declarations[..index]
            .iter()
            .position(|candidate| candidate.id == DeclaredPassId::Known(id));
        if let Some(first_index) = maybe_first_index {
            return Err(ManifestValidationError::Duplicate {
                id,
                first_index,
                duplicate_index: index,
            });
        }
    }

    for expected in PASS_GRAPH {
        let is_present = declarations
            .iter()
            .any(|declaration| declaration.id == DeclaredPassId::Known(expected.id));
        if !is_present {
            return Err(ManifestValidationError::Missing { id: expected.id });
        }
    }

    for (index, (expected, actual)) in PASS_GRAPH.iter().zip(declarations).enumerate() {
        let DeclaredPassId::Known(actual_id) = actual.id else {
            unreachable!("unknown declarations returned before order validation");
        };
        if actual_id != expected.id {
            return Err(ManifestValidationError::Reordered {
                index,
                expected: expected.id,
                actual: actual_id,
            });
        }
        if actual.descriptor != *expected {
            return Err(ManifestValidationError::DescriptorMismatch {
                index,
                id: expected.id,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{StepConfiguration, StepConfigurationError};

    use super::*;
    use crate::particle::solver::{PassMultiplicity, PassScope, trace_complete_graph};

    const EXPECTED_PASS_IDS: [&str; PASS_COUNT] = [
        "lifetime",
        "zombie_compaction",
        "refresh_particle_flags",
        "refresh_group_flags",
        "pause_gate",
        "particle_contacts",
        "body_contacts",
        "weight",
        "solid_depth",
        "reactive_topology",
        "force",
        "viscous",
        "repulsive",
        "powder",
        "tensile",
        "solid",
        "color_mixing",
        "gravity",
        "static_pressure",
        "pressure",
        "damping",
        "extra_damping",
        "elastic",
        "spring",
        "limit_velocity",
        "rigid_damping",
        "barrier",
        "collision",
        "rigid",
        "wall",
        "integrate",
    ];

    fn declarations() -> Vec<PassDeclaration> {
        PASS_GRAPH
            .iter()
            .copied()
            .map(PassDeclaration::from)
            .collect()
    }

    fn step_with_particle_iterations(
        particle_iterations: u32,
    ) -> Result<StepConfiguration, StepConfigurationError> {
        StepConfiguration::new(1.0 / 60.0, 8, 3)?.with_particle_iterations(particle_iterations)
    }

    #[test]
    fn manifest_has_exact_version_count_and_order() {
        // Arrange
        let graph = validated_pass_graph().expect("canonical manifest should validate");

        // Act
        let actual_ids: Vec<_> = graph
            .iter()
            .map(|descriptor| descriptor.id.as_str())
            .collect();
        let unique_ids: HashSet<_> = actual_ids.iter().copied().collect();

        // Assert
        assert_eq!(PASS_GRAPH_VERSION, "phase10-pass-graph-v1");
        assert_eq!(actual_ids, EXPECTED_PASS_IDS);
        assert_eq!(unique_ids.len(), PASS_COUNT);
        assert_eq!(
            graph
                .iter()
                .filter(|descriptor| descriptor.scope == PassScope::Outer)
                .count(),
            OUTER_PASS_COUNT
        );
        assert_eq!(
            graph
                .iter()
                .filter(|descriptor| descriptor.scope == PassScope::ParticleIteration)
                .count(),
            PARTICLE_ITERATION_PASS_COUNT
        );
    }

    #[test]
    fn manifest_encodes_exact_gates_and_multiplicity() {
        // Arrange
        let expected_gates = [
            PassGate::ExpirationLane,
            PassGate::AggregateParticleFlags(ParticleFlags::ZOMBIE),
            PassGate::DirtyParticleFlags,
            PassGate::DirtyGroupFlags,
            PassGate::PauseTerminator,
            PassGate::Always,
            PassGate::Always,
            PassGate::Always,
            PassGate::NeedsGroupDepth,
            PassGate::AggregateParticleFlags(ParticleFlags::REACTIVE),
            PassGate::PendingForce,
            PassGate::AggregateParticleFlags(ParticleFlags::VISCOUS),
            PassGate::AggregateParticleFlags(ParticleFlags::REPULSIVE),
            PassGate::AggregateParticleFlags(ParticleFlags::POWDER),
            PassGate::AggregateParticleFlags(ParticleFlags::TENSILE),
            PassGate::AggregateGroupFlags(ParticleGroupFlags::SOLID),
            PassGate::AggregateParticleFlags(ParticleFlags::COLOR_MIXING),
            PassGate::Always,
            PassGate::AggregateParticleFlags(ParticleFlags::STATIC_PRESSURE),
            PassGate::Always,
            PassGate::Always,
            PassGate::ExtraDampingAggregateFlags(ParticleFlags::STATIC_PRESSURE),
            PassGate::AggregateParticleFlags(ParticleFlags::ELASTIC),
            PassGate::AggregateParticleFlags(ParticleFlags::SPRING),
            PassGate::Always,
            PassGate::AggregateGroupFlags(ParticleGroupFlags::RIGID),
            PassGate::AggregateParticleFlags(ParticleFlags::BARRIER),
            PassGate::Always,
            PassGate::AggregateGroupFlags(ParticleGroupFlags::RIGID),
            PassGate::AggregateParticleFlags(ParticleFlags::WALL),
            PassGate::Always,
        ];

        // Act
        let actual_gates: Vec<_> = PASS_GRAPH
            .iter()
            .map(|descriptor| descriptor.gate)
            .collect();

        // Assert
        assert_eq!(actual_gates, expected_gates);
        assert!(
            PASS_GRAPH[..OUTER_PASS_COUNT]
                .iter()
                .all(|descriptor| { descriptor.multiplicity == PassMultiplicity::OncePerStep })
        );
        assert!(PASS_GRAPH[OUTER_PASS_COUNT..].iter().all(|descriptor| {
            descriptor.multiplicity == PassMultiplicity::OncePerParticleIteration
        }));
    }

    #[test]
    fn manifest_trace_uses_same_order_and_iteration_multiplicity() {
        // Arrange
        let configuration =
            step_with_particle_iterations(2).expect("two iterations are within checked bounds");

        // Act
        let trace =
            trace_complete_graph(configuration).expect("canonical manifest should validate");
        let first_iteration: Vec<_> = trace
            [OUTER_PASS_COUNT..OUTER_PASS_COUNT + PARTICLE_ITERATION_PASS_COUNT]
            .iter()
            .map(|(id, iteration)| (id.as_str(), *iteration))
            .collect();
        let second_iteration: Vec<_> = trace[OUTER_PASS_COUNT + PARTICLE_ITERATION_PASS_COUNT..]
            .iter()
            .map(|(id, iteration)| (id.as_str(), *iteration))
            .collect();

        // Assert
        assert_eq!(
            trace.len(),
            OUTER_PASS_COUNT + 2 * PARTICLE_ITERATION_PASS_COUNT
        );
        assert!(
            trace[..OUTER_PASS_COUNT]
                .iter()
                .all(|(_, iteration)| iteration.is_none())
        );
        assert_eq!(
            first_iteration,
            EXPECTED_PASS_IDS[OUTER_PASS_COUNT..]
                .iter()
                .map(|id| (*id, Some(0)))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            second_iteration,
            EXPECTED_PASS_IDS[OUTER_PASS_COUNT..]
                .iter()
                .map(|id| (*id, Some(1)))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn manifest_rejects_deleted_pass() {
        // Arrange
        let mut mutated = declarations();
        mutated.remove(7);

        // Act
        let error = validate_manifest(&mutated).expect_err("missing weight must fail");

        // Assert
        assert_eq!(
            error,
            ManifestValidationError::Missing { id: PassId::Weight }
        );
        assert_eq!(error.to_string(), "missing pass \"weight\"");
    }

    #[test]
    fn manifest_rejects_duplicate_pass() {
        // Arrange
        let mut mutated = declarations();
        mutated[7] = mutated[6];

        // Act
        let error = validate_manifest(&mutated).expect_err("duplicate body contacts must fail");

        // Assert
        assert_eq!(
            error,
            ManifestValidationError::Duplicate {
                id: PassId::BodyContacts,
                first_index: 6,
                duplicate_index: 7,
            }
        );
        assert_eq!(
            error.to_string(),
            "duplicate pass \"body_contacts\" at index 7; first declared at index 6"
        );
    }

    #[test]
    fn manifest_rejects_reordered_passes() {
        // Arrange
        let mut mutated = declarations();
        mutated.swap(5, 6);

        // Act
        let error = validate_manifest(&mutated).expect_err("swapped contacts must fail");

        // Assert
        assert_eq!(
            error,
            ManifestValidationError::Reordered {
                index: 5,
                expected: PassId::ParticleContacts,
                actual: PassId::BodyContacts,
            }
        );
        assert_eq!(
            error.to_string(),
            "reordered pass at index 5: expected \"particle_contacts\", found \"body_contacts\""
        );
    }

    #[test]
    fn manifest_rejects_unknown_added_pass() {
        // Arrange
        let mut mutated = declarations();
        mutated.push(PassDeclaration {
            id: DeclaredPassId::Unknown("post_integrate_cleanup"),
            descriptor: PASS_GRAPH[PASS_COUNT - 1],
        });

        // Act
        let error = validate_manifest(&mutated).expect_err("unknown appended pass must fail");

        // Assert
        assert_eq!(
            error,
            ManifestValidationError::Unknown {
                index: PASS_COUNT,
                name: "post_integrate_cleanup",
            }
        );
        assert_eq!(
            error.to_string(),
            "unknown pass \"post_integrate_cleanup\" at index 31"
        );
    }

    #[test]
    fn manifest_rejects_changed_gate() {
        // Arrange
        let mut mutated = declarations();
        mutated[21].descriptor.gate = PassGate::ExtraDampingAggregateFlags(ParticleFlags::VISCOUS);

        // Act
        let error = validate_manifest(&mutated).expect_err("changed gate must fail");

        // Assert
        assert_eq!(
            error,
            ManifestValidationError::DescriptorMismatch {
                index: 21,
                id: PassId::ExtraDamping,
            }
        );
    }

    #[test]
    fn manifest_rejects_changed_scope() {
        // Arrange
        let mut mutated = declarations();
        mutated[5].descriptor.scope = PassScope::Outer;

        // Act
        let error = validate_manifest(&mutated).expect_err("changed scope must fail");

        // Assert
        assert_eq!(
            error,
            ManifestValidationError::DescriptorMismatch {
                index: 5,
                id: PassId::ParticleContacts,
            }
        );
    }

    #[test]
    fn manifest_rejects_changed_multiplicity() {
        // Arrange
        let mut mutated = declarations();
        mutated[5].descriptor.multiplicity = PassMultiplicity::OncePerStep;

        // Act
        let error = validate_manifest(&mutated).expect_err("changed multiplicity must fail");

        // Assert
        assert_eq!(
            error,
            ManifestValidationError::DescriptorMismatch {
                index: 5,
                id: PassId::ParticleContacts,
            }
        );
    }
}

#[cfg(test)]
#[path = "manifest/witness_registry.rs"]
mod witness_registry;
