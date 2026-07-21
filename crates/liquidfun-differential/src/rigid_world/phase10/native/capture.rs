//! Canonical capture through curated world views.

#![allow(
    clippy::similar_names,
    reason = "protocol endpoint fields deliberately use particle_a/b/c naming"
)]

use liquidfun_test_protocol::{
    FloatBits, Phase10BodyContact, Phase10GroupSnapshot, Phase10Observation, Phase10PairSnapshot,
    Phase10ParticleContact, Phase10ParticleSnapshot, Phase10Provenance, Phase10SemanticOutcome,
    Phase10StateObservation, Phase10TriadSnapshot, RigidWorldAction, RigidWorldActionRecord,
    RigidWorldObservation, RigidWorldTimeline, TransformBits,
};

use super::{
    NativeRigidWorldError, TimelineExecutor, action_error, checked_u32, evidence,
    semantic_particle_id, semantic_system_id, vec2_bits,
};

pub(super) fn inspect(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let provenance = executor
        .phase10
        .maybe_provenance
        .clone()
        .or_else(|| timeline_provenance(timeline))
        .ok_or_else(|| action_error(record, "Phase 10 inspection has no provenance"))?;
    let observation = capture_state(executor, provenance, record)?;
    observation
        .validate_semantics()
        .map_err(|error| action_error(record, error))?;
    executor
        .semantic_observations
        .push(RigidWorldObservation::ParticleGroup { observation });
    Ok(())
}

fn capture_state(
    executor: &TimelineExecutor,
    provenance: Phase10Provenance,
    record: &RigidWorldActionRecord,
) -> Result<Phase10Observation, NativeRigidWorldError> {
    let mut groups = Vec::with_capacity(executor.phase10.groups.len());
    let mut particles = Vec::new();
    let mut systems = Vec::new();
    for (ordinal, binding) in executor.phase10.groups.iter().enumerate() {
        let view = executor
            .world
            .particle_group_view(binding.group)
            .map_err(|error| action_error(record, error))?;
        let member_ids = view
            .member_ids()
            .iter()
            .map(|particle| semantic_particle_id(executor, binding.system, *particle, record))
            .collect::<Result<Vec<_>, _>>()?;
        groups.push(Phase10GroupSnapshot {
            ordinal: checked_u32(ordinal, record)?,
            group_id: binding.semantic_id.clone(),
            system_id: binding.semantic_system_id.clone(),
            member_ids: member_ids.clone().into_boxed_slice(),
            group_flags_bits: view.flags().bits(),
            transform: TransformBits {
                position: vec2_bits(view.position()),
                angle_bits: FloatBits::from_f32(view.angle()),
            },
            center: vec2_bits(view.center()),
            linear_velocity: vec2_bits(view.linear_velocity()),
            angular_velocity_bits: FloatBits::from_f32(view.angular_velocity()),
            mass_bits: FloatBits::from_f32(view.mass()),
            inertia_bits: FloatBits::from_f32(view.inertia()),
            maybe_depths_bits: view.maybe_depths().map(|depths| {
                depths
                    .iter()
                    .copied()
                    .map(FloatBits::from_f32)
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            }),
        });
        let system_view = executor
            .world
            .particle_system_view(binding.system)
            .map_err(|error| action_error(record, error))?;
        for (particle_id, particle) in member_ids.into_iter().zip(view.member_ids()) {
            let index = system_view
                .particle_ids()
                .iter()
                .position(|candidate| candidate == particle)
                .ok_or_else(|| {
                    action_error(record, "group member is absent from its system view")
                })?;
            let snapshot = executor
                .world
                .particle_snapshot_in_system(binding.system, *particle)
                .map_err(|error| action_error(record, error))?;
            particles.push(Phase10ParticleSnapshot {
                particle_id,
                system_id: binding.semantic_system_id.clone(),
                group_id: binding.semantic_id.clone(),
                position: vec2_bits(snapshot.position()),
                velocity: vec2_bits(snapshot.velocity()),
                flags_bits: snapshot.flags().bits(),
                color: snapshot.color().components(),
                weight_bits: FloatBits::from_f32(system_view.weights()[index]),
            });
        }
        if !systems.contains(&binding.system) {
            systems.push(binding.system);
        }
    }

    let topology = capture_system_records(executor, &particles, systems, record)?;
    let witnesses = evidence::capture_witnesses(
        executor,
        &particles,
        &topology.pairs,
        &topology.triads,
        &topology.body_contacts,
        record,
    )?;
    Ok(Phase10Observation::State {
        state: Phase10StateObservation {
            provenance,
            outcome: Phase10SemanticOutcome::Completed,
            groups: groups.into_boxed_slice(),
            particles: particles.into_boxed_slice(),
            pairs: topology.pairs.into_boxed_slice(),
            triads: topology.triads.into_boxed_slice(),
            particle_contacts: topology.particle_contacts.into_boxed_slice(),
            body_contacts: topology.body_contacts.into_boxed_slice(),
            events: executor.phase10.events.clone().into_boxed_slice(),
            witnesses: witnesses.into_boxed_slice(),
        },
    })
}

#[derive(Default)]
struct SystemRecords {
    pairs: Vec<Phase10PairSnapshot>,
    triads: Vec<Phase10TriadSnapshot>,
    particle_contacts: Vec<Phase10ParticleContact>,
    body_contacts: Vec<Phase10BodyContact>,
}

