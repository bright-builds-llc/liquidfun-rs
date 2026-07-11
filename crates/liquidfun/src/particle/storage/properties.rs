use proptest::prelude::*;

use super::*;

const DECLARED_CAPACITY: usize = 6;

#[derive(Debug, Clone)]
enum Operation {
    Create { value: i16, optional: bool },
    Rotate(u8),
    MarkDelete(u8),
    Compact,
    StaleAccess(u8),
    Mutate(u8, i16),
    CapacityProbe,
}

fn operation_strategy() -> impl Strategy<Value = Operation> {
    prop_oneof![
        (any::<i16>(), any::<bool>())
            .prop_map(|(value, optional)| Operation::Create { value, optional }),
        any::<u8>().prop_map(Operation::Rotate),
        any::<u8>().prop_map(Operation::MarkDelete),
        Just(Operation::Compact),
        any::<u8>().prop_map(Operation::StaleAccess),
        (any::<u8>(), any::<i16>())
            .prop_map(|(selector, value)| Operation::Mutate(selector, value)),
        Just(Operation::CapacityProbe),
    ]
}

fn input(value: i16, optional: bool) -> ParticleInput {
    let value = i32::from(value);
    ParticleInput {
        position: [value, value.saturating_neg()],
        velocity: [value.saturating_add(1), value.saturating_sub(1)],
        flags: value.unsigned_abs(),
        group: 0,
        maybe_color: optional.then_some(value.to_le_bytes()),
        maybe_lifetime: optional.then_some(value.unsigned_abs()),
    }
}

fn storage_with_capacity(declared_capacity: usize, allocation_capacity: usize) -> ParticleStorage {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    ParticleStorage::from_owned_lanes(
        world,
        system,
        0,
        declared_capacity,
        declared_capacity,
        OwnedLaneBundle::with_capacity(allocation_capacity, true),
    )
    .expect("test lane bundle is valid")
}

