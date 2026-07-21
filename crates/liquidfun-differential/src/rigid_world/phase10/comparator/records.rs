//! Cohesive field walkers for Phase 10 semantic record families.

mod groups;
mod topology;
mod witness;

pub(super) use groups::{compare_group, compare_particle};
pub(super) use topology::{
    compare_body_contact, compare_pair, compare_particle_contact, compare_triad,
};
pub(super) use witness::compare_witness;

macro_rules! check {
    ($scenario:expr, $operation:expr, $entity:expr, $index:expr, $path:expr, $left:expr, $right:expr) => {
        if let Some(found) = crate::rigid_world::phase10::comparator::mismatch_if(
            $scenario, $operation, $entity, $index, $path, &$left, &$right,
        ) {
            return Ok(Some(found));
        }
    };
}

pub(super) use check;
