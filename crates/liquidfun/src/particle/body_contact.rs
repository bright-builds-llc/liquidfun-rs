//! Stable fixture-particle contacts and source-ordered contact preparation.

use std::cmp::Ordering;

use crate::collision::{Aabb, ChildIndex, Shape};
use crate::math::{Transform, Vec2, settings};
use crate::{BodyId, FixtureId, ParticleFlags, ParticleId};

use super::ParticleSystemView;
use super::contact_scan::ContactProxy;
use super::proxy::visit_sorted_tag_indices_in_aabb;

const MAX_STRICT_CONTACTS_PER_PARTICLE: usize = 4;

/// One owned fixture-particle contact with stable semantic identities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleBodyContact {
    particle: ParticleId,
    body: BodyId,
    fixture: FixtureId,
    weight: f32,
    normal: Vec2,
    mass: f32,
}

impl ParticleBodyContact {
    pub(crate) const fn new_internal(
        particle: ParticleId,
        body: BodyId,
        fixture: FixtureId,
        weight: f32,
        normal: Vec2,
        mass: f32,
    ) -> Self {
        Self {
            particle,
            body,
            fixture,
            weight,
            normal,
            mass,
        }
    }

    /// Returns the stable particle identity.
    #[must_use]
    pub const fn particle(self) -> ParticleId {
        self.particle
    }

    /// Returns the contacted body identity.
    #[must_use]
    pub const fn body(self) -> BodyId {
        self.body
    }

    /// Returns the contacted fixture identity.
    #[must_use]
    pub const fn fixture(self) -> FixtureId {
        self.fixture
    }

    /// Returns `1 - distance / diameter` in pinned operation order.
    #[must_use]
    pub const fn weight(self) -> f32 {
        self.weight
    }

    /// Returns the contact normal directed from the fixture toward the particle system.
    #[must_use]
    pub const fn normal(self) -> Vec2 {
        self.normal
    }

    /// Returns the effective contact mass in kilograms.
    #[must_use]
    pub const fn mass(self) -> f32 {
        self.mass
    }
}

/// One source-timed fixture-particle listener effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleBodyContactEffect {
    /// A listener-flagged contact occurrence began.
    Begin(ParticleBodyContact),
    /// A listener-flagged fixture-particle occurrence ended.
    End {
        /// Fixture that no longer contacts the particle.
        fixture: FixtureId,
        /// Particle that no longer contacts the fixture.
        particle: ParticleId,
    },
}

/// Contacts and ordered listener effects prepared by one body-contact update.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleBodyContactUpdate {
    contacts: Vec<ParticleBodyContact>,
    effects: Vec<ParticleBodyContactEffect>,
}

impl ParticleBodyContactUpdate {
    /// Returns retained contacts in pinned generation or strict-prune order.
    #[must_use]
    pub fn contacts(&self) -> &[ParticleBodyContact] {
        &self.contacts
    }

    /// Returns new begins followed by remaining old ends.
    #[must_use]
    pub fn effects(&self) -> &[ParticleBodyContactEffect] {
        &self.effects
    }
}

#[derive(Clone)]
pub(crate) struct FixtureContactSource {
    pub(crate) fixture: FixtureId,
    pub(crate) body: BodyId,
    pub(crate) shape: Shape,
    pub(crate) transform: Transform,
    pub(crate) center: Vec2,
    pub(crate) inverse_mass: f32,
    pub(crate) inverse_inertia: f32,
}

