use super::super::mutation::{MutationCandidate, MutationCandidateKind};
use super::super::*;
use super::{input, ordinary_storage};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use std::panic::{AssertUnwindSafe, catch_unwind};
const ROW_COUNT: usize = 6;
const MODEL_SEED: u64 = 0x1007_5eed;
const ROLLBACK_SEED: u64 = 0x1007_fa17;
#[derive(Debug, Clone, Copy)]
enum Command {
    CreateGroup(u8),
    Rotate([u8; 3]),
    Join([u8; 3], u8),
    Split([u8; 3]),
    MarkZombie(u8),
    Compact,
    Reactive(u8),
    FlagChange,
}
fn command_strategy() -> impl Strategy<Value = Command> {
    prop_oneof![
        any::<u8>().prop_map(Command::CreateGroup),
        any::<[u8; 3]>().prop_map(Command::Rotate),
        (any::<[u8; 3]>(), any::<u8>()).prop_map(|(range, rest)| Command::Join(range, rest)),
        any::<[u8; 3]>().prop_map(Command::Split),
        any::<u8>().prop_map(Command::MarkZombie),
        Just(Command::Compact),
        any::<u8>().prop_map(Command::Reactive),
        Just(Command::FlagChange),
    ]
}
#[derive(Debug, Clone, Copy)]
enum InvalidRequest {
    ForeignHandle,
    InvalidRange(u8),
    Capacity,
    NonFiniteTopology(u8),
}
fn invalid_request_strategy() -> impl Strategy<Value = InvalidRequest> {
    prop_oneof![
        Just(InvalidRequest::ForeignHandle),
        any::<u8>().prop_map(InvalidRequest::InvalidRange),
        Just(InvalidRequest::Capacity),
        any::<u8>().prop_map(InvalidRequest::NonFiniteTopology),
    ]
}
#[derive(Debug, Clone, Copy, PartialEq)]
struct ModelRow {
    id: ParticleId,
    input: ParticleInput,
    pending: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModelPair {
    ids: [ParticleId; 2],
    flags: u32,
    strength: u32,
    distance: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModelTriad {
    ids: [ParticleId; 3],
    flags: u32,
    rest: [u32; 11],
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct TopologyState {
    pairs: Vec<ModelPair>,
    triads: Vec<ModelTriad>,
}
struct GroupModel {
    rows: Vec<ModelRow>,
    topology: TopologyState,
}
impl GroupModel {
    fn from_storage(storage: &ParticleStorage) -> Self {
        Self {
            rows: storage
                .dense_to_id
                .iter()
                .copied()
                .enumerate()
                .map(|(dense, id)| ModelRow {
                    id,
                    input: storage.input_at(ParticleIndex(dense)),
                    pending: false,
                })
                .collect(),
            topology: topology_state(storage),
        }
    }
    fn mapping_for_rotation(&self, selectors: [u8; 3]) -> Vec<Option<usize>> {
        let mut points = selectors.map(|value| usize::from(value) % (self.rows.len() + 1));
        points.sort_unstable();
        rotation_mapping(self.rows.len(), points[0], points[1], points[2])
    }

    fn apply_mapping(&mut self, old_to_new: &[Option<usize>]) {
        let new_count = old_to_new.iter().flatten().count();
        let mut remapped = vec![None; new_count];
        for (old, maybe_new) in old_to_new.iter().copied().enumerate() {
            if let Some(new) = maybe_new {
                remapped[new] = Some(self.rows[old]);
            }
        }
        self.rows = remapped
            .into_iter()
            .map(|maybe_row| maybe_row.expect("model mappings fill every destination"))
            .collect();
        let live_ids = self.rows.iter().map(|row| row.id).collect::<Vec<_>>();
        self.topology
            .pairs
            .retain(|pair| pair.ids.iter().all(|id| live_ids.contains(id)));
        self.topology
            .triads
            .retain(|triad| triad.ids.iter().all(|id| live_ids.contains(id)));
    }
    fn append_topology(&mut self, pairs: &[ParticlePair], triads: &[ParticleTriad]) {
        let appended_pairs = pairs
            .iter()
            .copied()
            .map(|pair| self.model_pair(pair))
            .collect::<Vec<_>>();
        let appended_triads = triads
            .iter()
            .copied()
            .map(|triad| self.model_triad(triad))
            .collect::<Vec<_>>();
        self.topology.pairs.extend(appended_pairs);
        self.topology.triads.extend(appended_triads);
        let row_ids = self.rows.iter().map(|row| row.id).collect::<Vec<_>>();
        self.topology
            .pairs
            .sort_by_key(|pair| dense_key(&row_ids, pair.ids));
        self.topology
            .triads
            .sort_by_key(|triad| dense_key(&row_ids, triad.ids));
        retain_first_pair_duplicate(&mut self.topology.pairs);
        retain_first_triad_duplicate(&mut self.topology.triads);
    }
    fn model_pair(&self, pair: ParticlePair) -> ModelPair {
        ModelPair {
            ids: pair.indices.map(|index| self.rows[index.0].id),
            flags: pair.flags.bits(),
            strength: pair.strength.to_bits(),
            distance: pair.distance.to_bits(),
        }
    }
    fn model_triad(&self, triad: ParticleTriad) -> ModelTriad {
        ModelTriad {
            ids: triad.indices.map(|index| self.rows[index.0].id),
            flags: triad.flags.bits(),
            rest: triad_rest_bits(triad),
        }
    }
}
fn fixture() -> (ParticleStorage, GroupModel) {
    let mut storage = ordinary_storage(ROW_COUNT);
    let group = ParticleGroupId::from_identity(Identity::new(storage.world, 20, 0));
    for value in 0..ROW_COUNT {
        let mut particle = input(
            i16::try_from(value + 1).expect("fixture value fits"),
            value % 2 == 0,
        );
        particle.maybe_group = Some(group);
        storage.create(particle).expect("fixture particle fits");
    }
    storage.pairs = vec![pair([0, 1], 10), pair([2, 3], 20)];
    storage.triads = vec![triad([0, 1, 2], 30), triad([3, 4, 5], 40)];
    storage
        .check_invariants()
        .expect("fixture satisfies storage invariants");
    let model = GroupModel::from_storage(&storage);
    (storage, model)
}
fn pair(indices: [usize; 2], rest: u8) -> ParticlePair {
    ParticlePair {
        indices: indices.map(ParticleIndex),
        flags: ParticleFlags::SPRING,
        strength: f32::from(rest) + 0.25,
        distance: f32::from(rest) + 0.5,
    }
}
fn triad(indices: [usize; 3], rest: u8) -> ParticleTriad {
    let rest = f32::from(rest);
    ParticleTriad {
        indices: indices.map(ParticleIndex),
        flags: ParticleFlags::ELASTIC,
        strength: rest + 0.25,
        pa: Vec2::new(rest + 1.0, rest + 2.0),
        pb: Vec2::new(rest + 3.0, rest + 4.0),
        pc: Vec2::new(rest + 5.0, rest + 6.0),
        ka: rest + 7.0,
        kb: rest + 8.0,
        kc: rest + 9.0,
        s: rest + 10.0,
    }
}
fn generated_topology(count: usize, selector: u8) -> (Vec<ParticlePair>, Vec<ParticleTriad>) {
    let pairs = (count >= 2)
        .then(|| {
            let first = usize::from(selector) % count;
            pair([first, (first + 1) % count], selector)
        })
        .into_iter()
        .collect();
    let triads = (count >= 3)
        .then(|| {
            let first = usize::from(selector) % count;
            triad([first, (first + 1) % count, (first + 2) % count], selector)
        })
        .into_iter()
        .collect();
    (pairs, triads)
}
fn rotation_mapping(count: usize, start: usize, middle: usize, end: usize) -> Vec<Option<usize>> {
    let mut mapping = (0..count).map(Some).collect::<Vec<_>>();
    for (old, destination) in mapping.iter_mut().enumerate().take(middle).skip(start) {
        *destination = Some(old + end - middle);
    }
    for (old, destination) in mapping.iter_mut().enumerate().take(end).skip(middle) {
        *destination = Some(old + start - middle);
    }
    mapping
}
fn apply_command(
    storage: &mut ParticleStorage,
    model: &mut GroupModel,
    command: Command,
) -> Result<(), TestCaseError> {
    match command {
        Command::CreateGroup(selector) => {
            let (pairs, triads) = generated_topology(model.rows.len(), selector);
            let candidate =
                MutationCandidate::prepare_create_group(storage, pairs.clone(), triads.clone())
                    .map_err(candidate_error)?;
            prop_assert_eq!(candidate.kind(), MutationCandidateKind::CreateGroup);
            model.append_topology(&pairs, &triads);
            candidate.commit(storage);
        }
        Command::Rotate(selectors) => {
            let mapping = model.mapping_for_rotation(selectors);
            let points = rotation_points(model.rows.len(), selectors);
            let candidate = MutationCandidate::prepare_ordinary_rotation(
                storage, points[0], points[1], points[2],
            )
            .map_err(candidate_error)?;
            prop_assert_eq!(candidate.kind(), MutationCandidateKind::OrdinaryRotation);
            model.apply_mapping(&mapping);
            candidate.commit(storage);
        }
        Command::Join(selectors, selector) => {
            let mapping = model.mapping_for_rotation(selectors);
            let (pairs, triads) = generated_topology(model.rows.len(), selector);
            let candidate = MutationCandidate::prepare_join_groups(
                storage,
                &mapping,
                pairs.clone(),
                triads.clone(),
            )
            .map_err(candidate_error)?;
            prop_assert_eq!(candidate.kind(), MutationCandidateKind::JoinGroups);
            model.apply_mapping(&mapping);
            model.append_topology(&pairs, &triads);
            candidate.commit(storage);
        }
        Command::Split(selectors) => {
            let mapping = model.mapping_for_rotation(selectors);
            let candidate = MutationCandidate::prepare_split_group(storage, &mapping)
                .map_err(candidate_error)?;
            prop_assert_eq!(candidate.kind(), MutationCandidateKind::SplitGroup);
            model.apply_mapping(&mapping);
            candidate.commit(storage);
        }
        Command::MarkZombie(selector) => {
            let Some(row) = selected_row_mut(&mut model.rows, selector) else {
                return Ok(());
            };
            let result = storage.mark_delete(row.id);
            if row.pending {
                prop_assert_eq!(result, Err(ParticleStorageError::PendingDelete));
            } else {
                result.map_err(candidate_error)?;
                row.input.flags.insert(ParticleFlags::ZOMBIE);
                row.pending = true;
            }
        }
        Command::Compact => {
            let mapping = compaction_mapping(&model.rows);
            let candidate = MutationCandidate::prepare_zombie_compaction(storage, &mapping)
                .map_err(candidate_error)?;
            prop_assert_eq!(candidate.kind(), MutationCandidateKind::ZombieCompaction);
            model.apply_mapping(&mapping);
            candidate.commit(storage);
        }
        Command::Reactive(selector) => {
            let (pairs, triads) = generated_topology(model.rows.len(), selector);
            let candidate = MutationCandidate::prepare_reactive_regeneration(
                storage,
                pairs.clone(),
                triads.clone(),
            )
            .map_err(candidate_error)?;
            prop_assert_eq!(
                candidate.kind(),
                MutationCandidateKind::ReactiveRegeneration
            );
            model.append_topology(&pairs, &triads);
            candidate.commit(storage);
        }
        Command::FlagChange => {
            let candidate =
                MutationCandidate::prepare_group_flag_change(storage).map_err(candidate_error)?;
            prop_assert_eq!(candidate.kind(), MutationCandidateKind::GroupFlagChange);
            candidate.commit(storage);
        }
    }
    Ok(())
}

fn rotation_points(count: usize, selectors: [u8; 3]) -> [usize; 3] {
    let mut points = selectors.map(|value| usize::from(value) % (count + 1));
    points.sort_unstable();
    points
}

fn selected_row_mut(rows: &mut [ModelRow], selector: u8) -> Option<&mut ModelRow> {
    let index = (!rows.is_empty()).then(|| usize::from(selector) % rows.len())?;
    rows.get_mut(index)
}

fn compaction_mapping(rows: &[ModelRow]) -> Vec<Option<usize>> {
    let mut next = 0;
    rows.iter()
        .map(|row| {
            if row.pending {
                return None;
            }
            let destination = next;
            next += 1;
            Some(destination)
        })
        .collect()
}

fn candidate_error(error: ParticleStorageError) -> TestCaseError {
    TestCaseError::fail(format!("candidate unexpectedly failed: {error:?}"))
}

fn assert_model(storage: &ParticleStorage, model: &GroupModel) -> Result<(), TestCaseError> {
    prop_assert_eq!(storage.check_invariants(), Ok(()));
    let expected_ids = model.rows.iter().map(|row| row.id).collect::<Vec<_>>();
    prop_assert_eq!(storage.dense_to_id.as_slice(), expected_ids.as_slice());
    prop_assert_eq!(
        storage
            .dense_to_id
            .iter()
            .enumerate()
            .map(|(dense, _id)| storage.input_at(ParticleIndex(dense)))
            .collect::<Vec<_>>(),
        model.rows.iter().map(|row| row.input).collect::<Vec<_>>()
    );
    prop_assert_eq!(topology_state(storage), model.topology.clone());
    assert_aligned_lanes(storage)?;
    assert_contiguous_group_ranges(storage, model)?;
    Ok(())
}

fn assert_aligned_lanes(storage: &ParticleStorage) -> Result<(), TestCaseError> {
    let count = storage.dense_to_id.len();
    for length in [
        storage.positions.len(),
        storage.velocities.len(),
        storage.flags.len(),
        storage.groups.len(),
        storage.weights.len(),
        storage.forces.len(),
    ] {
        prop_assert_eq!(length, count);
    }
    if let Some(colors) = &storage.maybe_colors {
        prop_assert_eq!(colors.len(), count);
    }
    if let Some(associations) = &storage.maybe_user_associations {
        prop_assert_eq!(associations.len(), count);
    }
    if let Some(expirations) = &storage.maybe_expiration_times {
        prop_assert_eq!(expirations.len(), count);
    }
    Ok(())
}

fn assert_contiguous_group_ranges(
    storage: &ParticleStorage,
    model: &GroupModel,
) -> Result<(), TestCaseError> {
    let expected_groups = model
        .rows
        .iter()
        .map(|row| row.input.maybe_group)
        .collect::<Vec<_>>();
    prop_assert_eq!(storage.groups.as_slice(), expected_groups.as_slice());
    let mut expected_ranges = Vec::new();
    for (dense, maybe_group) in expected_groups.iter().copied().enumerate() {
        let Some(group) = maybe_group else {
            continue;
        };
        if let Some((last_group, _first, last)) = expected_ranges.last_mut()
            && *last_group == group
            && *last == dense
        {
            *last = dense + 1;
        } else {
            expected_ranges.push((group, dense, dense + 1));
        }
    }
    let actual_ranges = storage
        .group_records
        .iter()
        .filter(|record| record.first != record.last)
        .map(|record| (record.id, record.first, record.last))
        .collect::<Vec<_>>();
    prop_assert_eq!(actual_ranges, expected_ranges);
    Ok(())
}

fn topology_state(storage: &ParticleStorage) -> TopologyState {
    TopologyState {
        pairs: storage
            .pairs
            .iter()
            .copied()
            .map(|pair| ModelPair {
                ids: pair.indices.map(|index| storage.dense_to_id[index.0]),
                flags: pair.flags.bits(),
                strength: pair.strength.to_bits(),
                distance: pair.distance.to_bits(),
            })
            .collect(),
        triads: storage
            .triads
            .iter()
            .copied()
            .map(|triad| ModelTriad {
                ids: triad.indices.map(|index| storage.dense_to_id[index.0]),
                flags: triad.flags.bits(),
                rest: triad_rest_bits(triad),
            })
            .collect(),
    }
}

fn triad_rest_bits(triad: ParticleTriad) -> [u32; 11] {
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
}

fn dense_key<const N: usize>(rows: &[ParticleId], ids: [ParticleId; N]) -> [usize; N] {
    ids.map(|id| {
        rows.iter()
            .position(|candidate| *candidate == id)
            .expect("topology identity remains live")
    })
}

fn retain_first_pair_duplicate(pairs: &mut Vec<ModelPair>) {
    let mut retained = Vec::with_capacity(pairs.len());
    for pair in pairs.iter().copied() {
        if retained
            .last()
            .is_none_or(|previous: &ModelPair| previous.ids != pair.ids)
        {
            retained.push(pair);
        }
    }
    *pairs = retained;
}

fn retain_first_triad_duplicate(triads: &mut Vec<ModelTriad>) {
    let mut retained = Vec::with_capacity(triads.len());
    for triad in triads.iter().copied() {
        if retained
            .last()
            .is_none_or(|previous: &ModelTriad| previous.ids != triad.ids)
        {
            retained.push(triad);
        }
    }
    *triads = retained;
}

fn property_config(seed: u64) -> ProptestConfig {
    ProptestConfig {
        cases: 128,
        rng_seed: RngSeed::Fixed(seed),
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(property_config(MODEL_SEED))]

    #[test]
    fn candidate_sequences_match_the_independent_semantic_model(
        generated in prop::collection::vec(command_strategy(), 0..40),
    ) {
        // Arrange
        let mut commands = vec![
            Command::CreateGroup(1),
            Command::Rotate([0, 2, 6]),
            Command::Join([0, 3, 6], 2),
            Command::Split([0, 1, 6]),
            Command::MarkZombie(2),
            Command::Compact,
            Command::Reactive(3),
            Command::FlagChange,
        ];
        commands.extend(generated);
        let (mut storage, mut model) = fixture();

        for (step, command) in commands.iter().copied().enumerate() {
            // Act
            apply_command(&mut storage, &mut model, command)?;

            // Assert
            assert_model(&storage, &model)
                .map_err(|error| TestCaseError::fail(
                    format!("{error}; command_prefix={:?}", &commands[..=step])
                ))?;
        }
    }
}

proptest! {
    #![proptest_config(property_config(ROLLBACK_SEED))]

    #[test]
    fn rejected_requests_leave_the_complete_storage_unchanged(
        request in invalid_request_strategy(),
    ) {
        // Arrange
        let (mut storage, _model) = fixture();
        let before = storage.clone();

        // Act
        let result = apply_invalid_request(&mut storage, request);

        // Assert
        prop_assert!(result.is_err());
        prop_assert!(storage == before);
    }
}

fn apply_invalid_request(
    storage: &mut ParticleStorage,
    request: InvalidRequest,
) -> Result<(), ParticleStorageError> {
    match request {
        InvalidRequest::ForeignHandle => {
            let foreign_world =
                WorldKey::fresh().map_err(|_error| ParticleStorageError::IdentityExhausted)?;
            let foreign_system =
                ParticleSystemId::from_identity(Identity::new(foreign_world, 0, 0));
            let foreign = ParticleId::from_identity(Identity::new_particle(
                foreign_world,
                0,
                0,
                foreign_system.identity(),
            ));
            storage.mark_delete(foreign).map(|_snapshot| ())
        }
        InvalidRequest::InvalidRange(selector) => {
            let invalid_middle = storage.len() + usize::from(selector) + 1;
            MutationCandidate::prepare_ordinary_rotation(storage, 0, invalid_middle, storage.len())
                .map(|_candidate| ())
        }
        InvalidRequest::Capacity => storage.create(input(i16::MAX, true)).map(|_id| ()),
        InvalidRequest::NonFiniteTopology(selector) => {
            let (mut pairs, _triads) = generated_topology(storage.len(), selector);
            let Some(pair) = pairs.first_mut() else {
                return Err(ParticleStorageError::InvalidLaneBundle);
            };
            pair.distance = f32::NAN;
            MutationCandidate::prepare_reactive_regeneration(storage, pairs, Vec::new())
                .map(|_candidate| ())
        }
    }
}

#[test]
fn prepared_candidate_abandoned_by_a_panic_leaves_storage_unchanged() {
    // Arrange
    let (storage, _model) = fixture();
    let before = storage.clone();

    // Act
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _candidate = MutationCandidate::prepare_group_flag_change(&storage)
            .expect("candidate preparation succeeds");
        panic!("injected panic before commit");
    }));

    // Assert
    assert!(panic.is_err());
    assert!(storage == before);
}
