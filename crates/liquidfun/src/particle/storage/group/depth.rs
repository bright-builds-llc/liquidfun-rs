use crate::identity::ParticleGroupId;
use crate::particle::storage::lanes::ParticleContact;
use crate::particle::storage::{ParticleStorage, ParticleStorageError};

use super::{GroupRecord, InternalGroupFlags};

impl ParticleStorage {
    pub(in crate::particle) fn compute_solid_depth(
        &mut self,
        particle_diameter: f32,
    ) -> Result<(), ParticleStorageError> {
        self.check_invariants()?;
        if !particle_diameter.is_finite() || particle_diameter <= 0.0 {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        let scheduled = self
            .group_records
            .iter()
            .filter(|record| {
                record
                    .internal_flags
                    .contains(InternalGroupFlags::NEEDS_UPDATE_DEPTH)
            })
            .map(|record| record.id)
            .collect::<Vec<_>>();
        if scheduled.is_empty() {
            return Ok(());
        }

        let mut candidate = self.clone();
        candidate.ensure_depths()?;
        let depths = compute_depth_candidate(
            candidate.len(),
            particle_diameter,
            &candidate.groups,
            &candidate.group_records,
            &candidate.particle_contacts,
            candidate.solver_state.maybe_depths(),
        )?;
        candidate.replace_depths(depths)?;
        for record in &mut candidate.group_records {
            if scheduled.contains(&record.id) {
                record
                    .internal_flags
                    .remove(InternalGroupFlags::NEEDS_UPDATE_DEPTH);
            }
        }
        candidate
            .solver_state
            .refresh_group_flags(&candidate.group_records);
        candidate.check_invariants()?;
        *self = candidate;
        Ok(())
    }
}

fn compute_depth_candidate(
    particle_count: usize,
    particle_diameter: f32,
    groups: &[Option<ParticleGroupId>],
    group_records: &[GroupRecord],
    contacts: &[ParticleContact],
    maybe_existing_depths: Option<&[f32]>,
) -> Result<Vec<f32>, ParticleStorageError> {
    if groups.len() != particle_count
        || maybe_existing_depths.is_some_and(|depths| depths.len() != particle_count)
    {
        return Err(ParticleStorageError::LaneLengthMismatch);
    }
    let (mut depths, accumulation) = initialize_depth_candidate(
        particle_count,
        groups,
        group_records,
        contacts,
        maybe_existing_depths,
    )?;
    initialize_scheduled_depths(&mut depths, &accumulation, group_records);
    relax_depths(&mut depths, particle_count, groups, group_records, contacts)?;
    scale_scheduled_depths(&mut depths, particle_diameter, group_records)?;
    Ok(depths)
}

fn initialize_depth_candidate(
    particle_count: usize,
    groups: &[Option<ParticleGroupId>],
    group_records: &[GroupRecord],
    contacts: &[ParticleContact],
    maybe_existing_depths: Option<&[f32]>,
) -> Result<(Vec<f32>, Vec<f32>), ParticleStorageError> {
    let mut depths = Vec::new();
    depths
        .try_reserve_exact(particle_count)
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    if let Some(existing_depths) = maybe_existing_depths {
        depths.extend_from_slice(existing_depths);
    } else {
        depths.resize(particle_count, 0.0);
    }
    let mut accumulation = Vec::new();
    accumulation
        .try_reserve_exact(particle_count)
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    accumulation.resize(particle_count, 0.0_f32);

    for contact in contacts {
        let [a, b] = contact.indices;
        if a.0 >= particle_count
            || b.0 >= particle_count
            || !contact.weight.is_finite()
            || !(0.0..=1.0).contains(&contact.weight)
        {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        let Some(group) = groups[a.0] else {
            continue;
        };
        if groups[b.0] != Some(group) || !needs_update_depth(group_records, group) {
            continue;
        }
        accumulation[a.0] += contact.weight;
        accumulation[b.0] += contact.weight;
        if !accumulation[a.0].is_finite() || !accumulation[b.0].is_finite() {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
    }
    Ok((depths, accumulation))
}

fn initialize_scheduled_depths(
    depths: &mut [f32],
    accumulation: &[f32],
    group_records: &[GroupRecord],
) {
    for record in group_records {
        if !record
            .internal_flags
            .contains(InternalGroupFlags::NEEDS_UPDATE_DEPTH)
        {
            continue;
        }
        for dense in record.range() {
            depths[dense] = if accumulation[dense] < 0.8 {
                0.0
            } else {
                f32::MAX
            };
        }
    }
}

fn relax_depths(
    depths: &mut [f32],
    particle_count: usize,
    groups: &[Option<ParticleGroupId>],
    group_records: &[GroupRecord],
    contacts: &[ParticleContact],
) -> Result<(), ParticleStorageError> {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the pinned C++ depth iteration count converts the bounded particle count to float32"
    )]
    let iteration_count = (particle_count as f32).sqrt() as usize;
    for _iteration in 0..iteration_count {
        let mut updated = false;
        for contact in contacts {
            let [a, b] = contact.indices;
            let Some(group) = groups[a.0] else {
                continue;
            };
            if groups[b.0] != Some(group) || !needs_update_depth(group_records, group) {
                continue;
            }
            let distance = 1.0 - contact.weight;
            let a_next = finite_depth_step(depths[b.0], distance)?;
            let b_next = finite_depth_step(depths[a.0], distance)?;
            if depths[a.0] > a_next {
                depths[a.0] = a_next;
                updated = true;
            }
            if depths[b.0] > b_next {
                depths[b.0] = b_next;
                updated = true;
            }
        }
        if !updated {
            break;
        }
    }
    Ok(())
}

fn scale_scheduled_depths(
    depths: &mut [f32],
    particle_diameter: f32,
    group_records: &[GroupRecord],
) -> Result<(), ParticleStorageError> {
    for record in group_records {
        if !record
            .internal_flags
            .contains(InternalGroupFlags::NEEDS_UPDATE_DEPTH)
        {
            continue;
        }
        for dense in record.range() {
            depths[dense] = if depths[dense] < f32::MAX {
                let scaled = depths[dense] * particle_diameter;
                if !scaled.is_finite() {
                    return Err(ParticleStorageError::InvalidLaneBundle);
                }
                scaled
            } else {
                0.0
            };
        }
    }
    Ok(())
}

fn needs_update_depth(group_records: &[GroupRecord], group: ParticleGroupId) -> bool {
    group_records.iter().any(|record| {
        record.id == group
            && record
                .internal_flags
                .contains(InternalGroupFlags::NEEDS_UPDATE_DEPTH)
    })
}

fn finite_depth_step(depth: f32, distance: f32) -> Result<f32, ParticleStorageError> {
    if depth.to_bits() == f32::MAX.to_bits() {
        return Ok(f32::MAX);
    }
    let candidate = depth + distance;
    if !candidate.is_finite() {
        return Err(ParticleStorageError::InvalidLaneBundle);
    }
    Ok(candidate)
}
