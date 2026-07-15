//! Stable fixture-particle contacts and source-ordered contact preparation.

use std::cmp::Ordering;

use crate::collision::{ChildIndex, Shape};
use crate::math::{Transform, Vec2, settings};
use crate::{BodyId, FixtureId, ParticleFlags, ParticleId};

use super::ParticleSystemView;

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

pub(crate) fn generate(
    view: &ParticleSystemView<'_>,
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

    for source in sources {
        for child in 0..source.shape.child_count() {
            let child = ChildIndex::new(child, source.shape.child_count())
                .expect("enumerated shape child remains valid");
            for (row, particle) in view.particle_ids().iter().copied().enumerate() {
                let position = view.positions()[row];
                let distance = source
                    .shape
                    .distance_to_point(source.transform, position, child)
                    .expect("world-owned checked shapes and transforms remain queryable");
                if distance.distance() >= diameter {
                    continue;
                }
                let flags = view.flags()[row];
                let normal = -distance.normal();
                let offset = position - source.center;
                let normal_lever = offset.cross(distance.normal());
                let particle_term = if flags.contains(ParticleFlags::WALL) {
                    0.0
                } else {
                    particle_inverse_mass
                };
                let inverse_effective_mass = particle_term
                    + source.inverse_mass
                    + source.inverse_inertia * normal_lever * normal_lever;
                let contact = ParticleBodyContact {
                    particle,
                    body: source.body,
                    fixture: source.fixture,
                    weight: 1.0 - distance.distance() * inverse_diameter,
                    normal,
                    mass: if inverse_effective_mass > 0.0 {
                        1.0 / inverse_effective_mass
                    } else {
                        0.0
                    },
                };
                if flags.contains(ParticleFlags::FIXTURE_CONTACT_FILTER) && !filter(&contact) {
                    continue;
                }
                contacts.push(contact);
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

    let effects = listener_effects(view, previous, &contacts);
    ParticleBodyContactUpdate { contacts, effects }
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
    view.particle_ids()
        .iter()
        .position(|candidate| *candidate == particle)
        .expect("generated body contacts retain a current particle")
}
