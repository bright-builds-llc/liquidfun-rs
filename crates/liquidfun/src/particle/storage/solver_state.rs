use crate::math::Vec2;
use crate::particle::{ParticleFlags, ParticleGroupFlags};

use super::ParticleStorageError;
use super::group::{GroupRecord, InternalGroupFlags};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::particle) struct AggregateGroupFlags {
    pub(in crate::particle) public: ParticleGroupFlags,
    pub(in crate::particle) internal: InternalGroupFlags,
}

impl AggregateGroupFlags {
    const EMPTY: Self = Self {
        public: ParticleGroupFlags::empty(),
        internal: InternalGroupFlags::empty(),
    };
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SolverState {
    aggregate_particle_flags: ParticleFlags,
    particle_flags_dirty: bool,
    aggregate_group_flags: AggregateGroupFlags,
    group_flags_dirty: bool,
    pending_system_force: bool,
    maybe_static_pressures: Option<Vec<f32>>,
    maybe_tensile_accumulations: Option<Vec<Vec2>>,
    maybe_depths: Option<Vec<f32>>,
}

impl SolverState {
    pub(super) const fn new() -> Self {
        Self {
            aggregate_particle_flags: ParticleFlags::WATER,
            particle_flags_dirty: false,
            aggregate_group_flags: AggregateGroupFlags::EMPTY,
            group_flags_dirty: false,
            pending_system_force: false,
            maybe_static_pressures: None,
            maybe_tensile_accumulations: None,
            maybe_depths: None,
        }
    }

    pub(super) fn prepare_append(
        &self,
        existing_particle_flags: &[ParticleFlags],
        appended_flags: ParticleFlags,
        group_records: &[GroupRecord],
        declared_capacity: usize,
    ) -> Result<Self, ParticleStorageError> {
        self.validate_scratch_lanes(existing_particle_flags.len())?;
        let new_count = existing_particle_flags
            .len()
            .checked_add(1)
            .ok_or(ParticleStorageError::InvalidLaneBundle)?;
        preflight_count(new_count, declared_capacity)?;
        let maybe_static_pressures = clone_and_append(
            self.maybe_static_pressures.as_deref(),
            0.0,
            declared_capacity,
        )?;
        let maybe_tensile_accumulations = clone_and_append(
            self.maybe_tensile_accumulations.as_deref(),
            Vec2::ZERO,
            declared_capacity,
        )?;
        let maybe_depths = clone_and_append(self.maybe_depths.as_deref(), 0.0, declared_capacity)?;
        let aggregate_particle_flags = existing_particle_flags
            .iter()
            .copied()
            .chain(std::iter::once(appended_flags))
            .fold(ParticleFlags::WATER, |aggregate, flags| aggregate | flags);

        Ok(Self {
            aggregate_particle_flags,
            particle_flags_dirty: false,
            aggregate_group_flags: scan_group_flags(group_records),
            group_flags_dirty: false,
            pending_system_force: self.pending_system_force,
            maybe_static_pressures,
            maybe_tensile_accumulations,
            maybe_depths,
        })
    }

    pub(super) fn prepare_permutation(
        &self,
        old_to_new: &[Option<usize>],
        new_count: usize,
        particle_flags: &[ParticleFlags],
        group_records: &[GroupRecord],
        declared_capacity: usize,
    ) -> Result<Self, ParticleStorageError> {
        preflight_count(new_count, declared_capacity)?;
        Ok(Self {
            aggregate_particle_flags: scan_particle_flags(particle_flags),
            particle_flags_dirty: false,
            aggregate_group_flags: scan_group_flags(group_records),
            group_flags_dirty: false,
            pending_system_force: self.pending_system_force,
            maybe_static_pressures: permute_optional_lane(
                self.maybe_static_pressures.as_deref(),
                old_to_new,
                new_count,
                declared_capacity,
                0.0,
            )?,
            maybe_tensile_accumulations: permute_optional_lane(
                self.maybe_tensile_accumulations.as_deref(),
                old_to_new,
                new_count,
                declared_capacity,
                Vec2::ZERO,
            )?,
            maybe_depths: permute_optional_lane(
                self.maybe_depths.as_deref(),
                old_to_new,
                new_count,
                declared_capacity,
                0.0,
            )?,
        })
    }

