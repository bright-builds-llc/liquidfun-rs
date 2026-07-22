//! Public-API-only execution for Phase 10 particle groups.

mod capture;
mod evidence;
mod recipe;

use liquidfun::{
    NoDecisionHook, ParticleGroupId, ParticleId, ParticleSystemId, StepConfiguration, StepLimits,
    World,
};
use liquidfun_test_protocol::{
    FloatBits, Phase10BehaviorLeaf, Phase10EventKind, Phase10GroupDefinition,
    Phase10GroupDestination, Phase10Operation, Phase10Provenance, Phase10WitnessObservation,
    RigidWorldAction, RigidWorldActionRecord, RigidWorldTimeline, ScenarioId, Vec2Bits,
    WitnessRole,
};

use super::super::{NativeRigidWorldError, TimelineExecutor};
use crate::rigid_world::model::{action_error, vec2_bits};

pub(crate) fn catalog_recipe(
    definition: &Phase10GroupDefinition,
    destination: liquidfun::particle::ParticleGroupDestination,
) -> Result<liquidfun::particle::ParticleGroupRecipe<()>, String> {
    recipe::recipe(definition, destination)
}

#[derive(Debug, Clone)]
struct GroupBinding {
    semantic_id: ScenarioId,
    semantic_system_id: ScenarioId,
    system: ParticleSystemId,
    group: ParticleGroupId,
}

/// Phase 10 semantic identity and evidence retained beside the shared world.
#[derive(Debug, Default)]
pub(in crate::rigid_world) struct NativePhase10State {
    groups: Vec<GroupBinding>,
    maybe_provenance: Option<Phase10Provenance>,
    events: Vec<liquidfun_test_protocol::Phase10Event>,
    witnesses: Vec<liquidfun_test_protocol::Phase10Witness>,
    velocity_witnesses: Vec<(ScenarioId, Vec2Bits, Vec2Bits)>,
}

impl NativePhase10State {
    pub(in crate::rigid_world) fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub(in crate::rigid_world) fn retain_live(&mut self, world: &World) {
        self.groups
            .retain(|binding| world.contains_particle_group(binding.group));
    }
}

pub(in crate::rigid_world) fn execute_action(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    record: &RigidWorldActionRecord,
) -> Result<bool, NativeRigidWorldError> {
    let RigidWorldAction::ParticleGroup { operation } = record.action() else {
        return Ok(false);
    };
    match operation {
        Phase10Operation::CreateGroup { definition } => {
            create_group(executor, definition, record)?;
        }
        Phase10Operation::JoinGroups {
            target_group_id,
            source_group_id,
        } => join_groups(executor, target_group_id, source_group_id, record)?,
        Phase10Operation::SplitGroup {
            group_id,
            created_group_ids,
        } => split_group(executor, group_id, created_group_ids, record)?,
        Phase10Operation::SetGroupFlags {
            group_id,
            group_flags_bits,
        } => set_group_flags(executor, group_id, *group_flags_bits, record)?,
        Phase10Operation::DestroyGroup { group_id } => {
            destroy_group(executor, group_id, record)?;
        }
        Phase10Operation::Step {
            timestep_bits,
            velocity_iterations,
            position_iterations,
            particle_iterations,
        } => step(
            executor,
            *timestep_bits,
            *velocity_iterations,
            *position_iterations,
            *particle_iterations,
            record,
        )?,
        Phase10Operation::InspectState => capture::inspect(executor, timeline, record)?,
    }
    Ok(true)
}

