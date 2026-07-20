use std::ops::Range;

use crate::identity::ParticleGroupId;
use crate::math::{Transform, Vec2};
use crate::particle::storage::{ParticleStorage, ParticleStorageError};

use super::GroupStatisticsCache;

#[derive(Debug, Clone, PartialEq)]
pub(in crate::particle) struct RigidGroupState {
    pub(in crate::particle) range: Range<usize>,
    pub(in crate::particle) transform: Transform,
    pub(in crate::particle) statistics: GroupStatisticsCache,
}

impl ParticleStorage {
    pub(in crate::particle) fn update_group_statistics(
        &mut self,
        group: ParticleGroupId,
        particle_mass: f32,
        timestamp: u32,
    ) -> Result<GroupStatisticsCache, ParticleStorageError> {
        self.check_invariants()?;
        if !particle_mass.is_finite() || particle_mass <= 0.0 {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        let record_index = self
            .group_records
            .iter()
            .position(|record| record.id == group && record.system == self.system)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if self.group_records[record_index]
            .statistics
            .maybe_source_timestamp
            == Some(timestamp)
        {
            return Ok(self.group_records[record_index].statistics);
        }
        let range = self.group_records[record_index].range();
        let statistics = compute_statistics_candidate(
            &self.positions[range.clone()],
            &self.velocities[range],
            particle_mass,
            timestamp,
        )?;
        self.group_records[record_index].statistics = statistics;
        Ok(statistics)
    }

    pub(in crate::particle) fn rigid_group_state(
        &mut self,
        group: ParticleGroupId,
        particle_mass: f32,
        timestamp: u32,
    ) -> Result<RigidGroupState, ParticleStorageError> {
        let statistics = self.update_group_statistics(group, particle_mass, timestamp)?;
        let record = self
            .group_records
            .iter()
            .find(|record| record.id == group && record.system == self.system)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        Ok(RigidGroupState {
            range: record.range(),
            transform: record.transform,
            statistics,
        })
    }
}

fn compute_statistics_candidate(
    positions: &[Vec2],
    velocities: &[Vec2],
    particle_mass: f32,
    timestamp: u32,
) -> Result<GroupStatisticsCache, ParticleStorageError> {
    if positions.len() != velocities.len() {
        return Err(ParticleStorageError::LaneLengthMismatch);
    }
    let mut statistics = GroupStatisticsCache {
        maybe_source_timestamp: Some(timestamp),
        ..GroupStatisticsCache::INVALIDATED_ZERO
    };
    for (position, velocity) in positions.iter().copied().zip(velocities.iter().copied()) {
        statistics.mass += particle_mass;
        statistics.center += particle_mass * position;
        statistics.linear_velocity += particle_mass * velocity;
        if !statistics.is_finite() {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
    }
    if statistics.mass > 0.0 {
        let inverse_mass = 1.0 / statistics.mass;
        statistics.center *= inverse_mass;
        statistics.linear_velocity *= inverse_mass;
    }
    for (position, velocity) in positions.iter().copied().zip(velocities.iter().copied()) {
        let relative_position = position - statistics.center;
        let relative_velocity = velocity - statistics.linear_velocity;
        statistics.inertia += particle_mass * relative_position.dot(relative_position);
        statistics.angular_velocity += particle_mass * relative_position.cross(relative_velocity);
        if !statistics.is_finite() {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
    }
    if statistics.inertia > 0.0 {
        statistics.angular_velocity *= 1.0 / statistics.inertia;
    }
    if !statistics.is_finite() {
        return Err(ParticleStorageError::InvalidLaneBundle);
    }
    Ok(statistics)
}
