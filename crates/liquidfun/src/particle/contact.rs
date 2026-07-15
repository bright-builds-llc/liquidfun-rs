//! Stable particle contacts, flag-gated decisions, and ordered listener effects.

use crate::math::{Vec2, inverse_sqrt};
use crate::{ParticleFlags, ParticleId};

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
        let diameter = neighborhood.diameter();
        let squared_diameter = diameter * diameter;
        let inverse_diameter = 1.0 / diameter;
        let mut contacts = Vec::new();
        for candidate in neighborhood.pairs() {
            let particles = candidate.particles();
            let [a, b] = particle_rows(view, particles)?;
            let difference = view.positions()[b] - view.positions()[a];
            let distance_squared = difference.dot(difference);
            if distance_squared >= squared_diameter {
                continue;
            }
            let inverse_distance = inverse_sqrt(distance_squared);
            let contact = ParticleContact {
                particles,
                flags: view.flags()[a] | view.flags()[b],
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

        let effects = listener_effects(view, previous, &contacts)?;
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
    for pair in neighborhood.pairs() {
        particle_rows(view, pair.particles())?;
    }
    for contact in previous {
        particle_rows(view, contact.particles)?;
    }
    Ok(())
}

fn listener_effects(
    view: &ParticleSystemView<'_>,
    previous: &[ParticleContact],
    contacts: &[ParticleContact],
) -> Result<Vec<ParticleContactEffect>, ParticleContactError> {
    let mut old = previous
        .iter()
        .map(|contact| {
            let rows = particle_rows(view, contact.particles)?;
            let flags = view.flags()[rows[0]] | view.flags()[rows[1]];
            Ok((rows, contact.particles, flags))
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
    Ok(effects)
}

fn particle_rows(
    view: &ParticleSystemView<'_>,
    particles: [ParticleId; 2],
) -> Result<[usize; 2], ParticleContactError> {
    let row_for = |particle| {
        view.particle_ids()
            .iter()
            .position(|candidate| *candidate == particle)
            .ok_or(ParticleContactError::MissingParticle)
    };
    Ok([row_for(particles[0])?, row_for(particles[1])?])
}

fn unordered_pair_matches(left: [ParticleId; 2], right: [ParticleId; 2]) -> bool {
    left == right || left == [right[1], right[0]]
}