    pub(super) fn validate(
        &self,
        particle_count: usize,
        particle_flags: &[ParticleFlags],
        group_records: &[GroupRecord],
    ) -> Result<(), ParticleStorageError> {
        self.validate_scratch_lanes(particle_count)?;
        let particle_aggregate_valid = self.particle_flags_dirty
            || self.aggregate_particle_flags == scan_particle_flags(particle_flags);
        let group_aggregate_valid =
            self.group_flags_dirty || self.aggregate_group_flags == scan_group_flags(group_records);
        if !particle_aggregate_valid || !group_aggregate_valid {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        Ok(())
    }

    fn validate_scratch_lanes(&self, particle_count: usize) -> Result<(), ParticleStorageError> {
        let lengths_match = [
            self.maybe_static_pressures.as_ref().map(Vec::len),
            self.maybe_tensile_accumulations.as_ref().map(Vec::len),
            self.maybe_depths.as_ref().map(Vec::len),
        ]
        .into_iter()
        .flatten()
        .all(|lane_count| lane_count == particle_count);
        if !lengths_match {
            return Err(ParticleStorageError::LaneLengthMismatch);
        }
        let values_are_finite = self
            .maybe_static_pressures
            .as_deref()
            .is_none_or(|lane| lane.iter().all(|value| value.is_finite()))
            && self
                .maybe_tensile_accumulations
                .as_deref()
                .is_none_or(|lane| lane.iter().all(|value| value.is_valid()))
            && self
                .maybe_depths
                .as_deref()
                .is_none_or(|lane| lane.iter().all(|value| value.is_finite()));
        if !values_are_finite {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        Ok(())
    }

    pub(super) fn mark_particle_flags_dirty(&mut self) {
        self.particle_flags_dirty = true;
    }

    pub(super) fn mark_group_flags_dirty(&mut self) {
        self.group_flags_dirty = true;
    }

    pub(super) fn refresh_particle_flags(&mut self, particle_flags: &[ParticleFlags]) {
        self.aggregate_particle_flags = scan_particle_flags(particle_flags);
        self.particle_flags_dirty = false;
    }

    pub(super) fn refresh_group_flags(&mut self, group_records: &[GroupRecord]) {
        self.aggregate_group_flags = scan_group_flags(group_records);
        self.group_flags_dirty = false;
    }

    pub(super) const fn aggregate_particle_flags(&self) -> ParticleFlags {
        self.aggregate_particle_flags
    }

    pub(super) const fn aggregate_group_flags(&self) -> AggregateGroupFlags {
        self.aggregate_group_flags
    }

    pub(super) fn ensure_static_pressures(
        &mut self,
        particle_count: usize,
        declared_capacity: usize,
    ) -> Result<(), ParticleStorageError> {
        ensure_optional_lane(
            &mut self.maybe_static_pressures,
            particle_count,
            declared_capacity,
            0.0,
        )
    }

    pub(super) fn ensure_tensile_accumulations(
        &mut self,
        particle_count: usize,
        declared_capacity: usize,
    ) -> Result<(), ParticleStorageError> {
        ensure_optional_lane(
            &mut self.maybe_tensile_accumulations,
            particle_count,
            declared_capacity,
            Vec2::ZERO,
        )
    }

    pub(super) fn ensure_depths(
        &mut self,
        particle_count: usize,
        declared_capacity: usize,
    ) -> Result<(), ParticleStorageError> {
        ensure_optional_lane(
            &mut self.maybe_depths,
            particle_count,
            declared_capacity,
            0.0,
        )
    }

    pub(super) fn replace_static_pressures(
        &mut self,
        candidate: Vec<f32>,
        particle_count: usize,
    ) -> Result<(), ParticleStorageError> {
        replace_optional_lane(
            &mut self.maybe_static_pressures,
            candidate,
            particle_count,
            |value| value.is_finite(),
        )
    }

    pub(super) fn replace_tensile_accumulations(
        &mut self,
        candidate: Vec<Vec2>,
        particle_count: usize,
    ) -> Result<(), ParticleStorageError> {
        replace_optional_lane(
            &mut self.maybe_tensile_accumulations,
            candidate,
            particle_count,
            |value| value.is_valid(),
        )
    }

    pub(super) fn replace_depths(
        &mut self,
        candidate: Vec<f32>,
        particle_count: usize,
    ) -> Result<(), ParticleStorageError> {
        replace_optional_lane(&mut self.maybe_depths, candidate, particle_count, |value| {
            value.is_finite()
        })
    }

    pub(super) fn maybe_static_pressures(&self) -> Option<&[f32]> {
        self.maybe_static_pressures.as_deref()
    }

    pub(super) fn maybe_tensile_accumulations(&self) -> Option<&[Vec2]> {
        self.maybe_tensile_accumulations.as_deref()
    }

    pub(super) fn maybe_depths(&self) -> Option<&[f32]> {
        self.maybe_depths.as_deref()
    }

    pub(super) const fn has_pending_system_force(&self) -> bool {
        self.pending_system_force
    }

    pub(super) fn mark_pending_system_force(&mut self) {
        self.pending_system_force = true;
    }

    pub(super) fn clear_pending_system_force(&mut self) {
        self.pending_system_force = false;
    }
}

fn preflight_count(
    particle_count: usize,
    declared_capacity: usize,
) -> Result<(), ParticleStorageError> {
    if particle_count > declared_capacity || particle_count > i32::MAX as usize {
        return Err(ParticleStorageError::InvalidLaneBundle);
    }
    Ok(())
}

fn zeroed_lane<T: Copy>(
    particle_count: usize,
    declared_capacity: usize,
    zero: T,
) -> Result<Vec<T>, ParticleStorageError> {
    preflight_count(particle_count, declared_capacity)?;
    let mut candidate = Vec::new();
    candidate
        .try_reserve_exact(declared_capacity)
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    candidate.resize(particle_count, zero);
    Ok(candidate)
}

fn clone_and_append<T: Copy>(
    maybe_lane: Option<&[T]>,
    zero: T,
    declared_capacity: usize,
) -> Result<Option<Vec<T>>, ParticleStorageError> {
    let Some(lane) = maybe_lane else {
        return Ok(None);
    };
    let new_count = lane
        .len()
        .checked_add(1)
        .ok_or(ParticleStorageError::InvalidLaneBundle)?;
    let mut candidate = zeroed_lane(new_count, declared_capacity, zero)?;
    candidate[..lane.len()].copy_from_slice(lane);
    Ok(Some(candidate))
}

fn permute_optional_lane<T: Copy>(
    maybe_lane: Option<&[T]>,
    old_to_new: &[Option<usize>],
    new_count: usize,
    declared_capacity: usize,
    zero: T,
) -> Result<Option<Vec<T>>, ParticleStorageError> {
    let Some(lane) = maybe_lane else {
        return Ok(None);
    };
    if lane.len() != old_to_new.len() {
        return Err(ParticleStorageError::LaneLengthMismatch);
    }
    let mut candidate = zeroed_lane(new_count, declared_capacity, zero)?;
    for (old, maybe_new) in old_to_new.iter().copied().enumerate() {
        if let Some(new) = maybe_new {
            candidate[new] = lane[old];
        }
    }
    Ok(Some(candidate))
}

fn ensure_optional_lane<T: Copy>(
    maybe_lane: &mut Option<Vec<T>>,
    particle_count: usize,
    declared_capacity: usize,
    zero: T,
) -> Result<(), ParticleStorageError> {
    if maybe_lane.is_some() {
        return Ok(());
    }
    let candidate = zeroed_lane(particle_count, declared_capacity, zero)?;
    *maybe_lane = Some(candidate);
    Ok(())
}

fn replace_optional_lane<T>(
    maybe_lane: &mut Option<Vec<T>>,
    candidate: Vec<T>,
    particle_count: usize,
    is_valid: impl Fn(&T) -> bool,
) -> Result<(), ParticleStorageError> {
    if maybe_lane.is_none()
        || candidate.len() != particle_count
        || candidate.iter().any(|value| !is_valid(value))
    {
        return Err(ParticleStorageError::InvalidLaneBundle);
    }
    *maybe_lane = Some(candidate);
    Ok(())
}

fn scan_particle_flags(particle_flags: &[ParticleFlags]) -> ParticleFlags {
    particle_flags
        .iter()
        .copied()
        .fold(ParticleFlags::WATER, |aggregate, flags| aggregate | flags)
}

fn scan_group_flags(group_records: &[GroupRecord]) -> AggregateGroupFlags {
    group_records
        .iter()
        .fold(AggregateGroupFlags::EMPTY, |mut aggregate, record| {
            aggregate.public |= record.flags;
            aggregate.internal.insert(record.internal_flags);
            aggregate
        })
}

#[cfg(test)]
#[path = "solver_state/tests.rs"]
mod tests;
