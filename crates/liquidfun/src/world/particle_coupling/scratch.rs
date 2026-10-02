//! Reusable allocation ownership, separate from authoritative step backups.

use super::CcdFixtureRecord;
use crate::collision::{Aabb, ChildIndex};
use crate::particle::body_contact::FixtureContactSource;
use crate::particle::solver::boundary::{BoundaryBuffers, FilteredCollisionHit};

#[derive(Default)]
pub(in crate::world) struct ParticleStepScratch {
    pub(super) boundary: BoundaryBuffers,
    pub(super) body_sources: Vec<FixtureContactSource>,
    pub(super) collision: CollisionBuffers,
}

#[derive(Default)]
pub(super) struct CollisionBuffers {
    pub(super) fixtures: Vec<CcdFixtureRecord>,
    pub(super) child_pool: Vec<Vec<(ChildIndex, Option<Aabb>)>>,
    pub(super) hits: Vec<FilteredCollisionHit>,
}

impl CollisionBuffers {
    pub(super) fn recycle_records(&mut self) {
        for fixture in self.fixtures.drain(..) {
            let mut children = fixture.children;
            children.clear();
            self.child_pool.push(children);
        }
    }
}

impl ParticleStepScratch {
    pub(super) fn clear_transient(&mut self) {
        self.boundary.clear();
        self.body_sources.clear();
        self.collision.recycle_records();
        self.collision.hits.clear();
    }
}