#[allow(
    clippy::too_many_arguments,
    reason = "body-contact generation keeps the sorted proxy window beside the fixture query inputs"
)]
pub(crate) fn generate(
    view: &ParticleSystemView<'_>,
    proxies: &[ContactProxy],
    sources: &[FixtureContactSource],
    previous: &[ParticleBodyContact],
    diameter: f32,
    density: f32,
    strict: bool,
    mut filter: impl FnMut(&ParticleBodyContact) -> bool,
) -> ParticleBodyContactUpdate {
    let inverse_diameter = 1.0 / diameter;
    let inverse_stride = inverse_diameter * (1.0 / settings::PARTICLE_STRIDE);
    let particle_inverse_mass = (1.0 / density) * inverse_stride * inverse_stride;
    let mut contacts = Vec::new();
    let mut candidate_rows = Vec::new();
    let use_proxies = proxies.len() == view.positions().len();
    let ids = view.particle_ids();
    let positions = view.positions();
    let flags = view.flags();

    for source in sources {
        for child in 0..source.shape.child_count() {
            let child = ChildIndex::new(child, source.shape.child_count())
                .expect("enumerated shape child remains valid");
            let maybe_aabb = expanded_fixture_aabb(source, child, diameter);
            let mut write = BodyContactWrite {
                ids,
                positions,
                flags,
                source,
                child,
                maybe_aabb,
                diameter,
                inverse_diameter,
                particle_inverse_mass,
                contacts: &mut contacts,
                filter: &mut filter,
            };
            // A matching proxy buffer is the contact-pass tag sort. Query that
            // window, restore particle-row order, then keep the exact point
            // and distance tests. A length mismatch or a failed tag query
            // scans every particle, which is the previous behavior.
            if use_proxies
                && let Some(aabb) = maybe_aabb
                && rows_in_expanded_bounds(proxies, aabb, diameter, &mut candidate_rows)
            {
                candidate_rows.sort_unstable();
                for row in candidate_rows.iter().copied() {
                    consider_particle(&mut write, row);
                }
            } else {
                for row in 0..ids.len() {
                    consider_particle(&mut write, row);
                }
            }
        }
    }

    if strict {
        contacts.sort_by(|left, right| {
            let particle_order =
                particle_row(view, left.particle).cmp(&particle_row(view, right.particle));
            if particle_order == Ordering::Equal {
                right
                    .weight
                    .partial_cmp(&left.weight)
                    .unwrap_or(Ordering::Equal)
            } else {
                particle_order
            }
        });
        prune_strict_contacts(view, sources, diameter, &mut contacts);
    }

    let effects = if fixture_contact_listeners_active(view) {
        listener_effects(view, previous, &contacts)
    } else {
        Vec::new()
    };
    ParticleBodyContactUpdate { contacts, effects }
}

pub(crate) fn fixture_contact_listeners_active(view: &ParticleSystemView<'_>) -> bool {
    view.flags()
        .iter()
        .any(|flags| flags.contains(ParticleFlags::FIXTURE_CONTACT_LISTENER))
}

fn prune_strict_contacts(
    view: &ParticleSystemView<'_>,
    sources: &[FixtureContactSource],
    diameter: f32,
    contacts: &mut Vec<ParticleBodyContact>,
) {
    let mut maybe_particle = None;
    let mut contact_count = 0;
    contacts.retain(|contact| {
        if maybe_particle != Some(contact.particle) {
            maybe_particle = Some(contact.particle);
            contact_count = 0;
        }
        let within_limit = contact_count < MAX_STRICT_CONTACTS_PER_PARTICLE;
        contact_count += 1;
        if !within_limit {
            return false;
        }
        strict_contact_is_physical(view, sources, diameter, *contact)
    });
}

fn strict_contact_is_physical(
    view: &ParticleSystemView<'_>,
    sources: &[FixtureContactSource],
    diameter: f32,
    contact: ParticleBodyContact,
) -> bool {
    let row = particle_row(view, contact.particle);
    let projected = view.positions()[row] + diameter * (1.0 - contact.weight) * contact.normal;
    let source = sources
        .iter()
        .find(|source| source.fixture == contact.fixture)
        .expect("generated contacts retain their fixture source");
    if source
        .shape
        .test_point(source.transform, projected)
        .expect("world-owned checked shapes and transforms remain queryable")
    {
        return true;
    }
    (0..source.shape.child_count()).any(|child| {
        let child = ChildIndex::new(child, source.shape.child_count())
            .expect("enumerated shape child remains valid");
        source
            .shape
            .distance_to_point(source.transform, projected, child)
            .expect("world-owned checked shapes and transforms remain queryable")
            .distance()
            < settings::LINEAR_SLOP
    })
}

fn listener_effects(
    view: &ParticleSystemView<'_>,
    previous: &[ParticleBodyContact],
    contacts: &[ParticleBodyContact],
) -> Vec<ParticleBodyContactEffect> {
    let mut old = previous
        .iter()
        .copied()
        .filter(|contact| listener_enabled(view, contact.particle))
        .map(|contact| (contact.fixture, contact.particle, true))
        .collect::<Vec<_>>();
    let mut effects = Vec::new();
    for contact in contacts {
        if !listener_enabled(view, contact.particle) {
            continue;
        }
        let maybe_old = old.iter_mut().find(|(fixture, particle, valid)| {
            *valid && *fixture == contact.fixture && *particle == contact.particle
        });
        if let Some((_, _, valid)) = maybe_old {
            *valid = false;
        } else {
            effects.push(ParticleBodyContactEffect::Begin(*contact));
        }
    }
    effects.extend(old.into_iter().filter_map(|(fixture, particle, valid)| {
        valid.then_some(ParticleBodyContactEffect::End { fixture, particle })
    }));
    effects
}

