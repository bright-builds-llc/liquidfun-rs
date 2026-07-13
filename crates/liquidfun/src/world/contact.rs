use crate::collision::{ChildIndex, ContactFeatureId, Manifold, Shape};
use crate::math::max;
use crate::{BodyId, FixtureId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ContactEndpoint {
    pub(super) fixture: FixtureId,
    pub(super) body: BodyId,
    pub(super) child_index: ChildIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ContactKey {
    pub(super) first: ContactEndpoint,
    pub(super) second: ContactEndpoint,
}

impl ContactKey {
    pub(super) fn matches_unordered(self, other: Self) -> bool {
        self == other || (self.first == other.second && self.second == other.first)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ContactPoint {
    feature_id: ContactFeatureId,
    normal_impulse: f32,
    tangent_impulse: f32,
}

impl ContactPoint {
    fn cold(feature_id: ContactFeatureId) -> Self {
        Self {
            feature_id,
            normal_impulse: 0.0,
            tangent_impulse: 0.0,
        }
    }

    fn snapshot(self) -> ContactPointSnapshot {
        ContactPointSnapshot {
            feature_id: self.feature_id,
            normal_impulse: self.normal_impulse,
            tangent_impulse: self.tangent_impulse,
        }
    }

    pub(super) const fn feature_id(self) -> ContactFeatureId {
        self.feature_id
    }

    pub(super) const fn normal_impulse(self) -> f32 {
        self.normal_impulse
    }

    pub(super) const fn tangent_impulse(self) -> f32 {
        self.tangent_impulse
    }

    pub(super) fn set_impulses(&mut self, normal_impulse: f32, tangent_impulse: f32) {
        self.normal_impulse = normal_impulse;
        self.tangent_impulse = tangent_impulse;
    }
}

#[derive(Debug)]
pub(super) struct Contact {
    pub(super) key: ContactKey,
    pub(super) ordinal: u64,
    flags: u8,
    pub(super) maybe_manifold: Option<Manifold>,
    pub(super) points: Vec<ContactPoint>,
    pub(super) friction: f32,
    pub(super) restitution: f32,
}

impl Contact {
    const TOUCHING: u8 = 1 << 0;
    const ENABLED: u8 = 1 << 1;
    const NEEDS_FILTERING: u8 = 1 << 2;
    const SENSOR: u8 = 1 << 3;

    pub(super) fn new(
        key: ContactKey,
        ordinal: u64,
        friction_a: f32,
        friction_b: f32,
        restitution_a: f32,
        restitution_b: f32,
    ) -> Self {
        Self {
            key,
            ordinal,
            flags: Self::ENABLED,
            maybe_manifold: None,
            points: Vec::new(),
            friction: (friction_a * friction_b).sqrt(),
            restitution: max(restitution_a, restitution_b),
        }
    }

    pub(super) fn replace_manifold(&mut self, maybe_manifold: Option<Manifold>) {
        let previous = std::mem::take(&mut self.points);
        self.points = maybe_manifold
            .as_ref()
            .into_iter()
            .flat_map(Manifold::points)
            .map(|point| {
                previous
                    .iter()
                    .find(|candidate| candidate.feature_id == point.feature_id())
                    .copied()
                    .unwrap_or_else(|| ContactPoint::cold(point.feature_id()))
            })
            .collect();
        self.maybe_manifold = maybe_manifold;
    }

    pub(super) fn clear_manifold(&mut self) {
        self.maybe_manifold = None;
        self.points.clear();
    }

    pub(super) fn store_impulses(&mut self, impulses: &[(ContactFeatureId, f32, f32)]) {
        for (feature_id, normal_impulse, tangent_impulse) in impulses {
            let maybe_point = self
                .points
                .iter_mut()
                .find(|point| point.feature_id() == *feature_id);
            if let Some(point) = maybe_point {
                point.set_impulses(*normal_impulse, *tangent_impulse);
            }
        }
    }

    pub(super) const fn is_touching(&self) -> bool {
        self.flags & Self::TOUCHING != 0
    }

    pub(super) fn set_touching(&mut self, touching: bool) {
        self.set_flag(Self::TOUCHING, touching);
    }

    pub(super) const fn is_enabled(&self) -> bool {
        self.flags & Self::ENABLED != 0
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.set_flag(Self::ENABLED, enabled);
    }

    pub(super) const fn needs_filtering(&self) -> bool {
        self.flags & Self::NEEDS_FILTERING != 0
    }

    pub(super) fn set_needs_filtering(&mut self, needs_filtering: bool) {
        self.set_flag(Self::NEEDS_FILTERING, needs_filtering);
    }

    pub(super) const fn is_sensor(&self) -> bool {
        self.flags & Self::SENSOR != 0
    }

    pub(super) fn set_sensor(&mut self, sensor: bool) {
        self.set_flag(Self::SENSOR, sensor);
    }

    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    pub(super) fn snapshot(&self) -> ManagedContactSnapshot {
        ManagedContactSnapshot {
            occurrence: self.ordinal,
            fixtures: [self.key.first.fixture, self.key.second.fixture],
            child_indices: [self.key.first.child_index, self.key.second.child_index],
            touching: self.is_touching(),
            enabled: self.is_enabled(),
            sensor: self.is_sensor(),
            maybe_manifold: self.maybe_manifold.clone(),
            points: self
                .points
                .iter()
                .copied()
                .map(ContactPoint::snapshot)
                .collect(),
            friction: self.friction,
            restitution: self.restitution,
        }
    }

    pub(super) fn other_body(&self, body: BodyId) -> Option<BodyId> {
        if self.key.first.body == body {
            return Some(self.key.second.body);
        }
        if self.key.second.body == body {
            return Some(self.key.first.body);
        }
        None
    }
}

pub(super) fn canonical_contact_key(
    first: ContactEndpoint,
    first_shape: &Shape,
    second: ContactEndpoint,
    second_shape: &Shape,
) -> Option<ContactKey> {
    use Shape::{Chain, Circle, Edge, Polygon};

    let primary = match (first_shape, second_shape) {
        (Circle(_) | Polygon(_) | Edge(_) | Chain(_), Circle(_))
        | (Polygon(_) | Edge(_) | Chain(_), Polygon(_)) => true,
        (Circle(_), Polygon(_) | Edge(_) | Chain(_)) | (Polygon(_), Edge(_) | Chain(_)) => false,
        (Edge(_) | Chain(_), Edge(_) | Chain(_)) => {
            return None;
        }
    };

    Some(if primary {
        ContactKey { first, second }
    } else {
        ContactKey {
            first: second,
            second: first,
        }
    })
}

/// One owned semantic manifold point and its reserved warm-start impulses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactPointSnapshot {
    feature_id: ContactFeatureId,
    normal_impulse: f32,
    tangent_impulse: f32,
}

impl ContactPointSnapshot {
    /// Returns the semantic feature identity used for persistence.
    #[must_use]
    pub const fn feature_id(self) -> ContactFeatureId {
        self.feature_id
    }

    /// Returns the reserved normal impulse lane.
    #[must_use]
    pub const fn normal_impulse(self) -> f32 {
        self.normal_impulse
    }

    /// Returns the reserved tangent impulse lane.
    #[must_use]
    pub const fn tangent_impulse(self) -> f32 {
        self.tangent_impulse
    }
}

/// Owned, non-authoritative state for one private manager contact occurrence.
///
/// This snapshot carries no reusable contact identity or storage coordinate.
#[derive(Debug, Clone, PartialEq)]
pub struct ManagedContactSnapshot {
    occurrence: u64,
    fixtures: [FixtureId; 2],
    child_indices: [ChildIndex; 2],
    touching: bool,
    enabled: bool,
    sensor: bool,
    maybe_manifold: Option<Manifold>,
    points: Vec<ContactPointSnapshot>,
    friction: f32,
    restitution: f32,
}

impl ManagedContactSnapshot {
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    #[must_use]
    pub const fn differential_occurrence(&self) -> u64 {
        self.occurrence + 1
    }

    /// Returns fixture identities in the manager's oriented occurrence order.
    #[must_use]
    pub const fn fixtures(&self) -> [FixtureId; 2] {
        self.fixtures
    }

    /// Returns shape-child coordinates in oriented occurrence order.
    #[must_use]
    pub const fn child_indices(&self) -> [ChildIndex; 2] {
        self.child_indices
    }

    /// Returns whether the occurrence is currently touching.
    #[must_use]
    pub const fn is_touching(&self) -> bool {
        self.touching
    }

    /// Returns whether the occurrence is enabled for this update.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns whether either fixture is currently a sensor.
    #[must_use]
    pub const fn is_sensor(&self) -> bool {
        self.sensor
    }

    /// Returns the canonical manifold, absent for sensors and separation.
    #[must_use]
    pub const fn maybe_manifold(&self) -> Option<&Manifold> {
        self.maybe_manifold.as_ref()
    }

    /// Returns manifold points and reserved impulses in canonical point order.
    #[must_use]
    pub fn points(&self) -> &[ContactPointSnapshot] {
        &self.points
    }

    /// Returns the mixed friction captured when the contact was created.
    #[must_use]
    pub const fn friction(&self) -> f32 {
        self.friction
    }

    /// Returns the mixed restitution captured when the contact was created.
    #[must_use]
    pub const fn restitution(&self) -> f32 {
        self.restitution
    }
}

/// The exact touching transition for one private manager occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContactTransitionKind {
    /// A separated occurrence began touching.
    Begin,
    /// A touching occurrence remained touching with semantic persistence.
    Persist,
    /// A touching occurrence separated or was destroyed.
    End,
}

/// Owned contact transition evidence with no durable contact identity.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactTransition {
    kind: ContactTransitionKind,
    contact: ManagedContactSnapshot,
}

impl ContactTransition {
    pub(super) const fn new(kind: ContactTransitionKind, contact: ManagedContactSnapshot) -> Self {
        Self { kind, contact }
    }

    /// Returns the touching transition kind.
    #[must_use]
    pub const fn kind(&self) -> ContactTransitionKind {
        self.kind
    }

    /// Returns owned semantic state captured at the transition.
    #[must_use]
    pub const fn contact(&self) -> &ManagedContactSnapshot {
        &self.contact
    }
}
