//! Native timeline state and semantic identity maps.

use liquidfun::{BodyId, FixtureId, JointId, ManagedContactSnapshot, PreSolveDirective, World};
use liquidfun_test_protocol::{
    Phase9Occurrence, RigidContactEvent, RigidContactIdentity, RigidDestructionRecord,
    RigidWorldActionRecord, RigidWorldObservation, RigidWorldWitness, RigidWorldWitnessFamily,
    ScenarioId,
};

use super::{NativeRigidWorldError, action_error, checked_u32, phase10};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Observation {
    pub(super) witness: RigidWorldWitness,
    pub(super) maybe_contact: Option<RigidContactIdentity>,
}

pub(crate) struct TimelineExecutor {
    pub(crate) family: RigidWorldWitnessFamily,
    pub(crate) world: World,
    pub(crate) bodies: Vec<(ScenarioId, BodyId)>,
    pub(crate) fixtures: Vec<(ScenarioId, FixtureId)>,
    pub(super) fixture_owners: Vec<(FixtureId, BodyId)>,
    pub(crate) joints: Vec<(ScenarioId, JointId)>,
    pub(crate) particle_systems: Vec<(ScenarioId, liquidfun::ParticleSystemId)>,
    pub(crate) particles: Vec<(
        ScenarioId,
        liquidfun::ParticleSystemId,
        liquidfun::ParticleId,
    )>,
    pub(super) ropes: Vec<(ScenarioId, liquidfun::rope::Rope)>,
    pub(super) filter_directives: Vec<(FixtureId, FixtureId, bool)>,
    pub(super) pre_solve_directives: Vec<(FixtureId, FixtureId, PreSolveDirective)>,
    contact_identities: Vec<(u64, RigidContactIdentity)>,
    pub(super) seen_manager_occurrences: Vec<u64>,
    pub(super) seen_lifecycle_occurrences: Vec<u64>,
    pub(super) next_lifecycle_ordinal: u32,
    pub(crate) next_phase9_occurrence_ordinal: u32,
    pub(crate) phase9_occurrences: Vec<Phase9Occurrence>,
    pub(super) phase10: phase10::NativePhase10State,
    pub(super) maybe_last_contact: Option<RigidContactIdentity>,
    pub(super) events: Vec<RigidContactEvent>,
    pub(super) destructions: Vec<RigidDestructionRecord>,
    pub(super) observations: Vec<Observation>,
    pub(super) semantic_observations: Vec<RigidWorldObservation>,
}

impl TimelineExecutor {
    pub(super) fn new(family: RigidWorldWitnessFamily) -> Result<Self, NativeRigidWorldError> {
        let mut world = World::new().map_err(|error| NativeRigidWorldError::Action {
            action_id: "world-create".into(),
            message: error.to_string().into(),
        })?;
        if matches!(
            family,
            RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle
                | RigidWorldWitnessFamily::SingleContactLifecycle
        ) {
            world
                .set_continuous_physics_enabled(false)
                .map_err(|error| NativeRigidWorldError::Action {
                    action_id: "world-configure".into(),
                    message: error.to_string().into(),
                })?;
        }
        Ok(Self {
            family,
            world,
            bodies: Vec::new(),
            fixtures: Vec::new(),
            fixture_owners: Vec::new(),
            joints: Vec::new(),
            particle_systems: Vec::new(),
            particles: Vec::new(),
            ropes: Vec::new(),
            filter_directives: Vec::new(),
            pre_solve_directives: Vec::new(),
            contact_identities: Vec::new(),
            seen_manager_occurrences: Vec::new(),
            seen_lifecycle_occurrences: Vec::new(),
            next_lifecycle_ordinal: 0,
            next_phase9_occurrence_ordinal: 0,
            phase9_occurrences: Vec::new(),
            phase10: phase10::NativePhase10State::default(),
            maybe_last_contact: None,
            events: Vec::new(),
            destructions: Vec::new(),
            observations: Vec::new(),
            semantic_observations: Vec::new(),
        })
    }

