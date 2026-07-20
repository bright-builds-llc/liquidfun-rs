use std::ops::Range;

use crate::identity::{ParticleGroupId, ParticleId};
use crate::math::Transform;
use crate::particle::ParticleGroupFlags;
use crate::particle::topology::VoronoiLimits;
use crate::particle::topology::constraints::{
    ConnectionFilter, ConstraintError, TopologyGroup, TopologyInput, generate_pairs_and_triads,
};

use super::ParticleIndex;
use super::lanes::{ParticlePair, ParticleTriad};
use super::permutation::{
    PreparedPermutation, TopologyRemapMode, TopologyRemapPolicy, commit_prepared,
    prepare_permutation,
};
use super::{ParticleSnapshot, ParticleStorage, ParticleStorageError};

mod join;

use join::{JoinPlanError, JoinTopologyParameters};

#[derive(Debug, Clone, Copy)]
pub(crate) struct GroupPlanInput {
    pub(crate) group: ParticleGroupId,
    pub(crate) maybe_append_target: Option<ParticleGroupId>,
    pub(crate) flags: ParticleGroupFlags,
    pub(crate) strength: f32,
    pub(crate) transform: Transform,
    pub(crate) particle_diameter: f32,
    pub(crate) voronoi_limits: VoronoiLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GroupPlanError {
    Storage(ParticleStorageError),
    Topology,
}

impl From<ParticleStorageError> for GroupPlanError {
    fn from(error: ParticleStorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<ConstraintError> for GroupPlanError {
    fn from(_error: ConstraintError) -> Self {
        Self::Topology
    }
}

impl From<JoinPlanError> for GroupPlanError {
    fn from(error: JoinPlanError) -> Self {
        match error {
            JoinPlanError::Storage(error) => Self::Storage(error),
            JoinPlanError::Constraints(_error) => Self::Topology,
        }
    }
}

pub(crate) struct GroupPlan {
    candidate: ParticleStorage,
    result_group: ParticleGroupId,
}

impl GroupPlan {
    pub(crate) const fn result_group(&self) -> ParticleGroupId {
        self.result_group
    }

    pub(crate) fn commit_group(self, storage: &mut ParticleStorage) {
        *storage = self.candidate;
    }
}

struct CreateGroupFilter {
    range: Range<usize>,
}

impl ConnectionFilter for CreateGroupFilter {
    fn is_necessary(&self, index: ParticleIndex) -> bool {
        self.range.contains(&index.0)
    }

    fn should_create_pair(&self, indices: [ParticleIndex; 2]) -> bool {
        indices.iter().any(|index| self.range.contains(&index.0))
    }

    fn should_create_triad(&self, indices: [ParticleIndex; 3]) -> bool {
        indices.iter().any(|index| self.range.contains(&index.0))
    }
}

impl ParticleStorage {
    pub(crate) fn plan_group(&self, input: GroupPlanInput) -> Result<GroupPlan, GroupPlanError> {
        self.check_invariants()?;
        let mut candidate = self.clone();
        let record = candidate
            .group_records
            .iter_mut()
            .find(|record| record.id == input.group)
            .ok_or(ParticleStorageError::InvalidGroupRange)?;
        record.flags = input.flags;
        record.strength = input.strength;
        record.transform = input.transform;
        let range = record.range();
        candidate
            .solver_state
            .refresh_group_flags(&candidate.group_records);

        let groups = candidate
            .groups
            .iter()
            .map(|maybe_group| {
                maybe_group.and_then(|group| {
                    candidate
                        .group_records
                        .iter()
                        .find(|record| record.id == group)
                        .copied()
                        .map(TopologyGroup::from_record)
                })
            })
            .collect::<Vec<_>>();
        let generated = generate_pairs_and_triads(
            &TopologyInput {
                owner: candidate.system,
                positions: &candidate.positions,
                flags: &candidate.flags,
                groups: &groups,
                contacts: &candidate.particle_contacts,
                range: range.clone(),
                particle_diameter: input.particle_diameter,
                voronoi_limits: input.voronoi_limits,
            },
            &CreateGroupFilter { range },
        )?;
        MutationCandidate::prepare_create_group(&candidate, generated.pairs, generated.triads)?
            .commit(&mut candidate);

        let result_group = if let Some(target) = input.maybe_append_target {
            let join = candidate.plan_join(
                target,
                input.group,
                JoinTopologyParameters::new(input.particle_diameter, input.voronoi_limits),
            )?;
            join.commit(&mut candidate);
            target
        } else {
            input.group
        };
        candidate.check_invariants()?;
        Ok(GroupPlan {
            candidate,
            result_group,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MutationCandidateKind {
    CreateGroup,
    JoinGroups,
    SplitGroup,
    ZombieCompaction,
    ReactiveRegeneration,
    GroupFlagChange,
    OrdinaryRotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupStatisticsInvalidation {
    Preserve,
    InvalidateAffected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DepthInvalidation {
    Preserve,
    InvalidateAffected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MutationInvalidations {
    group_statistics: GroupStatisticsInvalidation,
    depth: DepthInvalidation,
}

impl MutationInvalidations {
    const PRESERVE: Self = Self {
        group_statistics: GroupStatisticsInvalidation::Preserve,
        depth: DepthInvalidation::Preserve,
    };

    const AFFECTED_GROUPS: Self = Self {
        group_statistics: GroupStatisticsInvalidation::InvalidateAffected,
        depth: DepthInvalidation::InvalidateAffected,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MutationLifecycleEffect {
    ParticleDestroyed(ParticleId),
}

struct MutationPayload {
    permutation: PreparedPermutation,
    topology_mode: TopologyRemapMode,
    invalidations: MutationInvalidations,
    lifecycle_effects: Vec<MutationLifecycleEffect>,
}

pub(super) struct CreateGroupCandidate {
    payload: MutationPayload,
}

pub(super) struct JoinGroupsCandidate {
    payload: MutationPayload,
}

pub(super) struct SplitGroupCandidate {
    payload: MutationPayload,
}

pub(super) struct ZombieCompactionCandidate {
    payload: MutationPayload,
}

pub(super) struct ReactiveRegenerationCandidate {
    payload: MutationPayload,
}

pub(super) struct GroupFlagChangeCandidate {
    payload: MutationPayload,
}

pub(super) struct OrdinaryRotationCandidate {
    payload: MutationPayload,
}

pub(super) enum MutationCandidate {
    CreateGroup(CreateGroupCandidate),
    JoinGroups(JoinGroupsCandidate),
    SplitGroup(SplitGroupCandidate),
    ZombieCompaction(ZombieCompactionCandidate),
    ReactiveRegeneration(ReactiveRegenerationCandidate),
    GroupFlagChange(GroupFlagChangeCandidate),
    OrdinaryRotation(OrdinaryRotationCandidate),
}

pub(super) struct MutationCommitOutcome {
    pub(super) destroyed: Vec<ParticleSnapshot>,
}

impl MutationCandidate {
    pub(super) fn prepare_create_group(
        storage: &ParticleStorage,
        pairs: Vec<ParticlePair>,
        triads: Vec<ParticleTriad>,
    ) -> Result<Self, ParticleStorageError> {
        let payload = prepare_identity_payload(
            storage,
            TopologyRemapPolicy::AppendStableSortFirstDuplicate { pairs, triads },
            MutationInvalidations::AFFECTED_GROUPS,
        )?;
        Ok(Self::CreateGroup(CreateGroupCandidate { payload }))
    }

    pub(super) fn prepare_join_groups(
        storage: &ParticleStorage,
        old_to_new: &[Option<usize>],
        pairs: Vec<ParticlePair>,
        triads: Vec<ParticleTriad>,
    ) -> Result<Self, ParticleStorageError> {
        require_no_removals(old_to_new)?;
        let payload = prepare_payload(
            storage,
            old_to_new,
            TopologyRemapPolicy::AppendStableSortFirstDuplicate { pairs, triads },
            MutationInvalidations::AFFECTED_GROUPS,
            false,
        )?;
        Ok(Self::JoinGroups(JoinGroupsCandidate { payload }))
    }

    pub(super) fn prepare_exact_join_groups(
        storage: &ParticleStorage,
        old_to_new: &[Option<usize>],
        pairs: Vec<ParticlePair>,
        triads: Vec<ParticleTriad>,
    ) -> Result<Self, ParticleStorageError> {
        require_no_removals(old_to_new)?;
        let payload = prepare_payload(
            storage,
            old_to_new,
            TopologyRemapPolicy::AppendPreservingHistoricalOrder(pairs, triads),
            MutationInvalidations::AFFECTED_GROUPS,
            false,
        )?;
        Ok(Self::JoinGroups(JoinGroupsCandidate { payload }))
    }

    pub(super) fn prepare_split_group(
        storage: &ParticleStorage,
        old_to_new: &[Option<usize>],
    ) -> Result<Self, ParticleStorageError> {
        require_no_removals(old_to_new)?;
        let payload = prepare_payload(
            storage,
            old_to_new,
            TopologyRemapPolicy::PreserveHistoricalOrder,
            MutationInvalidations::AFFECTED_GROUPS,
            false,
        )?;
        Ok(Self::SplitGroup(SplitGroupCandidate { payload }))
    }

    pub(super) fn prepare_zombie_compaction(
        storage: &ParticleStorage,
        old_to_new: &[Option<usize>],
    ) -> Result<Self, ParticleStorageError> {
        let payload = prepare_payload(
            storage,
            old_to_new,
            TopologyRemapPolicy::PreserveHistoricalOrder,
            MutationInvalidations::AFFECTED_GROUPS,
            true,
        )?;
        Ok(Self::ZombieCompaction(ZombieCompactionCandidate {
            payload,
        }))
    }

    pub(super) fn prepare_reactive_regeneration(
        storage: &ParticleStorage,
        pairs: Vec<ParticlePair>,
        triads: Vec<ParticleTriad>,
    ) -> Result<Self, ParticleStorageError> {
        let payload = prepare_identity_payload(
            storage,
            TopologyRemapPolicy::AppendStableSortFirstDuplicate { pairs, triads },
            MutationInvalidations::PRESERVE,
        )?;
        Ok(Self::ReactiveRegeneration(ReactiveRegenerationCandidate {
            payload,
        }))
    }

    pub(super) fn prepare_group_flag_change(
        storage: &ParticleStorage,
    ) -> Result<Self, ParticleStorageError> {
        let payload = prepare_identity_payload(
            storage,
            TopologyRemapPolicy::PreserveHistoricalOrder,
            MutationInvalidations::AFFECTED_GROUPS,
        )?;
        Ok(Self::GroupFlagChange(GroupFlagChangeCandidate { payload }))
    }

    pub(super) fn prepare_ordinary_rotation(
        storage: &ParticleStorage,
        start: usize,
        middle: usize,
        end: usize,
    ) -> Result<Self, ParticleStorageError> {
        let old_to_new = rotation_mapping(storage.len(), start, middle, end)?;
        let payload = prepare_payload(
            storage,
            &old_to_new,
            TopologyRemapPolicy::PreserveHistoricalOrder,
            MutationInvalidations::PRESERVE,
            false,
        )?;
        Ok(Self::OrdinaryRotation(OrdinaryRotationCandidate {
            payload,
        }))
    }

    pub(super) const fn kind(&self) -> MutationCandidateKind {
        match self {
            Self::CreateGroup(_) => MutationCandidateKind::CreateGroup,
            Self::JoinGroups(_) => MutationCandidateKind::JoinGroups,
            Self::SplitGroup(_) => MutationCandidateKind::SplitGroup,
            Self::ZombieCompaction(_) => MutationCandidateKind::ZombieCompaction,
            Self::ReactiveRegeneration(_) => MutationCandidateKind::ReactiveRegeneration,
            Self::GroupFlagChange(_) => MutationCandidateKind::GroupFlagChange,
            Self::OrdinaryRotation(_) => MutationCandidateKind::OrdinaryRotation,
        }
    }

    pub(super) fn commit(self, storage: &mut ParticleStorage) -> MutationCommitOutcome {
        let payload = self.into_payload();
        debug_assert_eq!(
            payload.lifecycle_effects.len(),
            payload.permutation.destroyed().len()
        );
        let destroyed = commit_prepared(storage, payload.permutation);
        MutationCommitOutcome { destroyed }
    }

    fn payload(&self) -> &MutationPayload {
        match self {
            Self::CreateGroup(candidate) => &candidate.payload,
            Self::JoinGroups(candidate) => &candidate.payload,
            Self::SplitGroup(candidate) => &candidate.payload,
            Self::ZombieCompaction(candidate) => &candidate.payload,
            Self::ReactiveRegeneration(candidate) => &candidate.payload,
            Self::GroupFlagChange(candidate) => &candidate.payload,
            Self::OrdinaryRotation(candidate) => &candidate.payload,
        }
    }

    fn into_payload(self) -> MutationPayload {
        match self {
            Self::CreateGroup(candidate) => candidate.payload,
            Self::JoinGroups(candidate) => candidate.payload,
            Self::SplitGroup(candidate) => candidate.payload,
            Self::ZombieCompaction(candidate) => candidate.payload,
            Self::ReactiveRegeneration(candidate) => candidate.payload,
            Self::GroupFlagChange(candidate) => candidate.payload,
            Self::OrdinaryRotation(candidate) => candidate.payload,
        }
    }
}

fn prepare_identity_payload(
    storage: &ParticleStorage,
    topology_policy: TopologyRemapPolicy,
    invalidations: MutationInvalidations,
) -> Result<MutationPayload, ParticleStorageError> {
    let old_to_new = (0..storage.len()).map(Some).collect::<Vec<_>>();
    prepare_payload(storage, &old_to_new, topology_policy, invalidations, false)
}

fn prepare_payload(
    storage: &ParticleStorage,
    old_to_new: &[Option<usize>],
    topology_policy: TopologyRemapPolicy,
    invalidations: MutationInvalidations,
    allow_destruction: bool,
) -> Result<MutationPayload, ParticleStorageError> {
    let topology_mode = topology_policy.mode();
    let permutation = prepare_permutation(storage, old_to_new, topology_policy)?;
    if !allow_destruction && !permutation.destroyed().is_empty() {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    let lifecycle_effects = destruction_effects(&permutation)?;
    Ok(MutationPayload {
        permutation,
        topology_mode,
        invalidations,
        lifecycle_effects,
    })
}

fn destruction_effects(
    permutation: &PreparedPermutation,
) -> Result<Vec<MutationLifecycleEffect>, ParticleStorageError> {
    let mut effects = Vec::new();
    effects
        .try_reserve_exact(permutation.destroyed().len())
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    effects.extend(
        permutation
            .destroyed()
            .iter()
            .map(|snapshot| MutationLifecycleEffect::ParticleDestroyed(snapshot.id)),
    );
    Ok(effects)
}

fn require_no_removals(old_to_new: &[Option<usize>]) -> Result<(), ParticleStorageError> {
    if old_to_new.iter().any(Option::is_none) {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    Ok(())
}

fn rotation_mapping(
    particle_count: usize,
    start: usize,
    middle: usize,
    end: usize,
) -> Result<Vec<Option<usize>>, ParticleStorageError> {
    if start > middle || middle > end || end > particle_count {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    let mut old_to_new = (0..particle_count).map(Some).collect::<Vec<_>>();
    for (old, destination) in old_to_new.iter_mut().enumerate().take(middle).skip(start) {
        *destination = Some(old + end - middle);
    }
    for (old, destination) in old_to_new.iter_mut().enumerate().take(end).skip(middle) {
        *destination = Some(old + start - middle);
    }
    Ok(old_to_new)
}

#[cfg(test)]
mod tests {
    use crate::identity::{HandleIdentity, Identity, ParticleGroupId, ParticleSystemId, WorldKey};
    use crate::math::Vec2;
    use crate::particle::ParticleFlags;

    use super::*;
    use crate::particle::storage::{ParticleIndex, ParticleInput};

    fn storage() -> ParticleStorage {
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        let group = ParticleGroupId::from_identity(Identity::new(world, 1, 0));
        let mut storage =
            ParticleStorage::new(world, system, 0, 8, 8).expect("storage contract is valid");
        for value in [0.0, 1.0, 2.0, 3.0] {
            storage
                .create(ParticleInput {
                    position: Vec2::new(value, -value),
                    velocity: Vec2::ZERO,
                    flags: ParticleFlags::SPRING | ParticleFlags::ELASTIC,
                    maybe_group: Some(group),
                    maybe_color: None,
                    maybe_user_association: None,
                    maybe_expiration_time: None,
                })
                .expect("fixture particle fits");
        }
        storage.pairs = vec![pair([0, 1], 10.0), pair([2, 3], 20.0), pair([1, 2], 30.0)];
        storage.triads = vec![triad([0, 1, 2], 10.0), triad([1, 2, 3], 20.0)];
        storage
    }

    fn pair(indices: [usize; 2], rest: f32) -> ParticlePair {
        ParticlePair {
            indices: indices.map(ParticleIndex),
            flags: ParticleFlags::SPRING,
            strength: rest + 0.25,
            distance: rest,
        }
    }

    fn triad(indices: [usize; 3], rest: f32) -> ParticleTriad {
        ParticleTriad {
            indices: indices.map(ParticleIndex),
            flags: ParticleFlags::ELASTIC,
            strength: rest + 0.5,
            pa: Vec2::new(rest + 1.0, rest + 2.0),
            pb: Vec2::new(rest + 3.0, rest + 4.0),
            pc: Vec2::new(rest + 5.0, rest + 6.0),
            ka: rest + 7.0,
            kb: rest + 8.0,
            kc: rest + 9.0,
            s: rest + 10.0,
        }
    }

    fn pair_rest_bits(storage: &ParticleStorage) -> Vec<(u32, u32)> {
        storage
            .pairs
            .iter()
            .map(|pair| (pair.strength.to_bits(), pair.distance.to_bits()))
            .collect()
    }

    fn triad_rest_bits(storage: &ParticleStorage) -> Vec<[u32; 11]> {
        storage
            .triads
            .iter()
            .map(|triad| {
                [
                    triad.strength.to_bits(),
                    triad.pa.x.to_bits(),
                    triad.pa.y.to_bits(),
                    triad.pb.x.to_bits(),
                    triad.pb.y.to_bits(),
                    triad.pc.x.to_bits(),
                    triad.pc.y.to_bits(),
                    triad.ka.to_bits(),
                    triad.kb.to_bits(),
                    triad.kc.to_bits(),
                    triad.s.to_bits(),
                ]
            })
            .collect()
    }

    #[test]
    fn candidate_kind_is_closed_over_all_seven_operations() {
        // Arrange
        let storage = storage();
        let identity = (0..storage.len()).map(Some).collect::<Vec<_>>();
        let candidates = [
            MutationCandidate::prepare_create_group(&storage, Vec::new(), Vec::new())
                .expect("create candidate validates"),
            MutationCandidate::prepare_join_groups(&storage, &identity, Vec::new(), Vec::new())
                .expect("join candidate validates"),
            MutationCandidate::prepare_split_group(&storage, &identity)
                .expect("split candidate validates"),
            MutationCandidate::prepare_zombie_compaction(&storage, &identity)
                .expect("compaction candidate validates"),
            MutationCandidate::prepare_reactive_regeneration(&storage, Vec::new(), Vec::new())
                .expect("reactive candidate validates"),
            MutationCandidate::prepare_group_flag_change(&storage)
                .expect("flag candidate validates"),
            MutationCandidate::prepare_ordinary_rotation(&storage, 0, 2, 4)
                .expect("rotation candidate validates"),
        ];

        // Act
        let kinds = candidates.map(|candidate| candidate.kind());

        // Assert
        assert_eq!(
            kinds,
            [
                MutationCandidateKind::CreateGroup,
                MutationCandidateKind::JoinGroups,
                MutationCandidateKind::SplitGroup,
                MutationCandidateKind::ZombieCompaction,
                MutationCandidateKind::ReactiveRegeneration,
                MutationCandidateKind::GroupFlagChange,
                MutationCandidateKind::OrdinaryRotation,
            ]
        );
    }

    #[test]
    fn ordinary_rotation_preserves_topology_order_and_every_rest_bit() {
        // Arrange
        let mut storage = storage();
        let pair_bits = pair_rest_bits(&storage);
        let triad_bits = triad_rest_bits(&storage);

        // Act
        let candidate = MutationCandidate::prepare_ordinary_rotation(&storage, 0, 2, 4)
            .expect("rotation candidate validates");
        assert_eq!(
            candidate.payload().topology_mode,
            TopologyRemapMode::PreserveHistoricalOrder
        );
        candidate.commit(&mut storage);

        // Assert
        assert_eq!(pair_rest_bits(&storage), pair_bits);
        assert_eq!(triad_rest_bits(&storage), triad_bits);
        assert_eq!(
            storage
                .pairs
                .iter()
                .map(|pair| pair.indices.map(|index| index.0))
                .collect::<Vec<_>>(),
            vec![[2, 3], [0, 1], [3, 0]]
        );
    }

    #[test]
    fn split_retarget_preserves_topology_order_and_every_rest_bit() {
        // Arrange
        let mut storage = storage();
        let mapping = [Some(1), Some(0), Some(3), Some(2)];
        let pair_bits = pair_rest_bits(&storage);
        let triad_bits = triad_rest_bits(&storage);

        // Act
        let candidate = MutationCandidate::prepare_split_group(&storage, &mapping)
            .expect("split candidate validates");
        candidate.commit(&mut storage);

        // Assert
        assert_eq!(pair_rest_bits(&storage), pair_bits);
        assert_eq!(triad_rest_bits(&storage), triad_bits);
        assert_eq!(
            storage
                .triads
                .iter()
                .map(|triad| triad.indices.map(|index| index.0))
                .collect::<Vec<_>>(),
            vec![[1, 0, 3], [0, 3, 2]]
        );
    }

    #[test]
    fn append_policy_stable_sorts_and_keeps_the_first_duplicate() {
        // Arrange
        let mut storage = storage();
        let existing_duplicate_bits = storage.pairs[0].distance.to_bits();
        let appended = vec![pair([3, 0], 40.0), pair([0, 1], 99.0)];

        // Act
        let candidate =
            MutationCandidate::prepare_reactive_regeneration(&storage, appended, Vec::new())
                .expect("reactive candidate validates");
        assert_eq!(
            candidate.payload().topology_mode,
            TopologyRemapMode::AppendStableSortFirstDuplicate
        );
        candidate.commit(&mut storage);

        // Assert
        assert_eq!(
            storage
                .pairs
                .iter()
                .map(|pair| pair.indices.map(|index| index.0))
                .collect::<Vec<_>>(),
            vec![[0, 1], [1, 2], [2, 3], [3, 0]]
        );
        assert_eq!(storage.pairs[0].distance.to_bits(), existing_duplicate_bits);
    }

    #[test]
    fn invalid_rotation_and_non_finite_append_leave_storage_unchanged() {
        // Arrange
        let storage = storage();
        let before = storage.clone();
        let invalid_pair = ParticlePair {
            distance: f32::NAN,
            ..pair([0, 1], 1.0)
        };

        // Act
        let range_result =
            MutationCandidate::prepare_ordinary_rotation(&storage, 0, 5, storage.len());
        let topology_result = MutationCandidate::prepare_join_groups(
            &storage,
            &[Some(0), Some(1), Some(2), Some(3)],
            vec![invalid_pair],
            Vec::new(),
        );

        // Assert
        assert!(matches!(
            range_result,
            Err(ParticleStorageError::InvalidPermutation)
        ));
        assert!(matches!(
            topology_result,
            Err(ParticleStorageError::InvalidLaneBundle)
        ));
        assert!(storage == before);
    }
}
