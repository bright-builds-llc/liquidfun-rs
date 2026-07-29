use super::{
    Deserialize, Phase9ParticleDeclaration, Phase9ParticleSystemDeclaration, ProtocolVersion,
    RequestId, RigidBodyDeclaration, RigidFixtureDeclaration, RigidJointDeclaration,
    RigidRopeDeclaration, RigidWorldAction, RigidWorldDecodeError, RigidWorldErrorKind,
    RigidWorldWitness, RigidWorldWitnessFamily, ScenarioId, ScenarioSchemaVersion, ScenarioSource,
    Serialize, Sha256Hex, ToleranceProfileVersion, TraceSchemaVersion, joints_are_empty,
    particle_systems_are_empty, particles_are_empty, ropes_are_empty, validation,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidWorldActionRecord {
    pub(in crate::scenario::rigid_world) action_id: ScenarioId,
    pub(in crate::scenario::rigid_world) phase: Box<str>,
    pub(in crate::scenario::rigid_world) action: RigidWorldAction,
}

impl RigidWorldActionRecord {
    #[must_use]
    pub const fn action_id(&self) -> &ScenarioId {
        &self.action_id
    }

    #[must_use]
    pub fn phase(&self) -> &str {
        &self.phase
    }

    #[must_use]
    pub const fn action(&self) -> &RigidWorldAction {
        &self.action
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidExpectedCounts {
    pub bodies: u32,
    pub fixtures: u32,
    pub contacts: u32,
    pub manifold_points: u32,
    pub events: u32,
    pub destructions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidContactIdentity {
    fixture_a_id: ScenarioId,
    child_a: u32,
    fixture_b_id: ScenarioId,
    child_b: u32,
    occurrence: u32,
}

impl RigidContactIdentity {
    /// Creates one oriented semantic fixture-child occurrence identity.
    ///
    /// # Errors
    ///
    /// Returns [`RigidWorldDecodeError`] when both fixture IDs are equal or the
    /// occurrence ordinal is zero.
    #[allow(
        clippy::similar_names,
        reason = "fixture_a_id and fixture_b_id mirror the oriented protocol contract"
    )]
    pub fn new(
        fixture_a_id: ScenarioId,
        child_a: u32,
        fixture_b_id: ScenarioId,
        child_b: u32,
        occurrence: u32,
    ) -> Result<Self, RigidWorldDecodeError> {
        if fixture_a_id == fixture_b_id || occurrence == 0 {
            return Err(validation(RigidWorldErrorKind::InvalidContactIdentity));
        }
        Ok(Self {
            fixture_a_id,
            child_a,
            fixture_b_id,
            child_b,
            occurrence,
        })
    }

    #[must_use]
    pub const fn fixture_a_id(&self) -> &ScenarioId {
        &self.fixture_a_id
    }

    #[must_use]
    pub const fn child_a(&self) -> u32 {
        self.child_a
    }

    #[must_use]
    pub const fn fixture_b_id(&self) -> &ScenarioId {
        &self.fixture_b_id
    }

    #[must_use]
    pub const fn child_b(&self) -> u32 {
        self.child_b
    }

    #[must_use]
    pub const fn occurrence(&self) -> u32 {
        self.occurrence
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidExpectedTransition {
    pub(in crate::scenario::rigid_world) witness: RigidWorldWitness,
    pub(in crate::scenario::rigid_world) maybe_contact: Option<RigidContactIdentity>,
}

impl RigidExpectedTransition {
    #[must_use]
    pub const fn witness(&self) -> RigidWorldWitness {
        self.witness
    }

    #[must_use]
    pub const fn maybe_contact(&self) -> Option<&RigidContactIdentity> {
        self.maybe_contact.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidExpectedCheckpoint {
    pub(in crate::scenario::rigid_world) checkpoint_id: ScenarioId,
    pub(in crate::scenario::rigid_world) after_action_id: ScenarioId,
    pub(in crate::scenario::rigid_world) phase: Box<str>,
    pub(in crate::scenario::rigid_world) counts: RigidExpectedCounts,
    pub(in crate::scenario::rigid_world) transitions: Box<[RigidExpectedTransition]>,
}

impl RigidExpectedCheckpoint {
    #[must_use]
    pub const fn checkpoint_id(&self) -> &ScenarioId {
        &self.checkpoint_id
    }

    #[must_use]
    pub const fn after_action_id(&self) -> &ScenarioId {
        &self.after_action_id
    }

    #[must_use]
    pub fn phase(&self) -> &str {
        &self.phase
    }

    #[must_use]
    pub const fn counts(&self) -> RigidExpectedCounts {
        self.counts
    }

    #[must_use]
    pub fn transitions(&self) -> &[RigidExpectedTransition] {
        &self.transitions
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidWorldTimeline {
    pub(in crate::scenario::rigid_world) witness_family: RigidWorldWitnessFamily,
    pub(in crate::scenario::rigid_world) bodies: Box<[RigidBodyDeclaration]>,
    pub(in crate::scenario::rigid_world) fixtures: Box<[RigidFixtureDeclaration]>,
    #[serde(default, skip_serializing_if = "joints_are_empty")]
    pub(in crate::scenario::rigid_world) joints: Box<[RigidJointDeclaration]>,
    #[serde(default, skip_serializing_if = "ropes_are_empty")]
    pub(in crate::scenario::rigid_world) ropes: Box<[RigidRopeDeclaration]>,
    #[serde(default, skip_serializing_if = "particle_systems_are_empty")]
    pub(in crate::scenario::rigid_world) particle_systems: Box<[Phase9ParticleSystemDeclaration]>,
    #[serde(default, skip_serializing_if = "particles_are_empty")]
    pub(in crate::scenario::rigid_world) particles: Box<[Phase9ParticleDeclaration]>,
    pub(in crate::scenario::rigid_world) actions: Box<[RigidWorldActionRecord]>,
    pub(in crate::scenario::rigid_world) checkpoints: Box<[RigidExpectedCheckpoint]>,
}

impl RigidWorldTimeline {
    #[must_use]
    pub const fn witness_family(&self) -> RigidWorldWitnessFamily {
        self.witness_family
    }

    #[must_use]
    pub fn bodies(&self) -> &[RigidBodyDeclaration] {
        &self.bodies
    }

    #[must_use]
    pub fn fixtures(&self) -> &[RigidFixtureDeclaration] {
        &self.fixtures
    }

    #[must_use]
    pub fn joints(&self) -> &[RigidJointDeclaration] {
        &self.joints
    }

    #[must_use]
    pub fn ropes(&self) -> &[RigidRopeDeclaration] {
        &self.ropes
    }

    #[must_use]
    pub fn particle_systems(&self) -> &[Phase9ParticleSystemDeclaration] {
        &self.particle_systems
    }

    #[must_use]
    pub fn particles(&self) -> &[Phase9ParticleDeclaration] {
        &self.particles
    }

    #[must_use]
    pub fn actions(&self) -> &[RigidWorldActionRecord] {
        &self.actions
    }

    #[must_use]
    pub fn checkpoints(&self) -> &[RigidExpectedCheckpoint] {
        &self.checkpoints
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidWorldScenario {
    pub(in crate::scenario::rigid_world) scenario_id: ScenarioId,
    pub(in crate::scenario::rigid_world) source: ScenarioSource,
    pub(in crate::scenario::rigid_world) timelines: Box<[RigidWorldTimeline]>,
}

impl RigidWorldScenario {
    #[must_use]
    pub const fn scenario_id(&self) -> &ScenarioId {
        &self.scenario_id
    }

    #[must_use]
    pub const fn source(&self) -> &ScenarioSource {
        &self.source
    }

    #[must_use]
    pub fn timelines(&self) -> &[RigidWorldTimeline] {
        &self.timelines
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidWorldRequestRecord {
    pub(in crate::scenario::rigid_world) protocol_version: ProtocolVersion,
    pub(in crate::scenario::rigid_world) record_kind: RigidWorldRequestKind,
    pub(in crate::scenario::rigid_world) request_id: RequestId,
    pub(in crate::scenario::rigid_world) scenario_schema_version: ScenarioSchemaVersion,
    pub(in crate::scenario::rigid_world) requested_trace_schema_version: TraceSchemaVersion,
    pub(in crate::scenario::rigid_world) tolerance_profile_version: ToleranceProfileVersion,
    pub(in crate::scenario::rigid_world) tolerance_profile_sha256: Sha256Hex,
    pub(in crate::scenario::rigid_world) scenario: RigidWorldScenario,
}

impl RigidWorldRequestRecord {
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    #[must_use]
    pub const fn scenario(&self) -> &RigidWorldScenario {
        &self.scenario
    }

    #[must_use]
    pub const fn tolerance_profile_sha256(&self) -> &Sha256Hex {
        &self.tolerance_profile_sha256
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::scenario::rigid_world) enum RigidWorldRequestKind {
    RigidWorldRequest,
}
