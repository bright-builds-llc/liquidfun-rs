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

/// Last sorted proxy order, kept across solver iterations only as a sort hint.
///
/// It never changes results: `(tag, row)` is a total order, so sorting any
/// permutation of the rows yields the same buffer. Equality ignores it.
#[derive(Debug, Clone, Default)]
pub(crate) struct ProxyOrderCache(pub(crate) Vec<ContactProxy>);

impl PartialEq for ProxyOrderCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
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

    if retag_retained_order(positions, inverse_diameter, proxies) {
        proxies.sort_unstable_by_key(|proxy| (proxy.tag, proxy.row));
        return Ok(());
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

/// Recomputes tags in place on the previous iteration's sorted order.
///
/// A retained buffer whose length equals `positions.len()` is a permutation of
/// rows `0..len`: it only ever comes from a full rebuild at that length or a
/// re-sort of one. Creation, destruction and compaction change which particle a
/// row names but keep the rows a permutation while the length matches. Returns
/// false when the caller must rebuild in row order instead, including on any
/// tag error, so errors still come from the first failing row in row order.
fn retag_retained_order(
    positions: &[Vec2],
    inverse_diameter: f32,
    proxies: &mut [ContactProxy],
) -> bool {
    if positions.is_empty() || proxies.len() != positions.len() {
        return false;
    }
    for proxy in proxies.iter_mut() {
        let Some(position) = positions.get(proxy.row) else {
            return false;
        };
        let Ok(tag) = checked_tag(inverse_diameter * position.x, inverse_diameter * position.y)
        else {
            return false;
        };
        proxy.tag = tag;
    }
    true
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

    const DIAMETER: f32 = 1.0;

    /// Flags and ids for `count` live particles; positions are supplied per scan.
    fn lanes(count: usize) -> (Vec<ParticleFlags>, Vec<ParticleId>) {
        let mut world = World::new().expect("test world key remains available");
        let system = world
            .create_particle_system()
            .expect("particle system should fit");
        for _ in 0..count {
            let _ = world
                .create_particle_with_def(system, None, &ParticleDef::default())
                .expect("particle should fit");
        }
        let view = world
            .particle_system_view(system)
            .expect("particle system should remain live");
        (view.flags().to_vec(), view.particle_ids().to_vec())
    }

    /// An irregular cluster so neighbors and tags vary from row to row.
    fn scattered_positions(count: usize) -> Vec<Vec2> {
        (0..count)
            .map(|row| {
                let step = f32::from(u16::try_from(row).expect("test rows fit in u16"));
                Vec2::new((step * 0.37) % 3.1 - 1.2, (step * 0.61) % 2.3 - 0.4)
            })
            .collect()
    }

    fn mirrored(positions: &[Vec2]) -> Vec<Vec2> {
        positions
            .iter()
            .map(|position| Vec2::new(-position.x, position.y + 0.25))
            .collect()
    }

    fn scan(
        positions: &[Vec2],
        lanes: &(Vec<ParticleFlags>, Vec<ParticleId>),
        diameter: f32,
        proxies: &mut Vec<ContactProxy>,
    ) -> (Result<(), ContactFillError>, Vec<StoredParticleContact>) {
        let mut contacts = Vec::new();
        let result = fill_stored_contacts(
            positions,
            &lanes.0[..positions.len()],
            &lanes.1[..positions.len()],
            diameter,
            proxies,
            &mut contacts,
            &mut |_contact| true,
        );
        (result, contacts)
    }

    fn sorted_scan(
        positions: &[Vec2],
        lanes: &(Vec<ParticleFlags>, Vec<ParticleId>),
    ) -> Vec<ContactProxy> {
        let mut proxies = Vec::new();
        let (result, _contacts) = scan(positions, lanes, DIAMETER, &mut proxies);
        result.expect("in-range positions should scan");
        proxies
    }

    #[test]
    fn retained_order_matches_fresh_rebuild_after_motion() {
        // Arrange
        let lanes = lanes(40);
        let before = scattered_positions(40);
        let after = mirrored(&before);
        let mut retained = sorted_scan(&before, &lanes);
        let mut fresh = Vec::new();

        // Act
        let (retained_result, retained_contacts) = scan(&after, &lanes, DIAMETER, &mut retained);
        let (fresh_result, fresh_contacts) = scan(&after, &lanes, DIAMETER, &mut fresh);

        // Assert
        retained_result.expect("retained order should scan");
        fresh_result.expect("fresh order should scan");
        assert_ne!(sorted_scan(&before, &lanes), fresh);
        assert_eq!(retained, fresh);
        assert!(!fresh_contacts.is_empty());
        assert_eq!(retained_contacts, fresh_contacts);
    }

    #[test]
    fn retained_order_with_other_length_falls_back() {
        // Arrange
        let lanes = lanes(40);
        let positions = scattered_positions(40);
        let mut retained = sorted_scan(&positions[..39], &lanes);
        let mut fresh = Vec::new();

        // Act
        let (retained_result, _) = scan(&positions, &lanes, DIAMETER, &mut retained);
        let (fresh_result, _) = scan(&positions, &lanes, DIAMETER, &mut fresh);

        // Assert
        retained_result.expect("retained order should scan");
        fresh_result.expect("fresh order should scan");
        assert_eq!(retained.len(), 40);
        assert_eq!(retained, fresh);
    }

    #[test]
    fn retained_order_reports_the_same_tag_error() {
        // Arrange
        let lanes = lanes(40);
        let mut positions = scattered_positions(40);
        let mut retained = sorted_scan(&positions, &lanes);
        positions[17] = Vec2::new(1.0e30, 0.0);
        let mut fresh = Vec::new();

        // Act
        let (retained_result, _) = scan(&positions, &lanes, DIAMETER, &mut retained);
        let (fresh_result, _) = scan(&positions, &lanes, DIAMETER, &mut fresh);

        // Assert
        assert!(matches!(
            retained_result,
            Err(ContactFillError::Proxy(
                ParticleProxyError::PositionOutOfTagRange
            ))
        ));
        assert!(matches!(
            fresh_result,
            Err(ContactFillError::Proxy(
                ParticleProxyError::PositionOutOfTagRange
            ))
        ));
        assert_eq!(retained, fresh);
    }

    #[test]
    fn retained_order_reports_the_same_diameter_error() {
        // Arrange
        let lanes = lanes(40);
        let positions = scattered_positions(40);
        let mut retained = sorted_scan(&positions, &lanes);
        let expected = retained.clone();

        // Act
        let (result, _) = scan(&positions, &lanes, 0.0, &mut retained);

        // Assert
        assert!(matches!(
            result,
            Err(ContactFillError::Proxy(
                ParticleProxyError::NonPositiveDiameter
            ))
        ));
        assert_eq!(retained, expected);
    }

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