fn listener_enabled(view: &ParticleSystemView<'_>, particle: ParticleId) -> bool {
    view.flags()[particle_row(view, particle)].contains(ParticleFlags::FIXTURE_CONTACT_LISTENER)
}

fn particle_row(view: &ParticleSystemView<'_>, particle: ParticleId) -> usize {
    view.maybe_live_row(particle)
        .expect("generated body contacts retain a current particle")
        .0
}

struct BodyContactWrite<'a, F> {
    ids: &'a [ParticleId],
    positions: &'a [Vec2],
    flags: &'a [ParticleFlags],
    source: &'a FixtureContactSource,
    child: ChildIndex,
    maybe_aabb: Option<Aabb>,
    diameter: f32,
    inverse_diameter: f32,
    particle_inverse_mass: f32,
    contacts: &'a mut Vec<ParticleBodyContact>,
    filter: &'a mut F,
}

fn consider_particle<F: FnMut(&ParticleBodyContact) -> bool>(
    write: &mut BodyContactWrite<'_, F>,
    row: usize,
) {
    let particle = write.ids[row];
    let position = write.positions[row];
    if let Some(aabb) = write.maybe_aabb
        && !aabb_contains_point(aabb, position)
    {
        return;
    }
    let distance = write
        .source
        .shape
        .distance_to_point(write.source.transform, position, write.child)
        .expect("world-owned checked shapes and transforms remain queryable");
    if distance.distance() >= write.diameter {
        return;
    }
    let flags = write.flags[row];
    let normal = -distance.normal();
    let offset = position - write.source.center;
    let normal_lever = offset.cross(distance.normal());
    let particle_term = if flags.contains(ParticleFlags::WALL) {
        0.0
    } else {
        write.particle_inverse_mass
    };
    let inverse_effective_mass = particle_term
        + write.source.inverse_mass
        + write.source.inverse_inertia * normal_lever * normal_lever;
    let contact = ParticleBodyContact {
        particle,
        body: write.source.body,
        fixture: write.source.fixture,
        weight: 1.0 - distance.distance() * write.inverse_diameter,
        normal,
        mass: if inverse_effective_mass > 0.0 {
            1.0 / inverse_effective_mass
        } else {
            0.0
        },
    };
    if flags.contains(ParticleFlags::FIXTURE_CONTACT_FILTER) && !(write.filter)(&contact) {
        return;
    }
    write.contacts.push(contact);
}

/// Fills `rows` with proxy rows whose tags overlap `aabb`.
///
/// Returns false when the tag query cannot run, so the caller scans every row.
fn rows_in_expanded_bounds(
    proxies: &[ContactProxy],
    aabb: Aabb,
    diameter: f32,
    rows: &mut Vec<usize>,
) -> bool {
    rows.clear();
    let queried = visit_sorted_tag_indices_in_aabb(
        proxies.len(),
        |index| proxies[index].tag,
        diameter,
        aabb,
        |index| rows.push(proxies[index].row),
    );
    queried.is_ok()
}

fn expanded_fixture_aabb(
    source: &FixtureContactSource,
    child: ChildIndex,
    diameter: f32,
) -> Option<Aabb> {
    let aabb = source.shape.compute_aabb(source.transform, child).ok()?;
    let expansion = Vec2::new(diameter, diameter);
    Aabb::new(
        aabb.lower_bound() - expansion,
        aabb.upper_bound() + expansion,
    )
    .ok()
}

fn aabb_contains_point(aabb: Aabb, position: Vec2) -> bool {
    let lower = aabb.lower_bound();
    let upper = aabb.upper_bound();
    position.x > lower.x && position.x < upper.x && position.y > lower.y && position.y < upper.y
}

