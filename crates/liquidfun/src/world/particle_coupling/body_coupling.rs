//! Candidate body impulse coupling used by particle solver passes.

use crate::arena::Arena;
use crate::math::Vec2;
use crate::particle::solver::material::MaterialBodyCoupling;
use crate::particle::solver::pressure::BodyCoupling;
use crate::{BodyId, StepError, WakePolicy};

use super::super::object::Body;

pub(super) struct CandidateBodyCoupling<'a> {
    bodies: &'a mut Arena<Body, BodyId>,
    maybe_error: Option<crate::BodyControlError>,
}

impl<'a> CandidateBodyCoupling<'a> {
    pub(super) fn new(bodies: &'a mut Arena<Body, BodyId>) -> Self {
        Self {
            bodies,
            maybe_error: None,
        }
    }

    pub(super) fn finish(self) -> Result<(), StepError> {
        self.maybe_error
            .map_or(Ok(()), |error| Err(StepError::ParticleCoupling(error)))
    }

    fn apply(&mut self, body: BodyId, impulse: Vec2, point: Vec2) {
        if self.maybe_error.is_some() {
            return;
        }
        let record = self
            .bodies
            .get_mut(body)
            .expect("solver contact validation retains live body identities");
        match record
            .state
            .candidate_apply_linear_impulse(impulse, point, WakePolicy::Wake)
        {
            Ok(state) => record.state = state,
            Err(error) => self.maybe_error = Some(error),
        }
    }
}

impl MaterialBodyCoupling for CandidateBodyCoupling<'_> {
    fn contains_body(&self, body: BodyId) -> bool {
        self.bodies.get(body).is_ok()
    }

    fn velocity_at(&self, body: BodyId, point: Vec2) -> Vec2 {
        let state = self
            .bodies
            .get(body)
            .expect("validated body contact retains a live body")
            .state;
        state.solver_linear()
            + Vec2::scalar_cross(state.solver_angular(), point - state.sweep().center())
    }

    fn apply_linear_impulse(&mut self, body: BodyId, impulse: Vec2, point: Vec2) {
        self.apply(body, impulse, point);
    }
}

impl BodyCoupling for CandidateBodyCoupling<'_> {
    fn contains_body(&self, body: BodyId) -> bool {
        MaterialBodyCoupling::contains_body(self, body)
    }

    fn velocity_at(&self, body: BodyId, point: Vec2) -> Vec2 {
        MaterialBodyCoupling::velocity_at(self, body, point)
    }

    fn apply_linear_impulse(&mut self, body: BodyId, impulse: Vec2, point: Vec2) {
        self.apply(body, impulse, point);
    }
}
