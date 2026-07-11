#![allow(
    dead_code,
    reason = "the private arena foundation is consumed by later world storage plans"
)]

use std::marker::PhantomData;

use crate::error::{ArenaInsertError, HandleError};
use crate::identity::{ErasedHandle, HandleIdentity, Identity, WorldKey};

enum Slot<T> {
    Occupied { generation: u64, value: T },
    Vacant { generation: u64 },
    Retired,
}

pub(crate) struct Arena<T, H> {
    world: WorldKey,
    slots: Vec<Slot<T>>,
    free_slots: Vec<usize>,
    retired_slots: usize,
    max_slots: usize,
    handle: PhantomData<fn() -> H>,
}

impl<T, H: HandleIdentity> Arena<T, H> {
    pub(crate) fn new(world: WorldKey, max_slots: usize) -> Self {
        Self {
            world,
            slots: Vec::new(),
            free_slots: Vec::new(),
            retired_slots: 0,
            max_slots,
            handle: PhantomData,
        }
    }

    pub(crate) fn insert(&mut self, value: T) -> Result<H, ArenaInsertError> {
        let Some(slot_index) = self.free_slots.pop() else {
            return self.insert_new_slot(value);
        };

        let slot = self
            .slots
            .get_mut(slot_index)
            .expect("a free-slot index always refers to an existing slot");
        let generation = match slot {
            Slot::Vacant { generation } => *generation,
            Slot::Occupied { .. } | Slot::Retired => {
                unreachable!("only vacant slots are placed on the free list")
            }
        };
        *slot = Slot::Occupied { generation, value };

        Ok(H::from_identity(Identity::new(
            self.world, slot_index, generation,
        )))
    }

    fn insert_new_slot(&mut self, value: T) -> Result<H, ArenaInsertError> {
        if self.slots.len() >= self.max_slots {
            return Err(self.exhaustion_error());
        }

        let slot_index = self.slots.len();
        let generation = 0;
        self.slots.push(Slot::Occupied { generation, value });

        Ok(H::from_identity(Identity::new(
            self.world, slot_index, generation,
        )))
    }

    fn exhaustion_error(&self) -> ArenaInsertError {
        if self.retired_slots > 0 {
            return ArenaInsertError::GenerationExhausted;
        }

        ArenaInsertError::CapacityExceeded {
            limit: self.max_slots,
        }
    }

    pub(crate) fn get(&self, handle: H) -> Result<&T, HandleError> {
        self.get_erased(handle.erased())
    }

    pub(crate) fn get_erased(&self, handle: ErasedHandle) -> Result<&T, HandleError> {
        if handle.kind() != H::KIND {
            return Err(HandleError::WrongKind {
                expected: H::KIND,
                actual: handle.kind(),
            });
        }

        let identity = handle.identity();
        if identity.world() != self.world {
            return Err(HandleError::WrongWorld);
        }

        let Some(slot) = self.slots.get(identity.slot()) else {
            return Err(HandleError::StaleOrDestroyed);
        };
        let Slot::Occupied { generation, value } = slot else {
            return Err(HandleError::StaleOrDestroyed);
        };
        if *generation != identity.generation() {
            return Err(HandleError::StaleOrDestroyed);
        }

        Ok(value)
    }

    pub(crate) fn remove(&mut self, handle: H) -> Result<T, HandleError> {
        self.validate_typed_handle(handle)?;
        let identity = handle.identity();
        let slot = &mut self.slots[identity.slot()];
        let previous = std::mem::replace(slot, Slot::Retired);
        let Slot::Occupied { generation, value } = previous else {
            unreachable!("validated handles always refer to occupied slots")
        };

        let Some(next_generation) = generation.checked_add(1) else {
            self.retired_slots += 1;
            return Ok(value);
        };

        *slot = Slot::Vacant {
            generation: next_generation,
        };
        self.free_slots.push(identity.slot());
        Ok(value)
    }

