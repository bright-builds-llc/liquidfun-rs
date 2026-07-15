use proptest::prelude::*;

use super::*;

#[derive(Debug, Clone)]
enum LifecycleOperation {
    Create {
        lifetime: i8,
        optional: bool,
        listener: bool,
    },
    Tick(u8),
    SetLifetime(u8, i8),
    Mark(u8, bool),
    DestroyOldest,
    Compact,
    PauseMarker,
}

fn lifecycle_operation_strategy() -> impl Strategy<Value = LifecycleOperation> {
    prop_oneof![
        (any::<i8>(), any::<bool>(), any::<bool>()).prop_map(|(lifetime, optional, listener)| {
            LifecycleOperation::Create {
                lifetime,
                optional,
                listener,
            }
        },),
        any::<u8>().prop_map(LifecycleOperation::Tick),
        (any::<u8>(), any::<i8>())
            .prop_map(|(selector, lifetime)| LifecycleOperation::SetLifetime(selector, lifetime)),
        (any::<u8>(), any::<bool>())
            .prop_map(|(selector, listener)| LifecycleOperation::Mark(selector, listener)),
        Just(LifecycleOperation::DestroyOldest),
        Just(LifecycleOperation::Compact),
        Just(LifecycleOperation::PauseMarker),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LifecycleRow {
    id: ParticleId,
    expiration: i32,
    pending: bool,
    listener: bool,
}

struct LifecycleModel {
    rows: Vec<LifecycleRow>,
    order: Vec<ParticleId>,
    elapsed: i32,
    tracking: bool,
    dirty: bool,
}

impl LifecycleModel {
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            order: Vec::new(),
            elapsed: 0,
            tracking: true,
            dirty: false,
        }
    }

    fn selected_index(&self, selector: u8) -> Option<usize> {
        (!self.rows.is_empty()).then(|| usize::from(selector) % self.rows.len())
    }

    fn expiration_for_creation(&mut self, lifetime: i32) -> i32 {
        if lifetime > 0 {
            self.tracking = true;
            return self.elapsed + lifetime;
        }
        if self.tracking { -self.elapsed } else { 0 }
    }

    fn sort_if_dirty(&mut self) {
        if !self.dirty {
            return;
        }
        self.order.sort_by(|left, right| {
            let left = self
                .rows
                .iter()
                .find(|row| row.id == *left)
                .expect("ordered identity remains present");
            let right = self
                .rows
                .iter()
                .find(|row| row.id == *right)
                .expect("ordered identity remains present");
            let left_infinite = left.expiration <= 0;
            let right_infinite = right.expiration <= 0;
            if left_infinite == right_infinite {
                return right.expiration.cmp(&left.expiration);
            }
            if left_infinite {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });
        self.dirty = false;
    }

    fn oldest_index(&mut self) -> Option<usize> {
        self.sort_if_dirty();
        let maybe_id = self
            .order
            .iter()
            .rev()
            .find(|id| {
                self.rows
                    .iter()
                    .find(|row| row.id == **id)
                    .is_some_and(|row| row.expiration > 0)
            })
            .or_else(|| {
                self.order.iter().find(|id| {
                    self.rows
                        .iter()
                        .find(|row| row.id == **id)
                        .is_some_and(|row| row.expiration <= 0)
                })
            })
            .copied();
        maybe_id.and_then(|id| self.rows.iter().position(|row| row.id == id))
    }

    fn compact(&mut self) -> (Vec<ParticleId>, Vec<ParticleId>) {
        let destroyed = self
            .rows
            .iter()
            .filter(|row| row.pending)
            .map(|row| row.id)
            .collect::<Vec<_>>();
        let occurrences = self
            .rows
            .iter()
            .filter(|row| row.pending && row.listener)
            .map(|row| row.id)
            .collect::<Vec<_>>();
        self.rows.retain(|row| !row.pending);
        self.order
            .retain(|id| self.rows.iter().any(|row| row.id == *id));
        (destroyed, occurrences)
    }
}

fn bounded_lifetime(raw: i8) -> i8 {
    raw.rem_euclid(8) - 2
}

fn lifecycle_input(optional: bool, listener: bool) -> ParticleInput {
    let mut input = input(7, optional);
    input.maybe_expiration_time = None;
    input
        .flags
        .set(ParticleFlags::DESTRUCTION_LISTENER, listener);
    input
}

fn lifecycle_definition() -> ParticleSystemDef {
    ParticleSystemDef::default()
        .with_lifetime_granularity(1.0)
        .expect("property granularity is valid")
        .with_capacity(ParticleCapacity::fixed(DECLARED_CAPACITY).expect("capacity is positive"))
        .expect("capacity and maximum agree")
        .with_maximum_count(DECLARED_CAPACITY)
        .expect("maximum fits capacity")
}

#[allow(
    clippy::too_many_lines,
    reason = "one focused property interpreter keeps every model and storage transition adjacent"
)]
fn apply_lifecycle_operation(
    storage: &mut ParticleStorage,
    state: &mut ParticleLifetimeState,
    model: &mut LifecycleModel,
    operation: &LifecycleOperation,
) -> Result<(), TestCaseError> {
    match *operation {
        LifecycleOperation::Create {
            lifetime,
            optional,
            listener,
        } => {
            if model.rows.len() == DECLARED_CAPACITY {
                let oldest = model.oldest_index().expect("full model has an oldest row");
                model.rows[oldest].pending = true;
                let expected = model.compact();
                let actual = state
                    .prepare_capacity_for_creation(storage)
                    .map_err(|error| TestCaseError::fail(format!("capacity failed: {error:?}")))?
                    .expect("full storage compacts");
                prop_assert_eq!(
                    actual
                        .destroyed
                        .iter()
                        .map(|snapshot| snapshot.id)
                        .collect::<Vec<_>>(),
                    expected.0
                );
                prop_assert_eq!(
                    actual
                        .requested_listener_occurrences
                        .iter()
                        .map(|occurrence| occurrence.particle())
                        .collect::<Vec<_>>(),
                    expected.1
                );
            }
            let lifetime = bounded_lifetime(lifetime);
            let id = storage
                .create(lifecycle_input(optional, listener))
                .map_err(|error| TestCaseError::fail(format!("create failed: {error:?}")))?;
            state
                .initialize_created_particle(storage, id, f32::from(lifetime))
                .map_err(|error| TestCaseError::fail(format!("lifetime init failed: {error:?}")))?;
            let expiration = model.expiration_for_creation(i32::from(lifetime));
            model.rows.push(LifecycleRow {
                id,
                expiration,
                pending: false,
                listener,
            });
            model.order.push(id);
            model.dirty = true;
        }
        LifecycleOperation::Tick(raw) => {
            if !model.tracking {
                return Ok(());
            }
            let timestep = raw % 4;
            state
                .solve_lifetimes(storage, f32::from(timestep))
                .map_err(|error| TestCaseError::fail(format!("tick failed: {error:?}")))?;
            model.elapsed += i32::from(timestep);
            model.sort_if_dirty();
            for row in &mut model.rows {
                if !row.pending && row.expiration > 0 && row.expiration <= model.elapsed {
                    row.pending = true;
                }
            }
        }
        LifecycleOperation::SetLifetime(selector, raw) => {
            let Some(index) = model.selected_index(selector) else {
                return Ok(());
            };
            if model.rows[index].pending {
                return Ok(());
            }
            let lifetime = bounded_lifetime(raw);
            state
                .set_particle_lifetime(storage, model.rows[index].id, f32::from(lifetime))
                .map_err(|error| TestCaseError::fail(format!("set lifetime failed: {error:?}")))?;
            model.tracking = true;
            model.rows[index].expiration = if lifetime > 0 {
                model.elapsed + i32::from(lifetime)
            } else {
                i32::from(lifetime)
            };
            model.dirty = true;
        }
        LifecycleOperation::Mark(selector, listener) => {
            let Some(index) = model.selected_index(selector) else {
                return Ok(());
            };
            if model.rows[index].pending {
                return Ok(());
            }
            storage
                .mark_delete_for_lifecycle(model.rows[index].id, listener)
                .map_err(|error| TestCaseError::fail(format!("mark failed: {error:?}")))?;
            model.rows[index].pending = true;
            model.rows[index].listener |= listener;
        }
        LifecycleOperation::DestroyOldest => {
            let Some(index) = model.oldest_index() else {
                return Ok(());
            };
            state
                .destroy_oldest_particle(storage, 0, false)
                .map_err(|error| TestCaseError::fail(format!("oldest failed: {error:?}")))?;
            model.rows[index].pending = true;
        }
        LifecycleOperation::Compact => {
            let expected = model.compact();
            let actual = compact_pending_with_occurrences(storage)
                .map_err(|error| TestCaseError::fail(format!("compact failed: {error:?}")))?;
            prop_assert_eq!(
                actual
                    .destroyed
                    .iter()
                    .map(|snapshot| snapshot.id)
                    .collect::<Vec<_>>(),
                expected.0
            );
            prop_assert_eq!(
                actual
                    .requested_listener_occurrences
                    .iter()
                    .map(|occurrence| occurrence.particle())
                    .collect::<Vec<_>>(),
                expected.1
            );
        }
        LifecycleOperation::PauseMarker => {}
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn lifecycle_state_machine_matches_independent_stable_identity_model(
        operations in prop::collection::vec(lifecycle_operation_strategy(), 1..48),
    ) {
        // Arrange
        let definition = lifecycle_definition();
        let mut storage = ordinary_storage(DECLARED_CAPACITY);
        let mut state = ParticleLifetimeState::new(definition, &mut storage);
        let mut model = LifecycleModel::new();

        for (step, operation) in operations.iter().enumerate() {
            // Act
            apply_lifecycle_operation(&mut storage, &mut state, &mut model, operation)?;

            // Assert
            prop_assert_eq!(storage.check_invariants(), Ok(()));
            prop_assert_eq!(
                storage.particle_ids(),
                model.rows.iter().map(|row| row.id).collect::<Vec<_>>(),
                "operation_prefix={:?}",
                &operations[..=step]
            );
            for row in &model.rows {
                prop_assert_eq!(storage.is_pending(row.id), Ok(row.pending));
            }
        }
    }
}
