//! Stable particle contacts, flag-gated decisions, and ordered listener effects.

use crate::math::{Vec2, inverse_sqrt};
use crate::{ParticleFlags, ParticleId};

use super::storage::lanes::ParticleContact as StoredParticleContact;
use super::{ParticleNeighborhood, ParticleSystemView};

/// A failure while generating contacts from a particle-system snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleContactError {
    /// The neighborhood belongs to a different particle system.
    WrongParticleSystem,
    /// A supplied previous or neighborhood contact references no current particle.
    MissingParticle,
}

/// One owned particle-particle contact occurrence with stable identities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleContact {
    particles: [ParticleId; 2],
    flags: ParticleFlags,
    weight: f32,
    normal: Vec2,
}

impl ParticleContact {
    pub(crate) const fn new_internal(
        particles: [ParticleId; 2],
        flags: ParticleFlags,
        weight: f32,
        normal: Vec2,
    ) -> Self {
        Self {
            particles,
            flags,
            weight,
            normal,
        }
    }

    /// Returns the stable particles in source contact order.
    #[must_use]
    pub const fn particles(self) -> [ParticleId; 2] {
        self.particles
    }

    /// Returns the exact union of both particle flag sets.
    #[must_use]
    pub const fn flags(self) -> ParticleFlags {
        self.flags
    }

    /// Returns `1 - distance / diameter` in source operation order.
    #[must_use]
    pub const fn weight(self) -> f32 {
        self.weight
    }

    /// Returns the normal from the first particle toward the second.
    #[must_use]
    pub const fn normal(self) -> Vec2 {
        self.normal
    }
}

/// One source-timed particle contact listener effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleContactEffect {
    /// A listener-flagged contact occurrence began.
    Begin(ParticleContact),
    /// A listener-flagged contact occurrence ended.
    End([ParticleId; 2]),
}

/// Contacts and ordered listener effects prepared by one update.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleContactUpdate {
    contacts: Vec<ParticleContact>,
    effects: Vec<ParticleContactEffect>,
}

impl ParticleContactUpdate {
    /// Generates, filters, and diffs one particle contact snapshot.
    ///
    /// `filter` is borrowed only for this call and is invoked only when the
    /// combined flags contain [`ParticleFlags::PARTICLE_CONTACT_FILTER`]. A
    /// `true` decision retains the contact. Listener effects are likewise
    /// computed only for pairs whose current combined flags request them.
    ///
    /// # Errors
    ///
    /// Returns a typed error before invoking `filter` if the neighborhood or a
    /// previous occurrence does not belong to the supplied current view.
    pub fn generate(
        view: &ParticleSystemView<'_>,
        neighborhood: &ParticleNeighborhood,
        previous: &[ParticleContact],
        mut filter: impl FnMut(&ParticleContact) -> bool,
    ) -> Result<Self, ParticleContactError> {
        if neighborhood.system() != view.system() {
            return Err(ParticleContactError::WrongParticleSystem);
        }

        validate_pairs(view, neighborhood, previous)?;
        let contacts = collect_new_contacts(view, neighborhood, &mut filter)?;
        let effects = listener_effects(view, previous, &contacts)?;
        Ok(Self { contacts, effects })
    }

    /// Generates contacts using already-indexed previous storage contacts.
    ///
    /// The stepping path keeps C++-shaped dense rows instead of allocating a
    /// semantic `ParticleContact` snapshot of last step's contacts.
    pub(crate) fn generate_from_stored(
        view: &ParticleSystemView<'_>,
        neighborhood: &ParticleNeighborhood,
        previous: &[StoredParticleContact],
        mut filter: impl FnMut(&ParticleContact) -> bool,
    ) -> Result<Self, ParticleContactError> {
        if neighborhood.system() != view.system() {
            return Err(ParticleContactError::WrongParticleSystem);
        }

        if neighborhood.pairs().len() != neighborhood.pair_rows().len() {
            return Err(ParticleContactError::MissingParticle);
        }
        if cfg!(debug_assertions) {
            validate_neighborhood(view, neighborhood)?;
        }
        let contacts = collect_new_contacts(view, neighborhood, &mut filter)?;
        let effects = listener_effects_from_stored(view, previous, &contacts)?;
        Ok(Self { contacts, effects })
    }

    /// Returns retained contacts in source generation order.
    #[must_use]
    pub fn contacts(&self) -> &[ParticleContact] {
        &self.contacts
    }

    /// Returns begin effects in new-contact order followed by remaining ends.
    #[must_use]
    pub fn effects(&self) -> &[ParticleContactEffect] {
        &self.effects
    }
}

fn validate_pairs(
    view: &ParticleSystemView<'_>,
    neighborhood: &ParticleNeighborhood,
    previous: &[ParticleContact],
) -> Result<(), ParticleContactError> {
    validate_neighborhood(view, neighborhood)?;
    for contact in previous {
        view.maybe_live_row(contact.particles[0])
            .ok_or(ParticleContactError::MissingParticle)?;
        view.maybe_live_row(contact.particles[1])
            .ok_or(ParticleContactError::MissingParticle)?;
    }
    Ok(())
}

fn validate_neighborhood(
    view: &ParticleSystemView<'_>,
    neighborhood: &ParticleNeighborhood,
) -> Result<(), ParticleContactError> {
    if neighborhood.pairs().len() != neighborhood.pair_rows().len() {
        return Err(ParticleContactError::MissingParticle);
    }
    for pair_rows in neighborhood.pair_rows() {
        let [a, b] = *pair_rows;
        if view.positions().get(a.0).is_none() || view.positions().get(b.0).is_none() {
            return Err(ParticleContactError::MissingParticle);
        }
    }
    Ok(())
}

