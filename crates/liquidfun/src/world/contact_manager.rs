use crate::arena::Arena;
use crate::collision::{BroadPhase, CollisionOutcome, collide_shapes, test_overlap};
#[cfg(feature = "differential-internals")]
use crate::math::settings::MAX_SUB_STEPS;
use crate::{BodyId, FixtureId};

use super::body::BodyType;
use super::contact::{
    Contact, ContactEndpoint, ContactKey, ContactTransition, ContactTransitionKind,
    ManagedContactSnapshot, canonical_contact_key,
};
#[cfg(test)]
use super::contact::{ToiAlpha, ToiCountLimitReached};
use super::contact_solver::ContactSolve;
use super::object::{Body, Fixture};
use super::proxy::FixtureProxy;

#[derive(Debug, Clone, Default)]
pub(super) struct ContactManager {
    contacts: Vec<Contact>,
    next_ordinal: u64,
    transitions: Vec<ContactTransition>,
}

#[derive(Debug, Clone)]
pub(super) struct HookContactOccurrence {
    pub(super) ordinal: u64,
    pub(super) snapshot: ManagedContactSnapshot,
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

    pub(super) fn refresh_continuous_contact(
        &mut self,
        index: usize,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &Arena<Fixture, FixtureId>,
    ) -> Option<()> {
        let contact = self.contacts.get_mut(index)?;
        let maybe_transition = update_contact(contact, bodies, fixtures);
        if let Some(transition) = maybe_transition {
            self.transitions.push(transition);
        }
        Some(())
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

    pub(super) fn find_new_contacts(
        &mut self,
        broad_phase: &mut BroadPhase<FixtureProxy>,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
    ) {
        let mut pairs = Vec::new();
        broad_phase
            .update_pairs(|_first_id, first, _second_id, second| pairs.push((*first, *second)))
            .expect("world-owned broad-phase entries must remain coherent");

        for (first, second) in pairs {
            self.add_pair(first, second, bodies, fixtures);
        }
    }

    pub(super) fn update_contacts(
        &mut self,
        broad_phase: &BroadPhase<FixtureProxy>,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
    ) {
        let mut index = 0;
        while index < self.contacts.len() {
            let key = self.contacts[index].key;
            if self.contacts[index].needs_filtering() && !pair_is_eligible(key, bodies, fixtures) {
                self.destroy_contact(index, bodies, fixtures);
                continue;
            }
            self.contacts[index].set_needs_filtering(false);

            if !broad_phase_overlap(key, broad_phase, fixtures) {
                self.destroy_contact(index, bodies, fixtures);
                continue;
            }

            let maybe_transition = update_contact(&mut self.contacts[index], bodies, fixtures);
            if let Some(transition) = maybe_transition {
                self.transitions.push(transition);
            }
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
    }

    pub(super) fn flag_fixture_for_filtering(&mut self, fixture: FixtureId) {
        for contact in &mut self.contacts {
            if contact.key.first.fixture == fixture || contact.key.second.fixture == fixture {
                contact.set_needs_filtering(true);
            }
        }
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

    pub(super) fn hook_contacts(&self) -> Vec<HookContactOccurrence> {
        self.contacts
            .iter()
            .filter(|contact| contact.is_touching())
            .map(|contact| HookContactOccurrence {
                ordinal: contact.ordinal,
                snapshot: contact.snapshot(),
            })
            .collect()
    }

    pub(super) fn set_hook_enabled(&mut self, ordinal: u64, enabled: bool) {
        let contact = self
            .contacts
            .iter_mut()
            .find(|contact| contact.ordinal == ordinal)
            .expect("hook occurrence must remain live while the world is locked");
        contact.set_enabled(enabled);
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

    fn add_pair(
        &mut self,
        first_proxy: FixtureProxy,
        second_proxy: FixtureProxy,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
    ) {
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
            return;
        };
        if !pair_is_eligible(key, bodies, fixtures)
            || self
                .contacts
                .iter()
                .any(|contact| contact.key.matches_unordered(key))
        {
            return;
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
    }

    fn destroy_contact(
        &mut self,
        index: usize,
        bodies: &mut Arena<Body, BodyId>,
        fixtures: &mut Arena<Fixture, FixtureId>,
    ) {
        let contact = self.contacts.remove(index);
        if contact.is_touching() && !contact.is_sensor() {
            wake_contact_bodies(contact.key, bodies);
        }
        unlink_contact(contact.ordinal, contact.key, bodies, fixtures);
        if contact.is_touching() {
            self.transitions.push(ContactTransition::new(
                ContactTransitionKind::End,
                contact.snapshot(),
            ));
        }
    }
}

fn pair_is_eligible(
    key: ContactKey,
    bodies: &Arena<Body, BodyId>,
    fixtures: &Arena<Fixture, FixtureId>,
) -> bool {
    if key.first.body == key.second.body {
        return false;
    }
    let body_a = bodies
        .get(key.first.body)
        .expect("contact body A must remain live");
    let body_b = bodies
        .get(key.second.body)
        .expect("contact body B must remain live");
    if !body_a.state.snapshot().is_active() || !body_b.state.snapshot().is_active() {
        return false;
    }
    let at_least_one_dynamic = body_a.state.snapshot().body_type() == BodyType::Dynamic
        || body_b.state.snapshot().body_type() == BodyType::Dynamic;
    if !at_least_one_dynamic {
        return false;
    }
    let fixture_a = fixtures
        .get(key.first.fixture)
        .expect("contact fixture A must remain live");
    let fixture_b = fixtures
        .get(key.second.fixture)
        .expect("contact fixture B must remain live");
    fixture_a
        .definition
        .filter_data()
        .should_collide(fixture_b.definition.filter_data())
}

fn broad_phase_overlap(
    key: ContactKey,
    broad_phase: &BroadPhase<FixtureProxy>,
    fixtures: &Arena<Fixture, FixtureId>,
) -> bool {
    let fixture_a = fixtures
        .get(key.first.fixture)
        .expect("contact fixture A remains live");
    let fixture_b = fixtures
        .get(key.second.fixture)
        .expect("contact fixture B remains live");
    let Some(proxy_a) = fixture_a.proxies.maybe_proxy_id(key.first.child_index) else {
        return false;
    };
    let Some(proxy_b) = fixture_b.proxies.maybe_proxy_id(key.second.child_index) else {
        return false;
    };
    let aabb_a = broad_phase
        .fat_aabb(proxy_a)
        .expect("contact proxy A remains live");
    let aabb_b = broad_phase
        .fat_aabb(proxy_b)
        .expect("contact proxy B remains live");
    aabb_a.overlaps(aabb_b)
}

fn update_contact(
    contact: &mut Contact,
    bodies: &mut Arena<Body, BodyId>,
    fixtures: &Arena<Fixture, FixtureId>,
) -> Option<ContactTransition> {
    let fixture_a = fixtures
        .get(contact.key.first.fixture)
        .expect("contact fixture A remains live");
    let fixture_b = fixtures
        .get(contact.key.second.fixture)
        .expect("contact fixture B remains live");
    let transform_a = bodies
        .get(contact.key.first.body)
        .expect("contact body A remains live")
        .state
        .transform();
    let transform_b = bodies
        .get(contact.key.second.body)
        .expect("contact body B remains live")
        .state
        .transform();
    let was_touching = contact.is_touching();
    contact.set_enabled(true);
    contact.set_sensor(fixture_a.definition.is_sensor() || fixture_b.definition.is_sensor());
    let touching = if contact.is_sensor() {
        contact.clear_manifold();
        test_overlap(
            fixture_a.definition.shape(),
            contact.key.first.child_index,
            transform_a,
            fixture_b.definition.shape(),
            contact.key.second.child_index,
            transform_b,
        )
        .expect("checked contact geometry must produce a finite overlap result")
    } else {
        let maybe_manifold = match collide_shapes(
            fixture_a.definition.shape(),
            contact.key.first.child_index,
            transform_a,
            fixture_b.definition.shape(),
            contact.key.second.child_index,
            transform_b,
        )
        .expect("checked contact geometry must produce a finite manifold result")
        {
            CollisionOutcome::Touching(pair) => Some(pair.manifold().clone()),
            CollisionOutcome::Separated => None,
            CollisionOutcome::Unsupported => {
                unreachable!("only registered shape pairs become contacts")
            }
        };
        let touching = maybe_manifold.is_some();
        contact.replace_manifold(maybe_manifold);
        touching
    };

    if !contact.is_sensor() && touching != was_touching {
        wake_contact_bodies(contact.key, bodies);
    }
    contact.set_touching(touching);
    let kind = match (was_touching, contact.is_touching()) {
        (false, true) => Some(ContactTransitionKind::Begin),
        (true, true) => Some(ContactTransitionKind::Persist),
        (true, false) => Some(ContactTransitionKind::End),
        (false, false) => None,
    };
    kind.map(|kind| ContactTransition::new(kind, contact.snapshot()))
}

fn wake_contact_bodies(key: ContactKey, bodies: &mut Arena<Body, BodyId>) {
    for body_id in [key.first.body, key.second.body] {
        let body = bodies
            .get_mut(body_id)
            .expect("contact body remains live while applying wake transition");
        body.state = body.state.candidate_set_awake(true);
        body.pending_wake = false;
    }
}

fn link_contact(
    ordinal: u64,
    key: ContactKey,
    bodies: &mut Arena<Body, BodyId>,
    fixtures: &mut Arena<Fixture, FixtureId>,
) {
    bodies
        .get_mut(key.first.body)
        .expect("contact body A remains live")
        .contacts
        .insert(0, ordinal);
    bodies
        .get_mut(key.second.body)
        .expect("contact body B remains live")
        .contacts
        .insert(0, ordinal);
    fixtures
        .get_mut(key.first.fixture)
        .expect("contact fixture A remains live")
        .contacts
        .insert(0, ordinal);
    fixtures
        .get_mut(key.second.fixture)
        .expect("contact fixture B remains live")
        .contacts
        .insert(0, ordinal);
}

fn unlink_contact(
    ordinal: u64,
    key: ContactKey,
    bodies: &mut Arena<Body, BodyId>,
    fixtures: &mut Arena<Fixture, FixtureId>,
) {
    remove_contact(
        &mut bodies
            .get_mut(key.first.body)
            .expect("live body A")
            .contacts,
        ordinal,
    );
    remove_contact(
        &mut bodies
            .get_mut(key.second.body)
            .expect("live body B")
            .contacts,
        ordinal,
    );
    remove_contact(
        &mut fixtures
            .get_mut(key.first.fixture)
            .expect("live fixture A")
            .contacts,
        ordinal,
    );
    remove_contact(
        &mut fixtures
            .get_mut(key.second.fixture)
            .expect("live fixture B")
            .contacts,
        ordinal,
    );
}

fn remove_contact(adjacency: &mut Vec<u64>, ordinal: u64) {
    let position = adjacency
        .iter()
        .position(|candidate| *candidate == ordinal)
        .expect("contact adjacency contains the manager occurrence");
    adjacency.remove(position);
}
