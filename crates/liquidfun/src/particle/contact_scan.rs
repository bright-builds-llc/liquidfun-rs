//! Hot-path particle contacts written during the proxy scan.
//!
//! The public neighborhood API still materializes broad pairs. Stepping follows
//! the pinned scalar `FindContacts_Reference` shape: update proxy tags, sort
//! that buffer in place, test distance while walking tag windows, and append
//! only real contacts. A four-wide distance kernel was measured on Dam Break
//! and did not beat this scan; neighbor windows there average about three
//! proxies, and the pinned x86 oracle uses this scalar path rather than NEON.

use crate::ParticleId;
use crate::math::{Vec2, inverse_sqrt};
use crate::particle::storage::lanes::ParticleContact as StoredParticleContact;
use crate::particle::{ParticleContact, ParticleFlags, ParticleProxyError};

use super::contact::ParticleContactError;
use super::proxy::{RELATIVE_BOTTOM_LEFT, RELATIVE_BOTTOM_RIGHT, RELATIVE_RIGHT, checked_tag};

/// One sorted spatial proxy retained for the rest of the solver iteration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContactProxy {
    pub(crate) tag: u32,
    pub(crate) row: usize,
}

#[derive(Debug)]
pub(crate) enum ContactFillError {
    Proxy(ParticleProxyError),
    Contact(ParticleContactError),
}

/// Rebuilds sorted proxies and appends narrow contacts in source window order.
pub(crate) fn fill_stored_contacts(
    positions: &[Vec2],
    flags: &[ParticleFlags],
    ids: &[ParticleId],
    diameter: f32,
    proxies: &mut Vec<ContactProxy>,
    contacts: &mut Vec<StoredParticleContact>,
    filter: &mut impl FnMut(&ParticleContact) -> bool,
) -> Result<(), ContactFillError> {
    contacts.clear();
    rebuild_proxies(positions, diameter, proxies)?;
    if proxies.is_empty() {
        return Ok(());
    }
    if flags.len() != positions.len() || ids.len() != positions.len() {
        return Err(ContactFillError::Contact(
            ParticleContactError::MissingParticle,
        ));
    }

    let mut write = ContactWrite {
        positions,
        flags,
        ids,
        squared_diameter: diameter * diameter,
        inverse_diameter: 1.0 / diameter,
        contacts,
        filter,
    };
    let mut below_start = 0;
    for a_index in 0..proxies.len() {
        let a = proxies[a_index];
        let right_tag = a.tag.wrapping_add(RELATIVE_RIGHT);
        let mut right_end = a_index + 1;
        while right_end < proxies.len() && proxies[right_end].tag <= right_tag {
            right_end += 1;
        }
        consider_window(a.row, &proxies[a_index + 1..right_end], &mut write);

        let bottom_left_tag = a.tag.wrapping_add(RELATIVE_BOTTOM_LEFT);
        while below_start < proxies.len() && proxies[below_start].tag < bottom_left_tag {
            below_start += 1;
        }
        let bottom_right_tag = a.tag.wrapping_add(RELATIVE_BOTTOM_RIGHT);
        let mut bottom_end = below_start;
        while bottom_end < proxies.len() && proxies[bottom_end].tag <= bottom_right_tag {
            bottom_end += 1;
        }
        consider_window(a.row, &proxies[below_start..bottom_end], &mut write);
    }
    Ok(())
}

struct ContactWrite<'a, F> {
    positions: &'a [Vec2],
    flags: &'a [ParticleFlags],
    ids: &'a [ParticleId],
    squared_diameter: f32,
    inverse_diameter: f32,
    contacts: &'a mut Vec<StoredParticleContact>,
    filter: &'a mut F,
}

fn rebuild_proxies(
    positions: &[Vec2],
    diameter: f32,
    proxies: &mut Vec<ContactProxy>,
) -> Result<(), ContactFillError> {
    if !diameter.is_finite() {
        return Err(ContactFillError::Proxy(
            ParticleProxyError::NonFiniteDiameter,
        ));
    }
    if diameter <= 0.0 {
        return Err(ContactFillError::Proxy(
            ParticleProxyError::NonPositiveDiameter,
        ));
    }
    let inverse_diameter = 1.0 / diameter;
    let squared_diameter = diameter * diameter;
    if !inverse_diameter.is_finite()
        || inverse_diameter <= 0.0
        || !squared_diameter.is_finite()
        || squared_diameter <= 0.0
    {
        return Err(ContactFillError::Proxy(
            ParticleProxyError::DiameterOutOfRange,
        ));
    }

    proxies.clear();
    proxies.reserve(positions.len());
    for (row, position) in positions.iter().copied().enumerate() {
        let tag = checked_tag(inverse_diameter * position.x, inverse_diameter * position.y)
            .map_err(ContactFillError::Proxy)?;
        proxies.push(ContactProxy { tag, row });
    }
    proxies.sort_by(|left, right| left.tag.cmp(&right.tag).then(left.row.cmp(&right.row)));
    Ok(())
}