fn create_group(
    executor: &mut TimelineExecutor,
    definition: &Phase10GroupDefinition,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let system = system(executor, &definition.system_id, record)?;
    let destination = match &definition.destination {
        Phase10GroupDestination::New => liquidfun::particle::ParticleGroupDestination::New,
        Phase10GroupDestination::AppendTo { target_group_id } => {
            let target = group(executor, target_group_id, record)?;
            if target.system != system {
                return Err(action_error(
                    record,
                    "append target belongs to another particle system",
                ));
            }
            liquidfun::particle::ParticleGroupDestination::AppendTo(target.group)
        }
    };
    let recipe =
        recipe::recipe(definition, destination).map_err(|message| action_error(record, message))?;
    let prior_members = match destination {
        liquidfun::particle::ParticleGroupDestination::New => 0,
        liquidfun::particle::ParticleGroupDestination::AppendTo(target) => executor
            .world
            .particle_group_view(target)
            .map_err(|error| action_error(record, error))?
            .member_count(),
    };
    let created_group = executor
        .world
        .create_particle_group(system, &recipe)
        .map_err(|error| action_error(record, error))?;
    let member_ids = executor
        .world
        .particle_group_view(created_group)
        .map_err(|error| action_error(record, error))?
        .member_ids()
        .to_vec();
    let created_members = member_ids
        .get(prior_members..)
        .ok_or_else(|| action_error(record, "group append shortened the target member range"))?;
    if created_members.len() != definition.member_ids.len() {
        return Err(action_error(
            record,
            format!(
                "group source produced {} particles for {} semantic member IDs",
                created_members.len(),
                definition.member_ids.len()
            ),
        ));
    }
    for (semantic_id, particle) in definition.member_ids.iter().cloned().zip(created_members) {
        executor.particles.push((semantic_id, system, *particle));
    }
    executor.phase10.maybe_provenance = Some(definition.provenance.clone());
    match destination {
        liquidfun::particle::ParticleGroupDestination::New => {
            executor.phase10.groups.push(GroupBinding {
                semantic_id: definition.group_id.clone(),
                semantic_system_id: definition.system_id.clone(),
                system,
                group: created_group,
            });
            let event_ordinal = evidence::push_event(
                executor,
                Phase10EventKind::GroupCreated,
                definition.system_id.clone(),
                Some(definition.group_id.clone()),
                None,
                None,
                None,
                record,
            )?;
            evidence::upsert_witness(
                &mut executor.phase10.witnesses,
                Phase10BehaviorLeaf::GroupCreate,
                WitnessRole::Activation,
                Phase10WitnessObservation::Occurrence { event_ordinal },
                record,
            )?;
        }
        liquidfun::particle::ParticleGroupDestination::AppendTo(_) => {
            evidence::upsert_witness(
                &mut executor.phase10.witnesses,
                Phase10BehaviorLeaf::GroupAppend,
                WitnessRole::Activation,
                Phase10WitnessObservation::Count {
                    value: checked_u32(created_members.len(), record)?,
                },
                record,
            )?;
        }
    }
    Ok(())
}

fn join_groups(
    executor: &mut TimelineExecutor,
    target_group_id: &ScenarioId,
    source_group_id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let target = group(executor, target_group_id, record)?;
    let source = group(executor, source_group_id, record)?;
    if target.system != source.system {
        return Err(action_error(
            record,
            "join operands belong to different particle systems",
        ));
    }
    let report = executor
        .world
        .join_particle_groups(target.group, source.group)
        .map_err(|error| action_error(record, error))?;
    if *report.value() != target.group {
        return Err(action_error(
            record,
            "join did not preserve the target group identity",
        ));
    }
    executor
        .phase10
        .groups
        .retain(|binding| binding.group != source.group);
    let event_ordinal = evidence::push_event(
        executor,
        Phase10EventKind::GroupsJoined,
        target.semantic_system_id,
        Some(target_group_id.clone()),
        None,
        None,
        None,
        record,
    )?;
    evidence::upsert_witness(
        &mut executor.phase10.witnesses,
        Phase10BehaviorLeaf::GroupJoin,
        WitnessRole::Activation,
        Phase10WitnessObservation::Occurrence { event_ordinal },
        record,
    )
}

fn split_group(
    executor: &mut TimelineExecutor,
    group_id: &ScenarioId,
    created_group_ids: &[ScenarioId],
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let source = group(executor, group_id, record)?;
    let result = executor
        .world
        .split_particle_group(source.group)
        .map_err(|error| action_error(record, error))?;
    let Some((retained, created)) = result.split_first() else {
        return Err(action_error(
            record,
            "split returned no retained group identity",
        ));
    };
    if *retained != source.group || created.len() != created_group_ids.len() {
        return Err(action_error(
            record,
            format!(
                "split produced {} new groups for {} semantic identities",
                created.len(),
                created_group_ids.len()
            ),
        ));
    }
    executor
        .phase10
        .groups
        .extend(
            created_group_ids
                .iter()
                .cloned()
                .zip(created)
                .map(|(semantic_id, group)| GroupBinding {
                    semantic_id,
                    semantic_system_id: source.semantic_system_id.clone(),
                    system: source.system,
                    group: *group,
                }),
        );
    let event_ordinal = evidence::push_event(
        executor,
        Phase10EventKind::GroupSplit,
        source.semantic_system_id,
        Some(group_id.clone()),
        None,
        None,
        None,
        record,
    )?;
    evidence::upsert_witness(
        &mut executor.phase10.witnesses,
        Phase10BehaviorLeaf::GroupSplit,
        WitnessRole::Activation,
        Phase10WitnessObservation::Occurrence { event_ordinal },
        record,
    )
}

fn set_group_flags(
    executor: &mut TimelineExecutor,
    group_id: &ScenarioId,
    flags_bits: u32,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let binding = group(executor, group_id, record)?;
    executor
        .world
        .set_particle_group_flags(
            binding.group,
            liquidfun::particle::ParticleGroupFlags::from_bits_retain(flags_bits),
        )
        .map_err(|error| action_error(record, error))?;
    evidence::upsert_witness(
        &mut executor.phase10.witnesses,
        Phase10BehaviorLeaf::GroupFlags,
        WitnessRole::Activation,
        Phase10WitnessObservation::Count { value: flags_bits },
        record,
    )
}