#[cfg(test)]
mod tests {
    use crate::collision::{CircleShape, Shape};
    use crate::identity::{
        BodyId, FixtureId, HandleIdentity, Identity, ParticleSystemId, WorldKey,
    };
    use crate::math::Transform;
    use crate::particle::contact_scan;
    use crate::particle::storage::{ParticleInput, ParticleStorage};
    use crate::particle::{ParticleFlags, ParticleSystemView};

    use super::*;

    fn circle_scene() -> (ParticleStorage, FixtureContactSource, f32) {
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        let mut storage =
            ParticleStorage::new(world, system, 32, 8, 8).expect("test storage contract is valid");
        let diameter = 0.4;
        let positions = [
            Vec2::new(0.0, 0.0),
            Vec2::new(1.05, 0.0),
            Vec2::new(1.5, 0.0),
            Vec2::new(8.0, 8.0),
        ];
        for position in positions {
            let _ = storage
                .create(ParticleInput {
                    position,
                    velocity: Vec2::ZERO,
                    flags: ParticleFlags::WATER,
                    maybe_group: None,
                    maybe_color: None,
                    maybe_user_association: None,
                    maybe_expiration_time: None,
                })
                .expect("test particle fits");
        }
        let shape =
            Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should be valid"));
        let source = FixtureContactSource {
            fixture: FixtureId::from_identity(Identity::new(world, 8, 0)),
            body: BodyId::from_identity(Identity::new(world, 1, 0)),
            shape,
            transform: Transform::from_position_angle(Vec2::ZERO, 0.0),
            center: Vec2::ZERO,
            inverse_mass: 0.0,
            inverse_inertia: 0.0,
        };
        (storage, source, diameter)
    }

    fn assert_same_contact_bits(left: &[ParticleBodyContact], right: &[ParticleBodyContact]) {
        assert_eq!(left.len(), right.len());
        assert!(!left.is_empty());
        for (full, filtered) in left.iter().zip(right) {
            assert_eq!(full.particle(), filtered.particle());
            assert_eq!(full.body(), filtered.body());
            assert_eq!(full.fixture(), filtered.fixture());
            assert_eq!(full.weight().to_bits(), filtered.weight().to_bits());
            assert_eq!(full.normal().x.to_bits(), filtered.normal().x.to_bits());
            assert_eq!(full.normal().y.to_bits(), filtered.normal().y.to_bits());
            assert_eq!(full.mass().to_bits(), filtered.mass().to_bits());
        }
    }

    #[test]
    fn proxy_rows_match_full_scan_contact_bits() {
        // Arrange
        let (storage, source, diameter) = circle_scene();
        let view = ParticleSystemView::new(&storage);
        let mut proxies = Vec::new();
        let mut stored_contacts = Vec::new();
        contact_scan::fill_stored_contacts(
            view.positions(),
            view.flags(),
            view.particle_ids(),
            diameter,
            &mut proxies,
            &mut stored_contacts,
            &mut |_contact| true,
        )
        .expect("proxy fill should accept the test diameter");
        let far = view.particle_ids()[3];

        // Act
        let sources = std::slice::from_ref(&source);
        let full = generate(&view, &[], sources, &[], diameter, 1.0, false, |_| true);
        let filtered = generate(&view, &proxies, sources, &[], diameter, 1.0, false, |_| {
            true
        });

        // Assert
        assert_same_contact_bits(full.contacts(), filtered.contacts());
        assert!(
            full.contacts()
                .iter()
                .any(|contact| contact.particle() == view.particle_ids()[0])
        );
        assert!(
            full.contacts()
                .iter()
                .all(|contact| contact.particle() != far)
        );
    }

    #[test]
    fn proxy_rows_match_strict_full_scan_contact_bits() {
        // Arrange
        let (storage, source, diameter) = circle_scene();
        let view = ParticleSystemView::new(&storage);
        let mut proxies = Vec::new();
        let mut stored_contacts = Vec::new();
        contact_scan::fill_stored_contacts(
            view.positions(),
            view.flags(),
            view.particle_ids(),
            diameter,
            &mut proxies,
            &mut stored_contacts,
            &mut |_contact| true,
        )
        .expect("proxy fill should accept the test diameter");

        // Act
        let sources = std::slice::from_ref(&source);
        let full = generate(&view, &[], sources, &[], diameter, 1.0, true, |_| true);
        let filtered = generate(&view, &proxies, sources, &[], diameter, 1.0, true, |_| true);

        // Assert
        assert_same_contact_bits(full.contacts(), filtered.contacts());
    }
}
