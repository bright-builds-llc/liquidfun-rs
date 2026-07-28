#![allow(
    dead_code,
    reason = "the private arena foundation is consumed by later world storage plans"
)]

use std::marker::PhantomData;

use crate::error::{ArenaInsertError, HandleError};
use crate::identity::{ErasedHandle, HandleIdentity, Identity, IdentityScope, WorldKey};

#[derive(Clone)]
enum Slot<T> {
    Occupied {
        generation: u64,
        maybe_particle_system: Option<IdentityScope>,
        value: T,
    },
    Vacant {
        generation: u64,
    },
    Retired,
}

#[derive(Clone)]
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
        self.insert_with_scope(value, None)
    }

    pub(crate) fn next_handle(&self) -> Result<H, ArenaInsertError> {
        let (slot_index, generation) = if let Some(slot_index) = self.free_slots.last().copied() {
            let slot = self
                .slots
                .get(slot_index)
                .expect("a free-slot index always refers to an existing slot");
            let Slot::Vacant { generation } = slot else {
                unreachable!("only vacant slots are placed on the free list")
            };
            (slot_index, *generation)
        } else {
            if self.slots.len() >= self.max_slots {
                return Err(self.exhaustion_error());
            }
            (self.slots.len(), 0)
        };

        Ok(H::from_identity(Identity::new(
            self.world, slot_index, generation,
        )))
    }

    pub(crate) fn insert_particle(
        &mut self,
        value: T,
        system: Identity,
    ) -> Result<H, ArenaInsertError> {
        self.insert_with_scope(value, Some(system.scope()))
    }

    fn insert_with_scope(
        &mut self,
        value: T,
        maybe_particle_system: Option<IdentityScope>,
    ) -> Result<H, ArenaInsertError> {
        let Some(slot_index) = self.free_slots.pop() else {
            return self.insert_new_slot(value, maybe_particle_system);
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
        *slot = Slot::Occupied {
            generation,
            maybe_particle_system,
            value,
        };

        Ok(H::from_identity(identity_for_slot(
            self.world,
            slot_index,
            generation,
            maybe_particle_system,
        )))
    }

    fn insert_new_slot(
        &mut self,
        value: T,
        maybe_particle_system: Option<IdentityScope>,
    ) -> Result<H, ArenaInsertError> {
        if self.slots.len() >= self.max_slots {
            return Err(self.exhaustion_error());
        }

        let slot_index = self.slots.len();
        let generation = 0;
        self.slots.push(Slot::Occupied {
            generation,
            maybe_particle_system,
            value,
        });

        Ok(H::from_identity(identity_for_slot(
            self.world,
            slot_index,
            generation,
            maybe_particle_system,
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

    pub(crate) fn get_mut(&mut self, handle: H) -> Result<&mut T, HandleError> {
        self.validate_typed_handle(handle)?;
        let identity = handle.identity();
        let Slot::Occupied { value, .. } = &mut self.slots[identity.slot()] else {
            unreachable!("validated handles always refer to occupied slots")
        };
        Ok(value)
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
        let Slot::Occupied {
            generation,
            maybe_particle_system,
            value,
        } = slot
        else {
            return Err(HandleError::StaleOrDestroyed);
        };
        if *generation != identity.generation() {
            return Err(HandleError::StaleOrDestroyed);
        }
        if *maybe_particle_system != identity.maybe_particle_system() {
            return Err(HandleError::WrongParticleSystem);
        }

        Ok(value)
    }

    pub(crate) fn remove(&mut self, handle: H) -> Result<T, HandleError> {
        self.validate_typed_handle(handle)?;
        let identity = handle.identity();
        let slot = &mut self.slots[identity.slot()];
        let previous = std::mem::replace(slot, Slot::Retired);
        let Slot::Occupied {
            generation, value, ..
        } = previous
        else {
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
                Slot::Occupied {
                    generation,
                    maybe_particle_system,
                    value,
                } => Some((
                    H::from_identity(identity_for_slot(
                        self.world,
                        slot_index,
                        *generation,
                        *maybe_particle_system,
                    )),
                    value,
                )),
                Slot::Vacant { .. } | Slot::Retired => None,
            })
    }

    pub(crate) fn values_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.slots.iter_mut().filter_map(|slot| match slot {
            Slot::Occupied { value, .. } => Some(value),
            Slot::Vacant { .. } | Slot::Retired => None,
        })
    }
}

fn identity_for_slot(
    world: WorldKey,
    slot: usize,
    generation: u64,
    maybe_particle_system: Option<IdentityScope>,
) -> Identity {
    let Some(system) = maybe_particle_system else {
        return Identity::new(world, slot, generation);
    };
    Identity::new_particle(world, slot, generation, system.identity())
}

#[cfg(test)]
mod tests;