fn destroy_group(
    executor: &mut TimelineExecutor,
    group_id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let binding = group(executor, group_id, record)?;
    let member_count = executor
        .world
        .particle_group_view(binding.group)
        .map_err(|error| action_error(record, error))?
        .member_count();
    if member_count == 0 {
        executor
            .world
            .destroy_empty_particle_group(binding.group)
            .map_err(|error| action_error(record, error))?;
        executor
            .phase10
            .groups
            .retain(|candidate| candidate.group != binding.group);
        let event_ordinal = evidence::push_event(
            executor,
            Phase10EventKind::GroupDestroyed,
            binding.semantic_system_id,
            Some(group_id.clone()),
            None,
            None,
            None,
            record,
        )?;
        evidence::upsert_witness(
            &mut executor.phase10.witnesses,
            Phase10BehaviorLeaf::GroupDestroy,
            WitnessRole::Activation,
            Phase10WitnessObservation::Occurrence { event_ordinal },
            record,
        )?;
    } else {
        executor
            .world
            .destroy_particle_group_particles(binding.group, true)
            .map_err(|error| action_error(record, error))?;
        evidence::upsert_witness(
            &mut executor.phase10.witnesses,
            Phase10BehaviorLeaf::GroupDestroy,
            WitnessRole::Activation,
            Phase10WitnessObservation::Count {
                value: checked_u32(member_count, record)?,
            },
            record,
        )?;
    }
    Ok(())
}

fn step(
    executor: &mut TimelineExecutor,
    timestep_bits: FloatBits,
    velocity_iterations: u32,
    position_iterations: u32,
    particle_iterations: u32,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let before = executor
        .particles
        .iter()
        .filter_map(|(id, _, particle)| {
            executor
                .world
                .particle_snapshot(*particle)
                .ok()
                .map(|snapshot| (id.clone(), vec2_bits(snapshot.velocity())))
        })
        .collect::<Vec<_>>();
    let configuration = StepConfiguration::new(
        timestep_bits.to_f32(),
        velocity_iterations,
        position_iterations,
    )
    .and_then(|configuration| configuration.with_particle_iterations(particle_iterations))
    .map_err(|error| action_error(record, error))?;
    let report = executor
        .world
        .step(configuration, &mut NoDecisionHook, StepLimits::default())
        .map_err(|error| action_error(record, error))?;
    evidence::collect_step_events(executor, report.lifecycle(), record)?;
    executor.phase10.velocity_witnesses = before
        .into_iter()
        .filter_map(|(id, before)| {
            let (_, _, particle) = executor
                .particles
                .iter()
                .find(|(candidate, _, _)| candidate == &id)?;
            let after = executor.world.particle_snapshot(*particle).ok()?;
            Some((id, before, vec2_bits(after.velocity())))
        })
        .collect();
    executor
        .particles
        .retain(|(_, _, particle)| executor.world.contains_particle(*particle));
    executor.phase10.retain_live(&executor.world);
    Ok(())
}

fn group(
    executor: &TimelineExecutor,
    id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<GroupBinding, NativeRigidWorldError> {
    executor
        .phase10
        .groups
        .iter()
        .find(|binding| &binding.semantic_id == id)
        .cloned()
        .ok_or_else(|| action_error(record, format!("unknown particle group `{id}`")))
}

fn system(
    executor: &TimelineExecutor,
    id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<ParticleSystemId, NativeRigidWorldError> {
    executor
        .particle_systems
        .iter()
        .find_map(|(candidate, system)| (candidate == id).then_some(*system))
        .ok_or_else(|| action_error(record, format!("unknown particle system `{id}`")))
}

fn semantic_system_id(
    executor: &TimelineExecutor,
    system: ParticleSystemId,
    record: &RigidWorldActionRecord,
) -> Result<ScenarioId, NativeRigidWorldError> {
    executor
        .particle_systems
        .iter()
        .find_map(|(id, candidate)| (*candidate == system).then(|| id.clone()))
        .ok_or_else(|| action_error(record, "particle system has no semantic identity"))
}

fn semantic_particle_id(
    executor: &TimelineExecutor,
    system: ParticleSystemId,
    particle: ParticleId,
    record: &RigidWorldActionRecord,
) -> Result<ScenarioId, NativeRigidWorldError> {
    executor
        .particles
        .iter()
        .find(|(_, candidate_system, candidate)| {
            *candidate_system == system && *candidate == particle
        })
        .map(|(id, _, _)| id.clone())
        .ok_or_else(|| action_error(record, "particle has no semantic identity"))
}

fn checked_u32(
    value: usize,
    record: &RigidWorldActionRecord,
) -> Result<u32, NativeRigidWorldError> {
    u32::try_from(value).map_err(|error| action_error(record, error))
}