    pub(super) fn body(
        &self,
        id: &ScenarioId,
        action: &RigidWorldActionRecord,
    ) -> Result<BodyId, NativeRigidWorldError> {
        self.bodies
            .iter()
            .find_map(|(candidate, body)| (candidate == id).then_some(*body))
            .ok_or_else(|| action_error(action, format!("unknown body `{id}`")))
    }

    pub(super) fn fixture(
        &self,
        id: &ScenarioId,
        action: &RigidWorldActionRecord,
    ) -> Result<FixtureId, NativeRigidWorldError> {
        self.fixtures
            .iter()
            .find_map(|(candidate, fixture)| (candidate == id).then_some(*fixture))
            .ok_or_else(|| action_error(action, format!("unknown fixture `{id}`")))
    }

    pub(crate) fn joint(
        &self,
        id: &ScenarioId,
        action: &RigidWorldActionRecord,
    ) -> Result<JointId, NativeRigidWorldError> {
        self.joints
            .iter()
            .find_map(|(candidate, joint)| (candidate == id).then_some(*joint))
            .ok_or_else(|| action_error(action, format!("unknown joint `{id}`")))
    }

    pub(crate) fn semantic_body(&self, body: BodyId) -> Result<ScenarioId, NativeRigidWorldError> {
        self.bodies
            .iter()
            .find_map(|(id, candidate)| (*candidate == body).then(|| id.clone()))
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "body-map".into(),
                message: "native body was not mapped to a semantic identity".into(),
            })
    }

    pub(crate) fn semantic_joint(
        &self,
        joint: JointId,
    ) -> Result<ScenarioId, NativeRigidWorldError> {
        self.joints
            .iter()
            .find_map(|(id, candidate)| (*candidate == joint).then(|| id.clone()))
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "joint-map".into(),
                message: "native joint was not mapped to a semantic identity".into(),
            })
    }

    pub(crate) fn semantic_fixture(
        &self,
        fixture: FixtureId,
    ) -> Result<ScenarioId, NativeRigidWorldError> {
        self.fixtures
            .iter()
            .find_map(|(id, candidate)| (*candidate == fixture).then(|| id.clone()))
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "contact-map".into(),
                message: "manager contact referenced an undeclared fixture".into(),
            })
    }

    pub(super) fn contact_identity(
        &mut self,
        contact: &ManagedContactSnapshot,
    ) -> Result<RigidContactIdentity, NativeRigidWorldError> {
        let manager_occurrence = contact.differential_occurrence();
        if let Some((_, identity)) = self
            .contact_identities
            .iter()
            .find(|(candidate, _)| *candidate == manager_occurrence)
        {
            return Ok(identity.clone());
        }
        let fixtures = contact.fixtures();
        let children = contact.child_indices();
        let first_fixture_id = self.semantic_fixture(fixtures[0])?;
        let second_fixture_id = self.semantic_fixture(fixtures[1])?;
        let child_a = checked_u32(children[0].get(), "contact-child")?;
        let child_b = checked_u32(children[1].get(), "contact-child")?;
        let prior_occurrences = self
            .contact_identities
            .iter()
            .filter(|(_, identity)| {
                identity.fixture_a_id() == &first_fixture_id
                    && identity.child_a() == child_a
                    && identity.fixture_b_id() == &second_fixture_id
                    && identity.child_b() == child_b
            })
            .count();
        let occurrence = u32::try_from(prior_occurrences)
            .ok()
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "contact-occurrence".into(),
                message: "contact occurrence exceeded the protocol representation".into(),
            })?;
        let identity = RigidContactIdentity::new(
            first_fixture_id,
            child_a,
            second_fixture_id,
            child_b,
            occurrence,
        )?;
        self.contact_identities
            .push((manager_occurrence, identity.clone()));
        Ok(identity)
    }

    pub(super) fn push_observation(
        &mut self,
        witness: RigidWorldWitness,
        maybe_contact: Option<RigidContactIdentity>,
    ) {
        let observation = Observation {
            witness,
            maybe_contact,
        };
        if !self.observations.contains(&observation) {
            self.observations.push(observation);
        }
    }
}
