use crate::arena::Arena;
use crate::collision::{BroadPhase, CollisionOutcome, collide_shapes, test_overlap};
#[cfg(feature = "differential-internals")]
use crate::math::settings::MAX_SUB_STEPS;
use crate::{BodyId, FixtureId, JointId};

use super::body::BodyType;
use super::contact::{
    Contact, ContactEndpoint, ContactKey, ContactTransition, ContactTransitionKind,
    canonical_contact_key,
};
#[cfg(test)]
use super::contact::{ToiAlpha, ToiCountLimitReached};
use super::contact_solver::ContactSolve;
use super::joint::JointRecord;
use super::object::{Body, Fixture};
use super::proxy::FixtureProxy;
use super::step::{CollisionDecisionHook, ContactHookRun, FixturePairSnapshot, StepError};

mod update;
use update::{
    apply_contact_hook, broad_phase_overlap, fixture_pair_snapshot, link_contact, pair_is_eligible,
    unlink_contact, update_contact, wake_contact_bodies,
};

#[derive(Debug, Clone, Default)]
pub(super) struct ContactManager {
    contacts: Vec<Contact>,
    next_ordinal: u64,
    transitions: Vec<ContactTransition>,
}

impl ContactManager {
    pub(super) const fn new() -> Self {
        Self {
            contacts: Vec::new(),
            next_ordinal: 0,
            transitions: Vec::new(),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.contacts.len()
    }

    pub(super) fn contacts(&self) -> &[Contact] {
        &self.contacts
    }

    pub(super) fn contact_mut(&mut self, index: usize) -> Option<&mut Contact> {
        self.contacts.get_mut(index)
    }

    pub(super) fn contact_index_for_ordinal(&self, ordinal: u64) -> Option<usize> {
        self.contacts
            .iter()
            .position(|contact| contact.ordinal == ordinal)
    }

    pub(super) fn reset_toi_state(&mut self) {
        for contact in &mut self.contacts {
            contact.reset_toi_state();
        }
    }

    pub(super) fn invalidate_toi_for_body(&mut self, body: BodyId) {
        for contact in &mut self.contacts {
            if contact.key.first.body == body || contact.key.second.body == body {
                contact.invalidate_toi();
            }
        }
    }

    pub(super) fn refresh_continuous_contact_with_hook<H: CollisionDecisionHook>(
        &mut self,
        index: usize,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &Arena<Fixture, FixtureId>,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<Option<()>, StepError> {
        let Some(contact) = self.contacts.get_mut(index) else {
            return Ok(None);
        };
        let update = update_contact(contact, bodies, fixtures);
        if let Some(transition) = update.maybe_transition {
            self.transitions.push(transition.clone());
            hook_run.record_contact(transition)?;
        }
        let contact = self
            .contacts
            .get_mut(index)
            .expect("refreshed continuous contact remains live");
        apply_contact_hook(contact, update.maybe_previous_manifold.as_ref(), hook_run)?;
        Ok(Some(()))
    }

    #[cfg(feature = "differential-internals")]
    pub(super) fn exhaust_toi_budget_for_diagnostic(&mut self, ordinal: u64) -> Option<()> {
        let contact = self
            .contacts
            .iter_mut()
            .find(|contact| contact.ordinal == ordinal)?;
        contact.reset_toi_state();
        for _ in 0..=MAX_SUB_STEPS {
            contact
                .increment_toi_count()
                .expect("the named diagnostic budget remains representable");
        }
        Some(())
    }

    #[cfg(test)]
    pub(super) fn seed_toi_state_for_test(
        &mut self,
        ordinal: u64,
        alpha: f32,
        count: usize,
    ) -> Result<(), ToiCountLimitReached> {
        let alpha = ToiAlpha::new(alpha).ok_or(ToiCountLimitReached)?;
        let contact = self
            .contacts
            .iter_mut()
            .find(|contact| contact.ordinal == ordinal)
            .expect("test contact occurrence must remain live");
        contact.seed_toi_state(alpha, count)
    }

    #[cfg(test)]
    pub(super) fn toi_state_for_test(&self, ordinal: u64) -> Option<(Option<f32>, usize)> {
        self.contacts
            .iter()
            .find(|contact| contact.ordinal == ordinal)
            .map(|contact| {
                (
                    contact.maybe_toi_alpha().map(ToiAlpha::get),
                    contact.toi_count(),
                )
            })
    }

    #[cfg(test)]
    pub(super) fn increment_toi_count_for_test(
        &mut self,
        ordinal: u64,
    ) -> Result<(), ToiCountLimitReached> {
        let contact = self
            .contacts
            .iter_mut()
            .find(|contact| contact.ordinal == ordinal)
            .expect("test contact occurrence must remain live");
        contact.increment_toi_count()
    }

    pub(super) fn find_new_contacts<H: CollisionDecisionHook>(
        &mut self,
        broad_phase: &mut BroadPhase<FixtureProxy>,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
        joints: &Arena<JointRecord, JointId>,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let mut pairs = Vec::new();
        broad_phase
            .update_pairs(|_first_id, first, _second_id, second| pairs.push((*first, *second)))
            .expect("world-owned broad-phase entries must remain coherent");

        for (first, second) in pairs {
            self.add_pair(first, second, bodies, fixtures, joints, hook_run)?;
        }
        Ok(())
    }

    pub(super) fn update_contacts<H: CollisionDecisionHook>(
        &mut self,
        broad_phase: &BroadPhase<FixtureProxy>,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
        joints: &Arena<JointRecord, JointId>,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let mut index = 0;
        while index < self.contacts.len() {
            let key = self.contacts[index].key;
            if self.contacts[index].needs_filtering() {
                let eligible = pair_is_eligible(key, bodies, fixtures, joints)
                    && hook_run.should_collide(&fixture_pair_snapshot(key))?;
                if !eligible {
                    self.destroy_contact_with_hook(index, bodies, fixtures, hook_run)?;
                    continue;
                }
            }
            self.contacts[index].set_needs_filtering(false);

            if !broad_phase_overlap(key, broad_phase, fixtures) {
                self.destroy_contact_with_hook(index, bodies, fixtures, hook_run)?;
                continue;
            }

            let update = update_contact(&mut self.contacts[index], bodies, fixtures);
            if let Some(transition) = update.maybe_transition {
                self.transitions.push(transition.clone());
                hook_run.record_contact(transition)?;
            }
            apply_contact_hook(
                &mut self.contacts[index],
                update.maybe_previous_manifold.as_ref(),
                hook_run,
            )?;
            fixtures
                .get_mut(key.first.fixture)
                .expect("contact fixture A remains live")
                .pending_refilter = false;
            fixtures
                .get_mut(key.second.fixture)
                .expect("contact fixture B remains live")
                .pending_refilter = false;
            index += 1;
        }
        Ok(())
    }

    pub(super) fn flag_fixture_for_filtering(&mut self, fixture: FixtureId) {
        for contact in &mut self.contacts {
            if contact.key.first.fixture == fixture || contact.key.second.fixture == fixture {
                contact.set_needs_filtering(true);
            }
        }
    }

    #[cfg(test)]
    pub(super) fn set_hook_enabled(&mut self, ordinal: u64, enabled: bool) {
        let contact = self
            .contacts
            .iter_mut()
            .find(|contact| contact.ordinal == ordinal)
            .expect("test hook occurrence must remain live");
        contact.set_enabled(enabled);
    }

    pub(super) fn destroy_for_body(
        &mut self,
        body: BodyId,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
    ) {
        let mut index = 0;
        while index < self.contacts.len() {
            let key = self.contacts[index].key;
            if key.first.body == body || key.second.body == body {
                self.destroy_contact(index, bodies, fixtures);
            } else {
                index += 1;
            }
        }
    }

    pub(super) fn destroy_for_fixture(
        &mut self,
        fixture: FixtureId,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
    ) {
        let mut index = 0;
        while index < self.contacts.len() {
            let key = self.contacts[index].key;
            if key.first.fixture == fixture || key.second.fixture == fixture {
                self.destroy_contact(index, bodies, fixtures);
            } else {
                index += 1;
            }
        }
    }

    pub(super) fn drain_transitions(&mut self) -> Vec<ContactTransition> {
        std::mem::take(&mut self.transitions)
    }

    pub(super) fn transition_checkpoint(&self) -> usize {
        self.transitions.len()
    }

    pub(super) fn drain_transitions_since(&mut self, checkpoint: usize) -> Vec<ContactTransition> {
        self.transitions.split_off(checkpoint)
    }

    #[cfg(feature = "differential-internals")]
    pub(super) fn rigid_diagnostics(
        &self,
    ) -> Vec<crate::rigid_differential::RigidContactDiagnostic> {
        self.contacts
            .iter()
            .map(|contact| {
                crate::rigid_differential::RigidContactDiagnostic::new(
                    contact.ordinal + 1,
                    contact.snapshot(),
                )
            })
            .collect()
    }

    #[cfg(test)]
    pub(super) fn seed_first_impulses_for_test(&mut self, normal: f32, tangent: f32) {
        let contact = self
            .contacts
            .first_mut()
            .expect("test contact should exist before seeding impulses");
        let feature_id = contact.points[0].feature_id();
        contact.store_impulses(&[(feature_id, normal, tangent)]);
    }

    pub(super) fn commit_impulses(
        &mut self,
        contact_index: usize,
        impulses: &[(crate::collision::ContactFeatureId, f32, f32)],
    ) -> ContactSolve {
        let contact = self
            .contacts
            .get_mut(contact_index)
            .expect("staged island contact remains live during commit");
        contact.store_impulses(impulses);
        ContactSolve::new(contact.snapshot())
    }

    pub(super) fn maybe_staged_solve(
        &self,
        contact_index: usize,
        impulses: &[(crate::collision::ContactFeatureId, f32, f32)],
    ) -> Option<ContactSolve> {
        self.contacts
            .get(contact_index)
            .map(|contact| ContactSolve::new(contact.staged_snapshot(impulses)))
    }

    fn add_pair<H: CollisionDecisionHook>(
        &mut self,
        first_proxy: FixtureProxy,
        second_proxy: FixtureProxy,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
        joints: &Arena<JointRecord, JointId>,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let first_fixture = fixtures
            .get(first_proxy.fixture)
            .expect("broad-phase fixture A must remain live");
        let second_fixture = fixtures
            .get(second_proxy.fixture)
            .expect("broad-phase fixture B must remain live");
        let first = ContactEndpoint {
            fixture: first_proxy.fixture,
            body: first_proxy.body,
            child_index: first_proxy.child_index,
        };
        let second = ContactEndpoint {
            fixture: second_proxy.fixture,
            body: second_proxy.body,
            child_index: second_proxy.child_index,
        };
        let Some(key) = canonical_contact_key(
            first,
            first_fixture.definition.shape(),
            second,
            second_fixture.definition.shape(),
        ) else {
            return Ok(());
        };
        if !pair_is_eligible(key, bodies, fixtures, joints) {
            return Ok(());
        }
        if self
            .contacts
            .iter()
            .any(|contact| contact.key.matches_unordered(key))
        {
            return Ok(());
        }
        if !hook_run.should_collide(&fixture_pair_snapshot(key))? {
            return Ok(());
        }

        let fixture_a = fixtures
            .get(key.first.fixture)
            .expect("canonical fixture A remains live");
        let fixture_b = fixtures
            .get(key.second.fixture)
            .expect("canonical fixture B remains live");
        let ordinal = self.next_ordinal;
        self.next_ordinal = self
            .next_ordinal
            .checked_add(1)
            .expect("private contact occurrence ordinals cannot exhaust in one process");
        let solid = !fixture_a.definition.is_sensor() && !fixture_b.definition.is_sensor();
        let contact = Contact::new(
            key,
            ordinal,
            fixture_a.definition.friction(),
            fixture_b.definition.friction(),
            fixture_a.definition.restitution(),
            fixture_b.definition.restitution(),
        );
        self.contacts.insert(0, contact);
        link_contact(ordinal, key, bodies, fixtures);
        if solid {
            wake_contact_bodies(key, bodies);
        }
        Ok(())
    }

    fn destroy_contact_with_hook<H: CollisionDecisionHook>(
        &mut self,
        index: usize,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let maybe_transition = self.destroy_contact(index, bodies, fixtures);
        if let Some(transition) = maybe_transition {
            hook_run.record_contact_destruction(transition)?;
        }
        Ok(())
    }

    fn destroy_contact(
        &mut self,
        index: usize,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
    ) -> Option<ContactTransition> {
        let contact = self.contacts.remove(index);
        if contact.is_touching() && !contact.is_sensor() {
            wake_contact_bodies(contact.key, bodies);
        }
        unlink_contact(contact.ordinal, contact.key, bodies, fixtures);
        if contact.is_touching() {
            let transition = ContactTransition::new(ContactTransitionKind::End, contact.snapshot());
            self.transitions.push(transition.clone());
            return Some(transition);
        }
        None
    }
}