    fn validate_typed_handle(&self, handle: H) -> Result<(), HandleError> {
        self.get(handle).map(|_value| ())
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (H, &T)> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(slot_index, slot)| match slot {
                Slot::Occupied { generation, value } => Some((
                    H::from_identity(Identity::new(self.world, slot_index, *generation)),
                    value,
                )),
                Slot::Vacant { .. } | Slot::Retired => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{BodyId, FixtureId};

    fn test_world() -> WorldKey {
        WorldKey::fresh().expect("test world key should remain available")
    }

    #[test]
    fn removed_handle_never_resolves_after_slot_reuse() {
        // Arrange
        let mut arena = Arena::<_, BodyId>::new(test_world(), 1);
        let stale = arena.insert("first").expect("first insert should fit");

        // Act
        let removed = arena.remove(stale).expect("live handle should remove");
        let replacement = arena
            .insert("second")
            .expect("vacant slot should be reused");

        // Assert
        assert_eq!(removed, "first");
        assert_ne!(stale, replacement);
        assert_eq!(arena.get(stale), Err(HandleError::StaleOrDestroyed));
        assert_eq!(arena.get(replacement), Ok(&"second"));
    }

    #[test]
    fn world_is_validated_before_slot_and_generation() {
        // Arrange
        let mut first = Arena::<_, BodyId>::new(test_world(), 1);
        let second = Arena::<i32, BodyId>::new(test_world(), 0);
        let handle = first.insert(7).expect("first insert should fit");

        // Act
        let result = second.get(handle);

        // Assert
        assert_eq!(result, Err(HandleError::WrongWorld));
    }

    #[test]
    fn erased_internal_lookup_rejects_wrong_kind() {
        // Arrange
        let mut arena = Arena::<_, BodyId>::new(test_world(), 1);
        let body = arena.insert(7).expect("first insert should fit");
        let fixture = FixtureId::from_identity(body.identity());

        // Act
        let result = arena.get_erased(fixture.erased());

        // Assert
        assert_eq!(
            result,
            Err(HandleError::WrongKind {
                expected: crate::ObjectKind::Body,
                actual: crate::ObjectKind::Fixture,
            })
        );
    }

    #[test]
    fn capacity_failure_preserves_existing_state() {
        // Arrange
        let mut arena = Arena::<_, BodyId>::new(test_world(), 1);
        let existing = arena.insert(7).expect("first insert should fit");

        // Act
        let result = arena.insert(9);

        // Assert
        assert_eq!(result, Err(ArenaInsertError::CapacityExceeded { limit: 1 }));
        assert_eq!(arena.get(existing), Ok(&7));
    }

    #[test]
    fn maximum_generation_retires_permanently() {
        // Arrange
        let world = test_world();
        let exhausted = BodyId::from_identity(Identity::new(world, 0, u64::MAX));
        let mut arena = Arena::<_, BodyId> {
            world,
            slots: vec![Slot::Occupied {
                generation: u64::MAX,
                value: 7,
            }],
            free_slots: Vec::new(),
            retired_slots: 0,
            max_slots: 1,
            handle: PhantomData,
        };

        // Act
        let removed = arena.remove(exhausted);
        let replacement = arena.insert(9);

        // Assert
        assert_eq!(removed, Ok(7));
        assert_eq!(arena.get(exhausted), Err(HandleError::StaleOrDestroyed));
        assert_eq!(replacement, Err(ArenaInsertError::GenerationExhausted));
        assert_eq!(arena.iter().count(), 0);
    }

    #[test]
    fn iteration_is_in_ascending_slot_order_after_reuse() {
        // Arrange
        let mut arena = Arena::<_, BodyId>::new(test_world(), 3);
        let first = arena.insert('a').expect("first insert should fit");
        let second = arena.insert('b').expect("second insert should fit");
        let third = arena.insert('c').expect("third insert should fit");
        arena.remove(second).expect("live handle should remove");

        // Act
        let replacement = arena.insert('d').expect("vacant slot should be reused");
        let ordered: Vec<_> = arena
            .iter()
            .map(|(handle, value)| (handle, *value))
            .collect();

        // Assert
        assert_eq!(
            ordered,
            vec![(first, 'a'), (replacement, 'd'), (third, 'c')]
        );
    }

    #[derive(Debug, Clone, Copy)]
    enum Operation {
        Insert(i16),
        Get(u8),
        Remove(u8),
        CrossWorldGet(u8),
    }

    struct Generator(u64);

    impl Generator {
        fn new(seed: u64) -> Self {
            Self(seed ^ 0x9e37_79b9_7f4a_7c15)
        }

        fn next(&mut self) -> u64 {
            let mut value = self.0;
            value ^= value >> 12;
            value ^= value << 25;
            value ^= value >> 27;
            self.0 = value;
            value.wrapping_mul(0x2545_f491_4f6c_dd1d)
        }
    }

    fn operations(seed: u64) -> Vec<Operation> {
        let mut generator = Generator::new(seed);
        let operation_count = 32
            + usize::try_from(generator.next() % 33).expect("a value below 33 always fits usize");
        (0..operation_count)
            .map(|_| {
                let selector = generator.next();
                let operand =
                    u8::try_from(generator.next() % 64).expect("a value below 64 always fits u8");
                match selector % 4 {
                    0 => Operation::Insert(i16::from(operand)),
                    1 => Operation::Get(operand),
                    2 => Operation::Remove(operand),
                    _ => Operation::CrossWorldGet(operand),
                }
            })
            .collect()
    }

    #[derive(Clone)]
    enum ModelSlot {
        Occupied { generation: u64, value: i16 },
        Vacant { generation: u64 },
        Retired,
    }

    struct Model {
        world: WorldKey,
        slots: Vec<ModelSlot>,
        free_slots: Vec<usize>,
        retired_slots: usize,
        max_slots: usize,
    }

    impl Model {
        fn new(world: WorldKey, max_slots: usize) -> Self {
            Self {
                world,
                slots: Vec::new(),
                free_slots: Vec::new(),
                retired_slots: 0,
                max_slots,
            }
        }

        fn insert(&mut self, value: i16) -> Result<BodyId, ArenaInsertError> {
            if let Some(slot_index) = self.free_slots.pop() {
                let ModelSlot::Vacant { generation } = self.slots[slot_index] else {
                    unreachable!("model free list contains only vacant slots")
                };
                self.slots[slot_index] = ModelSlot::Occupied { generation, value };
                return Ok(BodyId::from_identity(Identity::new(
                    self.world, slot_index, generation,
                )));
            }

            if self.slots.len() >= self.max_slots {
                if self.retired_slots > 0 {
                    return Err(ArenaInsertError::GenerationExhausted);
                }
                return Err(ArenaInsertError::CapacityExceeded {
                    limit: self.max_slots,
                });
            }

            let slot_index = self.slots.len();
            self.slots.push(ModelSlot::Occupied {
                generation: 0,
                value,
            });
            Ok(BodyId::from_identity(Identity::new(
                self.world, slot_index, 0,
            )))
        }

        fn get(&self, handle: BodyId) -> Result<i16, HandleError> {
            let identity = handle.identity();
            if identity.world() != self.world {
                return Err(HandleError::WrongWorld);
            }
            let Some(ModelSlot::Occupied { generation, value }) = self.slots.get(identity.slot())
            else {
                return Err(HandleError::StaleOrDestroyed);
            };
            if *generation != identity.generation() {
                return Err(HandleError::StaleOrDestroyed);
            }
            Ok(*value)
        }

        fn remove(&mut self, handle: BodyId) -> Result<i16, HandleError> {
            self.get(handle)?;
            let identity = handle.identity();
            let previous = std::mem::replace(&mut self.slots[identity.slot()], ModelSlot::Retired);
            let ModelSlot::Occupied { generation, value } = previous else {
                unreachable!("validated model handles refer to occupied slots")
            };
            let Some(next_generation) = generation.checked_add(1) else {
                self.retired_slots += 1;
                return Ok(value);
            };
            self.slots[identity.slot()] = ModelSlot::Vacant {
                generation: next_generation,
            };
            self.free_slots.push(identity.slot());
            Ok(value)
        }

        fn entries(&self) -> Vec<(BodyId, i16)> {
            self.slots
                .iter()
                .enumerate()
                .filter_map(|(slot_index, slot)| match slot {
                    ModelSlot::Occupied { generation, value } => Some((
                        BodyId::from_identity(Identity::new(self.world, slot_index, *generation)),
                        *value,
                    )),
                    ModelSlot::Vacant { .. } | ModelSlot::Retired => None,
                })
                .collect()
        }
    }

    #[test]
    fn seeded_operation_sequences_match_reference_model() {
        for seed in 0..128 {
            // Arrange
            let world = test_world();
            let other_world = test_world();
            let mut arena = Arena::<_, BodyId>::new(world, 8);
            let mut model = Model::new(world, 8);
            let mut handles = Vec::new();
            let sequence = operations(seed);

            for (step, operation) in sequence.iter().copied().enumerate() {
                // Act
                match operation {
                    Operation::Insert(value) => {
                        let actual = arena.insert(value);
                        let expected = model.insert(value);
                        assert_eq!(
                            actual,
                            expected,
                            "seed={seed}, operation_prefix={:?}",
                            &sequence[..=step]
                        );
                        if let Ok(handle) = actual {
                            handles.push(handle);
                        }
                    }
                    Operation::Get(selector) => {
                        if let Some(handle) = selected_handle(&handles, selector) {
                            let actual = arena.get(handle).copied();
                            let expected = model.get(handle);
                            assert_eq!(
                                actual,
                                expected,
                                "seed={seed}, operation_prefix={:?}",
                                &sequence[..=step]
                            );
                        }
                    }
                    Operation::Remove(selector) => {
                        if let Some(handle) = selected_handle(&handles, selector) {
                            let actual = arena.remove(handle);
                            let expected = model.remove(handle);
                            assert_eq!(
                                actual,
                                expected,
                                "seed={seed}, operation_prefix={:?}",
                                &sequence[..=step]
                            );
                        }
                    }
                    Operation::CrossWorldGet(selector) => {
                        if let Some(handle) = selected_handle(&handles, selector) {
                            let identity = handle.identity();
                            let foreign = BodyId::from_identity(Identity::new(
                                other_world,
                                identity.slot(),
                                identity.generation(),
                            ));
                            assert_eq!(
                                arena.get(foreign),
                                Err(HandleError::WrongWorld),
                                "seed={seed}, operation_prefix={:?}",
                                &sequence[..=step]
                            );
                        }
                    }
                }

                // Assert
                let actual_entries: Vec<_> = arena
                    .iter()
                    .map(|(handle, value)| (handle, *value))
                    .collect();
                assert_eq!(
                    actual_entries,
                    model.entries(),
                    "seed={seed}, operation_prefix={:?}",
                    &sequence[..=step]
                );
            }
        }
    }

    fn selected_handle(handles: &[BodyId], selector: u8) -> Option<BodyId> {
        if handles.is_empty() {
            return None;
        }
        Some(handles[usize::from(selector) % handles.len()])
    }
}
