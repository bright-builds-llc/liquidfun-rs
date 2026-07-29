use super::{
    Arena, Body, BodyId, BodyType, BroadPhase, CollisionDecisionHook, CollisionOutcome, Contact,
    ContactHookRun, ContactKey, ContactTransition, ContactTransitionKind, Fixture, FixtureId,
    FixturePairSnapshot, FixtureProxy, JointId, JointRecord, StepError, collide_shapes,
    test_overlap,
};

pub(super) fn pair_is_eligible(
    key: ContactKey,
    bodies: &Arena<Body, BodyId>,
    fixtures: &Arena<Fixture, FixtureId>,
    joints: &Arena<JointRecord, JointId>,
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
    if body_a.joints.iter().any(|joint_id| {
        let joint = joints
            .get(*joint_id)
            .expect("body joint adjacency contains only live joints");
        !joint.collide_connected
            && (joint.bodies == [key.first.body, key.second.body]
                || joint.bodies == [key.second.body, key.first.body])
    }) {
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

pub(super) fn broad_phase_overlap(
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

pub(super) struct ContactUpdate {
    pub(super) maybe_transition: Option<ContactTransition>,
    pub(super) maybe_previous_manifold: Option<crate::collision::Manifold>,
}

pub(super) fn update_contact(
    contact: &mut Contact,
    bodies: &mut Arena<Body, BodyId>,
    fixtures: &Arena<Fixture, FixtureId>,
) -> ContactUpdate {
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
    let maybe_previous_manifold = contact.maybe_manifold.clone();
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
    ContactUpdate {
        maybe_transition: kind.map(|kind| ContactTransition::new(kind, contact.snapshot())),
        maybe_previous_manifold,
    }
}

pub(super) fn fixture_pair_snapshot(key: ContactKey) -> FixturePairSnapshot {
    FixturePairSnapshot::new(
        [key.first.fixture, key.second.fixture],
        [key.first.body, key.second.body],
        [key.first.child_index, key.second.child_index],
    )
}

pub(super) fn apply_contact_hook<H: CollisionDecisionHook>(
    contact: &mut Contact,
    maybe_previous_manifold: Option<&crate::collision::Manifold>,
    hook_run: &mut ContactHookRun<'_, H>,
) -> Result<(), StepError> {
    if !contact.is_touching() {
        return Ok(());
    }
    let snapshot = contact.snapshot();
    let directive =
        hook_run.contact_updated(&snapshot, maybe_previous_manifold, !snapshot.is_sensor())?;
    let (maybe_friction, maybe_restitution, maybe_tangent_speed) = directive.material_controls();
    contact.apply_pre_solve_controls(
        directive.enabled(),
        maybe_friction,
        maybe_restitution,
        maybe_tangent_speed,
    );
    Ok(())
}

pub(super) fn wake_contact_bodies(key: ContactKey, bodies: &mut Arena<Body, BodyId>) {
    for body_id in [key.first.body, key.second.body] {
        let body = bodies
            .get_mut(body_id)
            .expect("contact body remains live while applying wake transition");
        body.state = body.state.candidate_set_awake(true);
        body.pending_wake = false;
    }
}

pub(super) fn link_contact(
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

pub(super) fn unlink_contact(
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