fn collect_new_contacts(
    view: &ParticleSystemView<'_>,
    neighborhood: &ParticleNeighborhood,
    filter: &mut impl FnMut(&ParticleContact) -> bool,
) -> Result<Vec<ParticleContact>, ParticleContactError> {
    let diameter = neighborhood.diameter();
    let squared_diameter = diameter * diameter;
    let inverse_diameter = 1.0 / diameter;
    let mut contacts = Vec::with_capacity(neighborhood.pairs().len());
    for (candidate, pair_rows) in neighborhood.pairs().iter().zip(neighborhood.pair_rows()) {
        let [a, b] = *pair_rows;
        let Some(position_a) = view.positions().get(a.0) else {
            return Err(ParticleContactError::MissingParticle);
        };
        let Some(position_b) = view.positions().get(b.0) else {
            return Err(ParticleContactError::MissingParticle);
        };
        let difference = *position_b - *position_a;
        let distance_squared = difference.dot(difference);
        if distance_squared >= squared_diameter {
            continue;
        }
        let Some(flags_a) = view.flags().get(a.0) else {
            return Err(ParticleContactError::MissingParticle);
        };
        let Some(flags_b) = view.flags().get(b.0) else {
            return Err(ParticleContactError::MissingParticle);
        };
        let inverse_distance = inverse_sqrt(distance_squared);
        let contact = ParticleContact {
            particles: candidate.particles(),
            flags: *flags_a | *flags_b,
            weight: 1.0 - distance_squared * inverse_distance * inverse_diameter,
            normal: inverse_distance * difference,
        };
        if contact
            .flags
            .contains(ParticleFlags::PARTICLE_CONTACT_FILTER)
            && !filter(&contact)
        {
            continue;
        }
        contacts.push(contact);
    }
    Ok(contacts)
}

fn listener_effects(
    view: &ParticleSystemView<'_>,
    previous: &[ParticleContact],
    contacts: &[ParticleContact],
) -> Result<Vec<ParticleContactEffect>, ParticleContactError> {
    let old = previous
        .iter()
        .map(|contact| {
            let Some(row_a) = view.maybe_live_row(contact.particles[0]) else {
                return Err(ParticleContactError::MissingParticle);
            };
            let Some(row_b) = view.maybe_live_row(contact.particles[1]) else {
                return Err(ParticleContactError::MissingParticle);
            };
            let Some(flags_a) = view.flags().get(row_a.0) else {
                return Err(ParticleContactError::MissingParticle);
            };
            let Some(flags_b) = view.flags().get(row_b.0) else {
                return Err(ParticleContactError::MissingParticle);
            };
            let flags = *flags_a | *flags_b;
            Ok(([row_a.0, row_b.0], contact.particles, flags))
        })
        .filter_map(|entry: Result<_, ParticleContactError>| match entry {
            Ok((rows, particles, flags))
                if flags.contains(ParticleFlags::PARTICLE_CONTACT_LISTENER) =>
            {
                Some(Ok((rows, particles, true)))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(diff_listener_effects(old, contacts))
}

fn listener_effects_from_stored(
    view: &ParticleSystemView<'_>,
    previous: &[StoredParticleContact],
    contacts: &[ParticleContact],
) -> Result<Vec<ParticleContactEffect>, ParticleContactError> {
    let ids = view.particle_ids();
    let flags = view.flags();
    let mut old = Vec::with_capacity(previous.len());
    for contact in previous {
        let [row_a, row_b] = contact.indices;
        let Some(flags_a) = flags.get(row_a.0) else {
            return Err(ParticleContactError::MissingParticle);
        };
        let Some(flags_b) = flags.get(row_b.0) else {
            return Err(ParticleContactError::MissingParticle);
        };
        if !(*flags_a | *flags_b).contains(ParticleFlags::PARTICLE_CONTACT_LISTENER) {
            continue;
        }
        let Some(&id_a) = ids.get(row_a.0) else {
            return Err(ParticleContactError::MissingParticle);
        };
        let Some(&id_b) = ids.get(row_b.0) else {
            return Err(ParticleContactError::MissingParticle);
        };
        old.push(([row_a.0, row_b.0], [id_a, id_b], true));
    }
    Ok(diff_listener_effects(old, contacts))
}

fn diff_listener_effects(
    mut old: Vec<([usize; 2], [ParticleId; 2], bool)>,
    contacts: &[ParticleContact],
) -> Vec<ParticleContactEffect> {
    old.sort_by_key(|(rows, _, _)| *rows);
    let mut effects = Vec::new();
    for contact in contacts {
        if !contact
            .flags
            .contains(ParticleFlags::PARTICLE_CONTACT_LISTENER)
        {
            continue;
        }
        let maybe_old = old.iter_mut().find(|(_, particles, valid)| {
            *valid && unordered_pair_matches(*particles, contact.particles)
        });
        if let Some((_, _, valid)) = maybe_old {
            *valid = false;
        } else {
            effects.push(ParticleContactEffect::Begin(*contact));
        }
    }
    effects.extend(
        old.into_iter()
            .filter(|(_, _, valid)| *valid)
            .map(|(_, particles, _)| ParticleContactEffect::End(particles)),
    );
    effects
}

fn unordered_pair_matches(left: [ParticleId; 2], right: [ParticleId; 2]) -> bool {
    left == right || left == [right[1], right[0]]
}