fn consider_window<F: FnMut(&ParticleContact) -> bool>(
    a_row: usize,
    proxies: &[ContactProxy],
    write: &mut ContactWrite<'_, F>,
) {
    let a_position = write.positions[a_row];
    for proxy in proxies {
        let b_row = proxy.row;
        let difference = write.positions[b_row] - a_position;
        let distance_squared = difference.dot(difference);
        if distance_squared >= write.squared_diameter {
            continue;
        }
        commit_contact(write, a_row, b_row, difference, distance_squared);
    }
}

fn commit_contact<F: FnMut(&ParticleContact) -> bool>(
    write: &mut ContactWrite<'_, F>,
    a_row: usize,
    b_row: usize,
    difference: Vec2,
    distance_squared: f32,
) {
    let combined = write.flags[a_row] | write.flags[b_row];
    let inverse_distance = inverse_sqrt(distance_squared);
    let weight = 1.0 - distance_squared * inverse_distance * write.inverse_diameter;
    let normal = inverse_distance * difference;
    if combined.contains(ParticleFlags::PARTICLE_CONTACT_FILTER) {
        let contact = ParticleContact::new_internal(
            [write.ids[a_row], write.ids[b_row]],
            combined,
            weight,
            normal,
        );
        if !(write.filter)(&contact) {
            return;
        }
    }
    write.contacts.push(StoredParticleContact {
        indices: [
            super::storage::ParticleIndex(a_row),
            super::storage::ParticleIndex(b_row),
        ],
        flags: combined,
        weight,
        normal,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use crate::math::Vec2;
    use crate::particle::{ParticleContactUpdate, ParticleDef, ParticleNeighborhood};

    #[test]
    fn stepped_scan_matches_neighborhood_contacts() {
        // Arrange
        let mut world = World::new().expect("test world key remains available");
        let system = world
            .create_particle_system()
            .expect("particle system should fit");
        let create_at = |world: &mut World, position: Vec2| {
            let definition = ParticleDef::default()
                .with_position(position)
                .expect("test position should be finite");
            let _ = world
                .create_particle_with_def(system, None, &definition)
                .expect("particle should fit");
        };
        for y in 0_u8..3 {
            for x in 0_u8..3 {
                create_at(
                    &mut world,
                    Vec2::new(f32::from(x) * 0.5, f32::from(y) * 0.5),
                );
            }
        }
        let view = world
            .particle_system_view(system)
            .expect("particle system should remain live");
        let diameter = 1.0;
        let neighborhood = ParticleNeighborhood::from_view(&view, diameter)
            .expect("finite positions should build proxies");
        let expected = ParticleContactUpdate::generate(&view, &neighborhood, &[], |_contact| true)
            .expect("neighborhood contacts should generate");

        // Act
        let mut proxies = Vec::new();
        let mut contacts = Vec::new();
        fill_stored_contacts(
            view.positions(),
            view.flags(),
            view.particle_ids(),
            diameter,
            &mut proxies,
            &mut contacts,
            &mut |_contact| true,
        )
        .expect("scan should accept the same positions");

        // Assert
        assert_eq!(contacts.len(), expected.contacts().len());
        for (stored, semantic) in contacts.iter().zip(expected.contacts()) {
            let ids = [
                view.particle_ids()[stored.indices[0].0],
                view.particle_ids()[stored.indices[1].0],
            ];
            assert_eq!(ids, semantic.particles());
            assert_eq!(stored.weight.to_bits(), semantic.weight().to_bits());
            assert_eq!(stored.normal.x.to_bits(), semantic.normal().x.to_bits());
            assert_eq!(stored.normal.y.to_bits(), semantic.normal().y.to_bits());
        }
    }
}
