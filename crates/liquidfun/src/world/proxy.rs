use crate::collision::{Aabb, BroadPhase, ChildIndex, FilterData, ProxyId, Shape};
use crate::math::settings::{AABB_EXTENSION, AABB_MULTIPLIER};
use crate::math::{Transform, Vec2};
use crate::{BodyId, FixtureId};

use super::fixture::FixtureBoundsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FixtureProxy {
    pub(super) fixture: FixtureId,
    pub(super) body: BodyId,
    pub(super) child_index: ChildIndex,
}

#[derive(Debug, Clone, Copy)]
struct FixtureProxyEntry {
    id: ProxyId,
    child_index: ChildIndex,
    aabb: Aabb,
}

#[derive(Debug, Clone, Default)]
pub(super) struct FixtureProxies {
    entries: Vec<FixtureProxyEntry>,
}

pub(super) struct PreparedFixtureBounds {
    children: Vec<(ChildIndex, Aabb)>,
}

pub(super) struct PreparedSynchronization {
    children: Vec<(ChildIndex, Aabb)>,
    displacement: Vec2,
}

pub(super) struct PreparedProxyOriginShift {
    children: Vec<(ChildIndex, Aabb)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProxyOriginShiftError {
    InconsistentProxy,
    NonFiniteBounds,
}

impl FixtureProxies {
    pub(super) const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn maybe_proxy_id(&self, child_index: ChildIndex) -> Option<ProxyId> {
        self.entries
            .iter()
            .find(|entry| entry.child_index == child_index)
            .map(|entry| entry.id)
    }

    pub(super) fn prepare_origin_shift(
        &self,
        broad_phase: &BroadPhase<FixtureProxy>,
        fixture: FixtureId,
        body: BodyId,
        shape: &Shape,
        active: bool,
        shift: Vec2,
    ) -> Result<PreparedProxyOriginShift, ProxyOriginShiftError> {
        let expected_children = if active { shape.child_count() } else { 0 };
        if self.entries.len() != expected_children {
            return Err(ProxyOriginShiftError::InconsistentProxy);
        }

        let mut children = Vec::with_capacity(self.entries.len());
        for (expected_index, entry) in self.entries.iter().enumerate() {
            let expected_child = shape
                .child_index(expected_index)
                .map_err(|_error| ProxyOriginShiftError::InconsistentProxy)?;
            if entry.child_index != expected_child
                || self.entries[..expected_index]
                    .iter()
                    .any(|prior| prior.id == entry.id)
            {
                return Err(ProxyOriginShiftError::InconsistentProxy);
            }
            let payload = broad_phase
                .payload(entry.id)
                .map_err(|_error| ProxyOriginShiftError::InconsistentProxy)?;
            if payload.fixture != fixture
                || payload.body != body
                || payload.child_index != entry.child_index
            {
                return Err(ProxyOriginShiftError::InconsistentProxy);
            }
            let fat_aabb = broad_phase
                .fat_aabb(entry.id)
                .map_err(|_error| ProxyOriginShiftError::InconsistentProxy)?;
            shifted_aabb(fat_aabb, shift)?;
            children.push((entry.child_index, shifted_aabb(entry.aabb, shift)?));
        }

        Ok(PreparedProxyOriginShift { children })
    }

    pub(super) fn commit_origin_shift(&mut self, prepared: PreparedProxyOriginShift) {
        debug_assert_eq!(self.entries.len(), prepared.children.len());
        for (entry, (child_index, aabb)) in self.entries.iter_mut().zip(prepared.children) {
            debug_assert_eq!(entry.child_index, child_index);
            entry.aabb = aabb;
        }
    }

    pub(super) fn prepare_creation(
        shape: &Shape,
        transform: Transform,
    ) -> Result<PreparedFixtureBounds, FixtureBoundsError> {
        let children = compute_child_bounds(shape, transform)?;
        for (_, aabb) in &children {
            validate_fat_aabb(*aabb)?;
        }
        Ok(PreparedFixtureBounds { children })
    }

    pub(super) fn create(
        &mut self,
        broad_phase: &mut BroadPhase<FixtureProxy>,
        fixture: FixtureId,
        body: BodyId,
        filter: FilterData,
        prepared: PreparedFixtureBounds,
    ) {
        debug_assert!(self.entries.is_empty());
        self.entries.reserve(prepared.children.len());
        for (child_index, aabb) in prepared.children {
            let payload = FixtureProxy {
                fixture,
                body,
                child_index,
            };
            let id = broad_phase
                .create_proxy(aabb, payload, filter)
                .expect("prevalidated fixture bounds must fit the broad phase");
            self.entries.push(FixtureProxyEntry {
                id,
                child_index,
                aabb,
            });
        }
    }

    pub(super) fn prepare_synchronization(
        &self,
        broad_phase: &BroadPhase<FixtureProxy>,
        fixture: FixtureId,
        body: BodyId,
        shape: &Shape,
        previous: Transform,
        current: Transform,
    ) -> Result<PreparedSynchronization, FixtureBoundsError> {
        self.validate_payloads(broad_phase, fixture, body);
        let previous_bounds = compute_child_bounds(shape, previous)?;
        let current_bounds = compute_child_bounds(shape, current)?;
        debug_assert_eq!(self.entries.len(), previous_bounds.len());
        debug_assert_eq!(self.entries.len(), current_bounds.len());
        let displacement = current.position() - previous.position();
        if !displacement.is_valid() {
            return Err(FixtureBoundsError::BroadPhaseOverflow);
        }

        let mut children = Vec::with_capacity(self.entries.len());
        for ((entry, (previous_child, previous_aabb)), (current_child, current_aabb)) in
            self.entries.iter().zip(previous_bounds).zip(current_bounds)
        {
            debug_assert_eq!(entry.child_index, previous_child);
            debug_assert_eq!(entry.child_index, current_child);
            let combined = previous_aabb.combined(current_aabb);
            validate_predicted_fat_aabb(combined, displacement)?;
            children.push((entry.child_index, combined));
        }

        Ok(PreparedSynchronization {
            children,
            displacement,
        })
    }