fn capture_system_records(
    executor: &TimelineExecutor,
    particles: &[Phase10ParticleSnapshot],
    systems: Vec<liquidfun::ParticleSystemId>,
    record: &RigidWorldActionRecord,
) -> Result<SystemRecords, NativeRigidWorldError> {
    let mut records = SystemRecords::default();
    for system in systems {
        let system_id = semantic_system_id(executor, system, record)?;
        let view = executor
            .world
            .particle_system_view(system)
            .map_err(|error| action_error(record, error))?;
        for pair in view.pairs() {
            let [particle_a, particle_b] = pair.particles();
            let Some((particle_a_id, particle_b_id)) =
                semantic_pair(executor, particles, system, particle_a, particle_b, record)?
            else {
                continue;
            };
            records.pairs.push(Phase10PairSnapshot {
                ordinal: checked_u32(records.pairs.len(), record)?,
                particle_a_id,
                particle_b_id,
                flags_bits: pair.flags().bits(),
                strength_bits: FloatBits::from_f32(pair.strength()),
                distance_bits: FloatBits::from_f32(pair.distance()),
            });
        }
        for triad in view.triads() {
            let [particle_a, particle_b, particle_c] = triad.particles();
            let ids = [particle_a, particle_b, particle_c]
                .map(|particle| semantic_particle_id(executor, system, particle, record));
            let [particle_a_id, particle_b_id, particle_c_id] = ids
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_ids: Vec<_>| action_error(record, "triad identity arity changed"))?;
            if ![&particle_a_id, &particle_b_id, &particle_c_id]
                .into_iter()
                .all(|id| particles.iter().any(|particle| &particle.particle_id == id))
            {
                continue;
            }
            records.triads.push(Phase10TriadSnapshot {
                ordinal: checked_u32(records.triads.len(), record)?,
                particle_a_id,
                particle_b_id,
                particle_c_id,
                flags_bits: triad.flags().bits(),
                strength_bits: FloatBits::from_f32(triad.strength()),
                pa: vec2_bits(triad.pa()),
                pb: vec2_bits(triad.pb()),
                pc: vec2_bits(triad.pc()),
                ka_bits: FloatBits::from_f32(triad.ka()),
                kb_bits: FloatBits::from_f32(triad.kb()),
                kc_bits: FloatBits::from_f32(triad.kc()),
                s_bits: FloatBits::from_f32(triad.s()),
            });
        }
        capture_contacts(
            executor,
            particles,
            system,
            &system_id,
            &view,
            &mut records,
            record,
        )?;
    }
    Ok(records)
}

fn capture_contacts(
    executor: &TimelineExecutor,
    particles: &[Phase10ParticleSnapshot],
    system: liquidfun::ParticleSystemId,
    system_id: &liquidfun_test_protocol::ScenarioId,
    view: &liquidfun::ParticleSystemView<'_>,
    records: &mut SystemRecords,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    for contact in view.particle_contacts() {
        let [particle_a, particle_b] = contact.particles();
        let Some((particle_a_id, particle_b_id)) =
            semantic_pair(executor, particles, system, particle_a, particle_b, record)?
        else {
            continue;
        };
        records.particle_contacts.push(Phase10ParticleContact {
            ordinal: checked_u32(records.particle_contacts.len(), record)?,
            system_id: system_id.clone(),
            particle_a_id,
            particle_b_id,
            flags_bits: contact.flags().bits(),
            weight_bits: FloatBits::from_f32(contact.weight()),
            normal: vec2_bits(contact.normal()),
        });
    }
    for contact in view.body_contacts() {
        let particle_id = semantic_particle_id(executor, system, contact.particle(), record)?;
        if !particles
            .iter()
            .any(|particle| particle.particle_id == particle_id)
        {
            continue;
        }
        records.body_contacts.push(Phase10BodyContact {
            ordinal: checked_u32(records.body_contacts.len(), record)?,
            system_id: system_id.clone(),
            particle_id,
            body_id: executor.semantic_body(contact.body())?,
            fixture_id: executor.semantic_fixture(contact.fixture())?,
            weight_bits: FloatBits::from_f32(contact.weight()),
            normal: vec2_bits(contact.normal()),
            mass_bits: FloatBits::from_f32(contact.mass()),
        });
    }
    Ok(())
}

fn semantic_pair(
    executor: &TimelineExecutor,
    particles: &[Phase10ParticleSnapshot],
    system: liquidfun::ParticleSystemId,
    particle_a: liquidfun::ParticleId,
    particle_b: liquidfun::ParticleId,
    record: &RigidWorldActionRecord,
) -> Result<
    Option<(
        liquidfun_test_protocol::ScenarioId,
        liquidfun_test_protocol::ScenarioId,
    )>,
    NativeRigidWorldError,
> {
    let particle_a_id = semantic_particle_id(executor, system, particle_a, record)?;
    let particle_b_id = semantic_particle_id(executor, system, particle_b, record)?;
    if [&particle_a_id, &particle_b_id]
        .into_iter()
        .all(|id| particles.iter().any(|particle| &particle.particle_id == id))
    {
        Ok(Some((particle_a_id, particle_b_id)))
    } else {
        Ok(None)
    }
}

fn timeline_provenance(timeline: &RigidWorldTimeline) -> Option<Phase10Provenance> {
    timeline.actions().iter().find_map(|record| {
        let RigidWorldAction::ParticleGroup {
            operation: liquidfun_test_protocol::Phase10Operation::CreateGroup { definition },
        } = record.action()
        else {
            return None;
        };
        Some(definition.provenance.clone())
    })
}