fn ordinary_storage(declared_capacity: usize) -> ParticleStorage {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    ParticleStorage::new(world, system, 0, declared_capacity, declared_capacity)
        .expect("ordinary test storage is valid")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModelRow {
    id: ParticleId,
    input: ParticleInput,
    pending: bool,
}

struct Model {
    rows: Vec<ModelRow>,
    known_ids: Vec<ParticleId>,
    optional_lanes: bool,
    capacity: usize,
}

impl Model {
    fn new(capacity: usize) -> Self {
        Self {
            rows: Vec::new(),
            known_ids: Vec::new(),
            optional_lanes: false,
            capacity,
        }
    }

    fn normalize_for_create(&mut self, mut input: ParticleInput) -> ParticleInput {
        if !self.optional_lanes && input.maybe_color.is_some() {
            for row in &mut self.rows {
                row.input.maybe_color = Some([0; 4]);
                row.input.maybe_lifetime = Some(0);
            }
            self.optional_lanes = true;
        }
        if self.optional_lanes {
            input.maybe_color.get_or_insert([0; 4]);
            input.maybe_lifetime.get_or_insert(0);
        }
        input
    }

    fn selected_id(&self, selector: u8) -> Option<ParticleId> {
        if self.known_ids.is_empty() {
            return None;
        }
        Some(self.known_ids[usize::from(selector) % self.known_ids.len()])
    }

    fn maybe_row_mut(&mut self, id: ParticleId) -> Option<&mut ModelRow> {
        self.rows.iter_mut().find(|row| row.id == id)
    }
}

fn semantic_rows(storage: &ParticleStorage) -> Vec<ModelRow> {
    storage
        .dense_to_id
        .iter()
        .copied()
        .enumerate()
        .map(|(dense, id)| {
            let local_slot = storage
                .local_slot(id)
                .expect("authoritative dense identities are locally scoped");
            let pending = matches!(
                storage.identities[local_slot].state,
                IdentityState::PendingDelete { .. }
            );
            ModelRow {
                id,
                input: storage.input_at(ParticleIndex(dense)),
                pending,
            }
        })
        .collect()
}

fn apply_create(
    storage: &mut ParticleStorage,
    model: &mut Model,
    raw_input: ParticleInput,
) -> Result<(), TestCaseError> {
    let actual = storage.create(raw_input);
    if model.rows.len() >= model.capacity {
        prop_assert_eq!(
            actual,
            Err(ParticleStorageError::CapacityExceeded {
                limit: model.capacity
            })
        );
        return Ok(());
    }

    let id = actual
        .map_err(|error| TestCaseError::fail(format!("unexpected create error: {error:?}")))?;
    let normalized = model.normalize_for_create(raw_input);
    model.rows.push(ModelRow {
        id,
        input: normalized,
        pending: false,
    });
    model.known_ids.push(id);
    Ok(())
}

fn apply_operation(
    storage: &mut ParticleStorage,
    model: &mut Model,
    operation: &Operation,
) -> Result<(), TestCaseError> {
    match *operation {
        Operation::Create { value, optional } => {
            apply_create(storage, model, input(value, optional))?;
        }
        Operation::CapacityProbe => {
            apply_create(storage, model, input(i16::MAX, true))?;
        }
        Operation::Rotate(selector) => {
            let count = model.rows.len();
            let pivot = usize::from(selector) % count.saturating_add(1);
            storage
                .rotate_rows(0, pivot, count)
                .map_err(|error| TestCaseError::fail(format!("rotation failed: {error:?}")))?;
            model.rows.rotate_left(pivot);
        }
        Operation::MarkDelete(selector) => {
            let Some(id) = model.selected_id(selector) else {
                return Ok(());
            };
            let actual = storage.mark_delete(id);
            let Some(row) = model.maybe_row_mut(id) else {
                prop_assert_eq!(actual, Err(ParticleStorageError::StaleOrDestroyed));
                return Ok(());
            };
            if row.pending {
                prop_assert_eq!(actual, Err(ParticleStorageError::PendingDelete));
            } else {
                prop_assert_eq!(
                    actual,
                    Ok(ParticleSnapshot {
                        id,
                        input: row.input
                    })
                );
                row.pending = true;
            }
        }
        Operation::Compact => {
            let expected: Vec<_> = model
                .rows
                .iter()
                .filter(|row| row.pending)
                .map(|row| ParticleSnapshot {
                    id: row.id,
                    input: row.input,
                })
                .collect();
            let actual = storage
                .compact_pending()
                .map_err(|error| TestCaseError::fail(format!("compaction failed: {error:?}")))?;
            prop_assert_eq!(actual, expected);
            model.rows.retain(|row| !row.pending);
        }
        Operation::StaleAccess(selector) => {
            let Some(id) = model.selected_id(selector) else {
                return Ok(());
            };
            let expected = match model.rows.iter().find(|row| row.id == id) {
                Some(row) if row.pending => Err(ParticleStorageError::PendingDelete),
                Some(row) => Ok(row.input),
                None => Err(ParticleStorageError::StaleOrDestroyed),
            };
            prop_assert_eq!(storage.input(id), expected);
        }
        Operation::Mutate(selector, value) => {
            let Some(id) = model.selected_id(selector) else {
                return Ok(());
            };
            let position = [i32::from(value), i32::from(value).saturating_neg()];
            let actual = storage.set_position(id, position);
            let expected = match model.maybe_row_mut(id) {
                Some(row) if row.pending => Err(ParticleStorageError::PendingDelete),
                Some(row) => {
                    row.input.position = position;
                    Ok(())
                }
                None => Err(ParticleStorageError::StaleOrDestroyed),
            };
            prop_assert_eq!(actual, expected);
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn bounded_state_machine_matches_independent_model(
        generated in prop::collection::vec(operation_strategy(), 1..64),
    ) {
        // Arrange
        let mut operations = vec![
            Operation::Create { value: 1, optional: false },
            Operation::Create { value: 2, optional: true },
            Operation::Rotate(1),
            Operation::MarkDelete(0),
            Operation::Compact,
            Operation::StaleAccess(0),
            Operation::CapacityProbe,
            Operation::Mutate(1, 3),
        ];
        operations.extend(generated);
        let mut storage = ordinary_storage(DECLARED_CAPACITY);
        let mut model = Model::new(DECLARED_CAPACITY);

        for (step, operation) in operations.iter().enumerate() {
            // Act
            apply_operation(&mut storage, &mut model, operation)?;

            // Assert
            prop_assert_eq!(
                storage.check_invariants(),
                Ok(()),
                "operation_prefix={:?}",
                &operations[..=step]
            );
            prop_assert_eq!(
                semantic_rows(&storage),
                model.rows.clone(),
                "operation_prefix={:?}",
                &operations[..=step]
            );
        }
    }
}

#[test]
fn declared_capacity_controls_growth_and_teardown_returns_owned_buffers() {
    // Arrange
    let mut storage = storage_with_capacity(1, 8);
    storage.create(input(1, true)).expect("declared row fits");

    // Act
    let overflow = storage.create(input(2, true));
    let visible_positions = storage.positions().to_vec();
    let lanes = storage.into_owned_lanes();

    // Assert
    assert_eq!(
        overflow,
        Err(ParticleStorageError::CapacityExceeded { limit: 1 })
    );
    assert_eq!(visible_positions, vec![[1, -1]]);
    assert_eq!(lanes.positions, visible_positions);
    assert_eq!(lanes.velocities, vec![[2, 0]]);
    assert_eq!(lanes.flags, vec![1]);
    assert_eq!(lanes.groups, vec![0]);
    assert_eq!(lanes.maybe_colors, Some(vec![[1, 0, 0, 0]]));
    assert_eq!(lanes.maybe_lifetimes, Some(vec![1]));
}

#[test]
fn undersized_owned_lane_bundle_is_rejected() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    let lanes = OwnedLaneBundle::with_capacity(1, true);

    // Act
    let result = ParticleStorage::from_owned_lanes(world, system, 0, 2, 2, lanes);

    // Assert
    assert!(matches!(
        result,
        Err(ParticleStorageError::InvalidLaneBundle)
    ));
}
