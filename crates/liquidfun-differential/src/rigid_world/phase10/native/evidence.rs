//! Stable lifecycle-event and witness projection.

use liquidfun::{
    DestroyedId, ParticleBodyContactEffect, ParticleContactEffect, ParticleFlags,
    StepLifecycleEvent,
};
use liquidfun_test_protocol::{
    Phase10BehaviorLeaf, Phase10BodyContact, Phase10Event, Phase10EventKind, Phase10PairSnapshot,
    Phase10ParticleSnapshot, Phase10TriadSnapshot, Phase10Witness, Phase10WitnessObservation,
    RigidWorldActionRecord, ScenarioId, WitnessRole,
};

use super::{
    NativeRigidWorldError, TimelineExecutor, action_error, checked_u32, semantic_system_id,
};

pub(super) fn collect_step_events(
    executor: &mut TimelineExecutor,
    lifecycle: &[StepLifecycleEvent],
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    for event in lifecycle {
        match event {
            StepLifecycleEvent::ParticleDestruction(destruction) => {
                let DestroyedId::Particle(particle) = destruction.destroyed() else {
                    continue;
                };
                let (system_id, particle_id) = particle_binding(executor, particle, record)?;
                push_event(
                    executor,
                    Phase10EventKind::ParticleDestroyed,
                    system_id,
                    None,
                    Some(particle_id),
                    None,
                    None,
                    record,
                )?;
            }
            StepLifecycleEvent::Destruction(destruction) => {
                let DestroyedId::ParticleGroup(group) = destruction.destroyed() else {
                    continue;
                };
                let binding = executor
                    .phase10
                    .groups
                    .iter()
                    .find(|binding| binding.group == group)
                    .cloned()
                    .ok_or_else(|| {
                        action_error(record, "particle-group event has no semantic identity")
                    })?;
                push_event(
                    executor,
                    Phase10EventKind::GroupDestroyed,
                    binding.semantic_system_id,
                    Some(binding.semantic_id),
                    None,
                    None,
                    None,
                    record,
                )?;
            }
            StepLifecycleEvent::ParticleContact(effect) => {
                let (kind, [particle_a, particle_b]) = match effect {
                    ParticleContactEffect::Begin(contact) => {
                        (Phase10EventKind::ParticleContactBegin, contact.particles())
                    }
                    ParticleContactEffect::End(particles) => {
                        (Phase10EventKind::ParticleContactEnd, *particles)
                    }
                };
                let (system_id, particle_a_id) = particle_binding(executor, particle_a, record)?;
                let (_, particle_b_id) = particle_binding(executor, particle_b, record)?;
                push_event(
                    executor,
                    kind,
                    system_id,
                    None,
                    Some(particle_a_id),
                    Some(particle_b_id),
                    None,
                    record,
                )?;
            }
            StepLifecycleEvent::ParticleBodyContact(effect) => {
                let (kind, particle, maybe_body) = match effect {
                    ParticleBodyContactEffect::Begin(contact) => (
                        Phase10EventKind::BodyContactBegin,
                        contact.particle(),
                        Some(executor.semantic_body(contact.body())?),
                    ),
                    ParticleBodyContactEffect::End { particle, .. } => {
                        (Phase10EventKind::BodyContactEnd, *particle, None)
                    }
                };
                let (system_id, particle_id) = particle_binding(executor, particle, record)?;
                push_event(
                    executor,
                    kind,
                    system_id,
                    None,
                    Some(particle_id),
                    None,
                    maybe_body,
                    record,
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn capture_witnesses(
    executor: &TimelineExecutor,
    particles: &[Phase10ParticleSnapshot],
    pairs: &[Phase10PairSnapshot],
    triads: &[Phase10TriadSnapshot],
    body_contacts: &[Phase10BodyContact],
    record: &RigidWorldActionRecord,
) -> Result<Vec<Phase10Witness>, NativeRigidWorldError> {
    let mut witnesses = executor.phase10.witnesses.clone();
    let aggregate_flags = particles
        .iter()
        .fold(0_u32, |flags, particle| flags | particle.flags_bits);
    let leaves = material_leaves();
    push_flag_witness(
        &mut witnesses,
        Phase10BehaviorLeaf::Water,
        particles.iter().any(|particle| particle.flags_bits == 0),
        0,
        record,
    )?;
    for (leaf, bits) in leaves {
        push_flag_witness(
            &mut witnesses,
            leaf,
            aggregate_flags & bits != 0,
            bits,
            record,
        )?;
    }
    let aggregate_group_flags =
        executor
            .phase10
            .groups
            .iter()
            .try_fold(0_u32, |flags, binding| {
                executor
                    .world
                    .particle_group_view(binding.group)
                    .map(|view| flags | view.flags().bits())
                    .map_err(|error| action_error(record, error))
            })?;
    for (leaf, bits) in [
        (
            Phase10BehaviorLeaf::SolidGroup,
            liquidfun::particle::ParticleGroupFlags::SOLID.bits(),
        ),
        (
            Phase10BehaviorLeaf::RigidGroup,
            liquidfun::particle::ParticleGroupFlags::RIGID.bits(),
        ),
    ] {
        let active = aggregate_group_flags & bits != 0;
        upsert_witness(
            &mut witnesses,
            leaf,
            if active {
                WitnessRole::Activation
            } else {
                WitnessRole::Control
            },
            if active {
                Phase10WitnessObservation::Count { value: 1 }
            } else {
                Phase10WitnessObservation::ControlUnchanged
            },
            record,
        )?;
    }
    let body_active = !body_contacts.is_empty();
    upsert_witness(
        &mut witnesses,
        Phase10BehaviorLeaf::BodyInteraction,
        if body_active {
            WitnessRole::Activation
        } else {
            WitnessRole::Control
        },
        if body_active {
            Phase10WitnessObservation::Count {
                value: checked_u32(body_contacts.len(), record)?,
            }
        } else {
            Phase10WitnessObservation::ControlUnchanged
        },
        record,
    )?;
    for leaf in [
        Phase10BehaviorLeaf::Spring,
        Phase10BehaviorLeaf::Elastic,
        Phase10BehaviorLeaf::Reactive,
    ] {
        upsert_witness(
            &mut witnesses,
            leaf,
            WitnessRole::Interaction,
            Phase10WitnessObservation::Topology {
                pair_count: checked_u32(pairs.len(), record)?,
                triad_count: checked_u32(triads.len(), record)?,
            },
            record,
        )?;
    }
    if let Some((particle_id, before, after)) = executor
        .phase10
        .velocity_witnesses
        .iter()
        .find(|(id, _, _)| particles.iter().any(|particle| &particle.particle_id == id))
    {
        for (leaf, bits) in leaves {
            if aggregate_flags & bits == 0 {
                continue;
            }
            upsert_witness(
                &mut witnesses,
                leaf,
                WitnessRole::Interaction,
                Phase10WitnessObservation::ParticleVelocity {
                    particle_id: particle_id.clone(),
                    before: *before,
                    after: *after,
                },
                record,
            )?;
        }
    }
    for (ordinal, witness) in witnesses.iter_mut().enumerate() {
        witness.ordinal = checked_u32(ordinal, record)?;
    }
    Ok(witnesses)
}

const fn material_leaves() -> [(Phase10BehaviorLeaf, u32); 12] {
    [
        (Phase10BehaviorLeaf::Zombie, ParticleFlags::ZOMBIE.bits()),
        (Phase10BehaviorLeaf::Wall, ParticleFlags::WALL.bits()),
        (Phase10BehaviorLeaf::Spring, ParticleFlags::SPRING.bits()),
        (Phase10BehaviorLeaf::Elastic, ParticleFlags::ELASTIC.bits()),
        (Phase10BehaviorLeaf::Viscous, ParticleFlags::VISCOUS.bits()),
        (Phase10BehaviorLeaf::Powder, ParticleFlags::POWDER.bits()),
        (Phase10BehaviorLeaf::Tensile, ParticleFlags::TENSILE.bits()),
        (
            Phase10BehaviorLeaf::ColorMixing,
            ParticleFlags::COLOR_MIXING.bits(),
        ),
        (Phase10BehaviorLeaf::Barrier, ParticleFlags::BARRIER.bits()),
        (
            Phase10BehaviorLeaf::StaticPressure,
            ParticleFlags::STATIC_PRESSURE.bits(),
        ),
        (
            Phase10BehaviorLeaf::Reactive,
            ParticleFlags::REACTIVE.bits(),
        ),
        (
            Phase10BehaviorLeaf::Repulsive,
            ParticleFlags::REPULSIVE.bits(),
        ),
    ]
}

fn push_flag_witness(
    witnesses: &mut Vec<Phase10Witness>,
    leaf: Phase10BehaviorLeaf,
    active: bool,
    flags_bits: u32,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    upsert_witness(
        witnesses,
        leaf,
        if active {
            WitnessRole::Activation
        } else {
            WitnessRole::Control
        },
        if active {
            Phase10WitnessObservation::FlagActivated { flags_bits }
        } else {
            Phase10WitnessObservation::ControlUnchanged
        },
        record,
    )
}

pub(super) fn upsert_witness(
    witnesses: &mut Vec<Phase10Witness>,
    leaf: Phase10BehaviorLeaf,
    role: WitnessRole,
    observation: Phase10WitnessObservation,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    if let Some(witness) = witnesses
        .iter_mut()
        .find(|witness| witness.behavior_leaf == leaf && witness.role == role)
    {
        witness.observation = observation;
        return Ok(());
    }
    witnesses.push(Phase10Witness {
        ordinal: checked_u32(witnesses.len(), record)?,
        behavior_leaf: leaf,
        role,
        observation,
    });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_event(
    executor: &mut TimelineExecutor,
    kind: Phase10EventKind,
    system_id: ScenarioId,
    maybe_group_id: Option<ScenarioId>,
    maybe_particle_id: Option<ScenarioId>,
    maybe_other_particle_id: Option<ScenarioId>,
    maybe_body_id: Option<ScenarioId>,
    record: &RigidWorldActionRecord,
) -> Result<u32, NativeRigidWorldError> {
    let ordinal = checked_u32(executor.phase10.events.len(), record)?;
    executor.phase10.events.push(Phase10Event {
        ordinal,
        kind,
        system_id,
        maybe_group_id,
        maybe_particle_id,
        maybe_other_particle_id,
        maybe_body_id,
    });
    Ok(ordinal)
}

fn particle_binding(
    executor: &TimelineExecutor,
    particle: liquidfun::ParticleId,
    record: &RigidWorldActionRecord,
) -> Result<(ScenarioId, ScenarioId), NativeRigidWorldError> {
    let (particle_id, system, _) = executor
        .particles
        .iter()
        .find(|(_, _, candidate)| *candidate == particle)
        .ok_or_else(|| action_error(record, "particle event has no semantic identity"))?;
    Ok((
        semantic_system_id(executor, *system, record)?,
        particle_id.clone(),
    ))
}
