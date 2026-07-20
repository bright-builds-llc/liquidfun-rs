//! Private admission, ordering, and tracing contracts for particle solver passes.

mod manifest;
mod pressure;

use super::{ParticleFlags, ParticleGroupFlags};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PassId {
    Lifetime,
    ZombieCompaction,
    RefreshParticleFlags,
    RefreshGroupFlags,
    PauseGate,
    ParticleContacts,
    BodyContacts,
    Weight,
    SolidDepth,
    ReactiveTopology,
    Force,
    Viscous,
    Repulsive,
    Powder,
    Tensile,
    Solid,
    ColorMixing,
    Gravity,
    StaticPressure,
    Pressure,
    Damping,
    ExtraDamping,
    Elastic,
    Spring,
    LimitVelocity,
    RigidDamping,
    Barrier,
    Collision,
    Rigid,
    Wall,
    Integrate,
}

impl PassId {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Lifetime => "lifetime",
            Self::ZombieCompaction => "zombie_compaction",
            Self::RefreshParticleFlags => "refresh_particle_flags",
            Self::RefreshGroupFlags => "refresh_group_flags",
            Self::PauseGate => "pause_gate",
            Self::ParticleContacts => "particle_contacts",
            Self::BodyContacts => "body_contacts",
            Self::Weight => "weight",
            Self::SolidDepth => "solid_depth",
            Self::ReactiveTopology => "reactive_topology",
            Self::Force => "force",
            Self::Viscous => "viscous",
            Self::Repulsive => "repulsive",
            Self::Powder => "powder",
            Self::Tensile => "tensile",
            Self::Solid => "solid",
            Self::ColorMixing => "color_mixing",
            Self::Gravity => "gravity",
            Self::StaticPressure => "static_pressure",
            Self::Pressure => "pressure",
            Self::Damping => "damping",
            Self::ExtraDamping => "extra_damping",
            Self::Elastic => "elastic",
            Self::Spring => "spring",
            Self::LimitVelocity => "limit_velocity",
            Self::RigidDamping => "rigid_damping",
            Self::Barrier => "barrier",
            Self::Collision => "collision",
            Self::Rigid => "rigid",
            Self::Wall => "wall",
            Self::Integrate => "integrate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PassScope {
    Outer,
    ParticleIteration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PassGate {
    ExpirationLane,
    AggregateParticleFlags(ParticleFlags),
    DirtyParticleFlags,
    DirtyGroupFlags,
    PauseTerminator,
    Always,
    NeedsGroupDepth,
    PendingForce,
    AggregateGroupFlags(ParticleGroupFlags),
    ExtraDampingAggregateFlags(ParticleFlags),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PassMultiplicity {
    OncePerStep,
    OncePerParticleIteration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PassDescriptor {
    id: PassId,
    scope: PassScope,
    gate: PassGate,
    multiplicity: PassMultiplicity,
}

impl PassDescriptor {
    const fn outer(id: PassId, gate: PassGate) -> Self {
        Self {
            id,
            scope: PassScope::Outer,
            gate,
            multiplicity: PassMultiplicity::OncePerStep,
        }
    }

    const fn particle_iteration(id: PassId, gate: PassGate) -> Self {
        Self {
            id,
            scope: PassScope::ParticleIteration,
            gate,
            multiplicity: PassMultiplicity::OncePerParticleIteration,
        }
    }
}

#[cfg(any(test, feature = "differential-internals"))]
type PassTraceEntry = (PassId, Option<u32>);

#[cfg(any(test, feature = "differential-internals"))]
fn trace_complete_graph(
    configuration: crate::StepConfiguration,
) -> Result<Vec<PassTraceEntry>, manifest::ManifestValidationError> {
    let graph = manifest::validated_pass_graph()?;
    let mut trace = Vec::with_capacity(
        5 + 26
            * usize::try_from(configuration.particle_iterations())
                .expect("checked particle iteration count fits usize"),
    );

    trace.extend(
        graph
            .iter()
            .filter(|descriptor| descriptor.scope == PassScope::Outer)
            .map(|descriptor| (descriptor.id, None)),
    );
    for particle_iteration in 0..configuration.particle_iterations() {
        trace.extend(
            graph
                .iter()
                .filter(|descriptor| descriptor.scope == PassScope::ParticleIteration)
                .map(|descriptor| (descriptor.id, Some(particle_iteration))),
        );
    }

    Ok(trace)
}
