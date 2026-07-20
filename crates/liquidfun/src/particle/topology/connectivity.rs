use std::ops::Range;

use crate::particle::ParticleFlags;
use crate::particle::storage::lanes::ParticleContact;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectivityError {
    InvalidGroupRange,
    InvalidContactEndpoint,
    ConnectedZombie,
    AllocationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::particle) struct SplitConnectivityPlan {
    source_range: Range<usize>,
    first_longest: usize,
    surviving_members: Vec<usize>,
    later_components: Vec<Vec<usize>>,
}

impl SplitConnectivityPlan {
    pub(in crate::particle) fn first_longest(&self) -> usize {
        self.first_longest
    }

    pub(in crate::particle) fn surviving_members(&self) -> &[usize] {
        &self.surviving_members
    }

    pub(in crate::particle) fn later_components(&self) -> &[Vec<usize>] {
        &self.later_components
    }

    pub(in crate::particle) fn moved_members(&self) -> impl Iterator<Item = usize> + '_ {
        self.later_components.iter().flatten().copied()
    }

    pub(in crate::particle) fn component_count(&self) -> usize {
        1 + self.later_components.len()
    }

    pub(in crate::particle) fn source_range(&self) -> Range<usize> {
        self.source_range.clone()
    }
}

struct ParticleLists {
    owner: Vec<usize>,
    next: Vec<Option<usize>>,
    counts: Vec<usize>,
}

impl ParticleLists {
    fn new(count: usize) -> Result<Self, ConnectivityError> {
        let mut owner = Vec::new();
        owner
            .try_reserve_exact(count)
            .map_err(|_error| ConnectivityError::AllocationFailed)?;
        owner.extend(0..count);
        let mut next = Vec::new();
        next.try_reserve_exact(count)
            .map_err(|_error| ConnectivityError::AllocationFailed)?;
        next.resize(count, None);
        let mut counts = Vec::new();
        counts
            .try_reserve_exact(count)
            .map_err(|_error| ConnectivityError::AllocationFailed)?;
        counts.resize(count, 1);
        Ok(Self {
            owner,
            next,
            counts,
        })
    }

    fn merge_lists(&mut self, mut list_a: usize, mut list_b: usize) {
        if self.counts[list_a] < self.counts[list_b] {
            std::mem::swap(&mut list_a, &mut list_b);
        }
        let mut tail_b = list_b;
        loop {
            self.owner[tail_b] = list_a;
            let Some(next_b) = self.next[tail_b] else {
                break;
            };
            tail_b = next_b;
        }
        self.next[tail_b] = self.next[list_a];
        self.next[list_a] = Some(list_b);
        self.counts[list_a] += self.counts[list_b];
        self.counts[list_b] = 0;
    }

    fn merge_singleton(&mut self, list: usize, node: usize) -> Result<(), ConnectivityError> {
        if self.owner[node] != node || self.counts[node] != 1 {
            return Err(ConnectivityError::ConnectedZombie);
        }
        self.owner[node] = list;
        self.next[node] = self.next[list];
        self.next[list] = Some(node);
        self.counts[list] += 1;
        self.counts[node] = 0;
        Ok(())
    }

    fn members(&self, head: usize) -> Result<Vec<usize>, ConnectivityError> {
        let count = self.counts[head];
        let mut members = Vec::new();
        members
            .try_reserve_exact(count)
            .map_err(|_error| ConnectivityError::AllocationFailed)?;
        let mut maybe_node = Some(head);
        while let Some(node) = maybe_node {
            members.push(node);
            maybe_node = self.next[node];
        }
        debug_assert_eq!(members.len(), count);
        Ok(members)
    }
}