    pub(super) fn synchronize(
        &mut self,
        broad_phase: &mut BroadPhase<FixtureProxy>,
        prepared: PreparedSynchronization,
    ) {
        debug_assert_eq!(self.entries.len(), prepared.children.len());
        for (entry, (child_index, aabb)) in self.entries.iter_mut().zip(prepared.children) {
            debug_assert_eq!(entry.child_index, child_index);
            broad_phase
                .move_proxy(entry.id, aabb, prepared.displacement)
                .expect("prevalidated live fixture entry must synchronize");
            entry.aabb = aabb;
        }
    }

    pub(super) fn touch(
        &self,
        broad_phase: &mut BroadPhase<FixtureProxy>,
        fixture: FixtureId,
        body: BodyId,
    ) {
        self.validate_payloads(broad_phase, fixture, body);
        for entry in &self.entries {
            broad_phase
                .touch_proxy(entry.id)
                .expect("validated fixture entry must remain live");
        }
    }

    pub(super) fn set_filter(
        &self,
        broad_phase: &mut BroadPhase<FixtureProxy>,
        fixture: FixtureId,
        body: BodyId,
        filter: FilterData,
    ) {
        self.validate_payloads(broad_phase, fixture, body);
        for entry in &self.entries {
            broad_phase
                .set_filter_data(entry.id, filter)
                .expect("validated fixture entry must remain live");
        }
    }

    pub(super) fn destroy(
        &mut self,
        broad_phase: &mut BroadPhase<FixtureProxy>,
        fixture: FixtureId,
        body: BodyId,
    ) {
        self.validate_payloads(broad_phase, fixture, body);
        for entry in self.entries.drain(..) {
            let payload = broad_phase
                .destroy_proxy(entry.id)
                .expect("validated fixture entry must remain live");
            debug_assert_eq!(payload.fixture, fixture);
            debug_assert_eq!(payload.body, body);
            debug_assert_eq!(payload.child_index, entry.child_index);
        }
    }

    fn validate_payloads(
        &self,
        broad_phase: &BroadPhase<FixtureProxy>,
        fixture: FixtureId,
        body: BodyId,
    ) {
        for entry in &self.entries {
            let payload = broad_phase
                .payload(entry.id)
                .expect("fixture storage must contain only live broad-phase entries");
            assert_eq!(payload.fixture, fixture, "fixture entry owner must match");
            assert_eq!(payload.body, body, "fixture entry body must match");
            assert_eq!(
                payload.child_index, entry.child_index,
                "fixture entry child must match"
            );
        }
    }
}

fn compute_child_bounds(
    shape: &Shape,
    transform: Transform,
) -> Result<Vec<(ChildIndex, Aabb)>, FixtureBoundsError> {
    let mut children = Vec::with_capacity(shape.child_count());
    for requested in 0..shape.child_count() {
        let child_index = shape
            .child_index(requested)
            .expect("iteration remains inside the checked shape child count");
        let aabb = shape
            .compute_aabb(transform, child_index)
            .map_err(|_error| FixtureBoundsError::NonFiniteDerivedBounds)?;
        children.push((child_index, aabb));
    }
    Ok(children)
}

fn validate_fat_aabb(aabb: Aabb) -> Result<(), FixtureBoundsError> {
    let extension = Vec2::new(AABB_EXTENSION, AABB_EXTENSION);
    Aabb::new(
        aabb.lower_bound() - extension,
        aabb.upper_bound() + extension,
    )
    .map(|_aabb| ())
    .map_err(|_error| FixtureBoundsError::BroadPhaseOverflow)
}

fn validate_predicted_fat_aabb(aabb: Aabb, displacement: Vec2) -> Result<(), FixtureBoundsError> {
    validate_fat_aabb(aabb)?;
    let extension = Vec2::new(AABB_EXTENSION, AABB_EXTENSION);
    let mut lower = aabb.lower_bound() - extension;
    let mut upper = aabb.upper_bound() + extension;
    let predicted = AABB_MULTIPLIER * displacement;
    if !predicted.is_valid() {
        return Err(FixtureBoundsError::BroadPhaseOverflow);
    }
    if predicted.x < 0.0 {
        lower.x += predicted.x;
    } else {
        upper.x += predicted.x;
    }
    if predicted.y < 0.0 {
        lower.y += predicted.y;
    } else {
        upper.y += predicted.y;
    }
    Aabb::new(lower, upper)
        .map(|_aabb| ())
        .map_err(|_error| FixtureBoundsError::BroadPhaseOverflow)
}

fn shifted_aabb(aabb: Aabb, shift: Vec2) -> Result<Aabb, ProxyOriginShiftError> {
    Aabb::new(aabb.lower_bound() - shift, aabb.upper_bound() - shift)
        .map_err(|_error| ProxyOriginShiftError::NonFiniteBounds)
}