pub(in crate::particle) fn plan_split_connectivity(
    source_range: Range<usize>,
    contacts: &[ParticleContact],
    flags: &[ParticleFlags],
) -> Result<Option<SplitConnectivityPlan>, ConnectivityError> {
    if source_range.start > source_range.end || source_range.end > flags.len() {
        return Err(ConnectivityError::InvalidGroupRange);
    }
    if contacts
        .iter()
        .flat_map(|contact| contact.indices)
        .any(|index| index.0 >= flags.len())
    {
        return Err(ConnectivityError::InvalidContactEndpoint);
    }
    let particle_count = source_range.len();
    if particle_count == 0 {
        return Ok(None);
    }

    let mut lists = ParticleLists::new(particle_count)?;
    for contact in contacts {
        let [a, b] = contact.indices.map(|index| index.0);
        if !source_range.contains(&a) || !source_range.contains(&b) {
            continue;
        }
        let list_a = lists.owner[a - source_range.start];
        let list_b = lists.owner[b - source_range.start];
        if list_a != list_b {
            lists.merge_lists(list_a, list_b);
        }
    }

    let first_longest = lists
        .counts
        .iter()
        .enumerate()
        .fold(0, |current, (candidate, count)| {
            if lists.counts[current] < *count {
                candidate
            } else {
                current
            }
        });
    for (local, particle_flags) in flags[source_range.clone()].iter().copied().enumerate() {
        if local != first_longest && particle_flags.contains(ParticleFlags::ZOMBIE) {
            lists.merge_singleton(first_longest, local)?;
        }
    }

    let surviving_members = lists
        .owner
        .iter()
        .enumerate()
        .filter_map(|(local, owner)| {
            (*owner == first_longest).then_some(source_range.start + local)
        })
        .collect::<Vec<_>>();
    let mut later_components = Vec::new();
    for head in 0..particle_count {
        if head == first_longest || lists.counts[head] == 0 {
            continue;
        }
        let members = lists
            .members(head)?
            .into_iter()
            .map(|local| source_range.start + local)
            .collect();
        later_components.push(members);
    }
    Ok(Some(SplitConnectivityPlan {
        source_range,
        first_longest,
        surviving_members,
        later_components,
    }))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::math::Vec2;
    use crate::particle::storage::ParticleIndex;

    use super::*;

    fn contact(a: usize, b: usize) -> ParticleContact {
        ParticleContact {
            indices: [ParticleIndex(a), ParticleIndex(b)],
            flags: ParticleFlags::WATER,
            weight: 0.5,
            normal: Vec2::new(1.0, 0.0),
        }
    }

    #[test]
    fn equal_length_union_retains_contact_list_a() {
        // Arrange
        let contacts = [contact(0, 1), contact(2, 3), contact(3, 1)];

        // Act
        let plan = plan_split_connectivity(0..4, &contacts, &[ParticleFlags::WATER; 4])
            .expect("connectivity is valid")
            .expect("nonempty range has a plan");

        // Assert
        assert_eq!(plan.first_longest(), 2);
        assert_eq!(plan.surviving_members(), &[0, 1, 2, 3]);
    }

    #[test]
    fn first_longest_survives_and_zombies_merge_in_reverse_insertion_order() {
        // Arrange
        let contacts = [contact(0, 2), contact(1, 3)];
        let mut flags = [ParticleFlags::WATER; 6];
        flags[4] = ParticleFlags::ZOMBIE;
        flags[5] = ParticleFlags::ZOMBIE;

        // Act
        let plan = plan_split_connectivity(0..6, &contacts, &flags)
            .expect("connectivity is valid")
            .expect("nonempty range has a plan");

        // Assert
        assert_eq!(plan.first_longest(), 0);
        assert_eq!(plan.surviving_members(), &[0, 2, 4, 5]);
        assert_eq!(plan.later_components(), &[vec![1, 3]]);
    }

    #[test]
    fn contacts_with_intermingled_other_groups_are_ignored() {
        // Arrange
        let contacts = [contact(1, 4), contact(1, 2), contact(2, 5)];

        // Act
        let plan = plan_split_connectivity(1..4, &contacts, &[ParticleFlags::WATER; 6])
            .expect("connectivity is valid")
            .expect("nonempty range has a plan");

        // Assert
        assert_eq!(plan.surviving_members(), &[1, 2]);
        assert_eq!(plan.later_components(), &[vec![3]]);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        #[test]
        fn bounded_plans_partition_every_source_member_once(
            particle_count in 1_usize..24,
            raw_contacts in prop::collection::vec((0_usize..24, 0_usize..24), 0..48),
            zombie_ordinals in prop::collection::vec(0_usize..24, 0..12),
        ) {
            // Arrange
            let contacts = raw_contacts
                .into_iter()
                .map(|(a, b)| contact(a % particle_count, b % particle_count))
                .collect::<Vec<_>>();
            let mut flags = vec![ParticleFlags::WATER; particle_count];
            for ordinal in zombie_ordinals {
                let local = ordinal % particle_count;
                let contacted = contacts.iter().any(|contact| {
                    contact.indices.iter().any(|index| index.0 == local)
                });
                if !contacted {
                    flags[local] = ParticleFlags::ZOMBIE;
                }
            }

            // Act
            let plan = plan_split_connectivity(0..particle_count, &contacts, &flags)
                .expect("bounded inputs are valid")
                .expect("positive count has a plan");

            // Assert
            let mut members = plan.surviving_members().to_vec();
            members.extend(plan.moved_members());
            members.sort_unstable();
            prop_assert_eq!(members, (0..particle_count).collect::<Vec<_>>());
            prop_assert!(
                flags
                    .iter()
                    .enumerate()
                    .filter(|(_, flags)| flags.contains(ParticleFlags::ZOMBIE))
                    .all(|(index, _)| plan.surviving_members().contains(&index))
            );
        }
    }
}
