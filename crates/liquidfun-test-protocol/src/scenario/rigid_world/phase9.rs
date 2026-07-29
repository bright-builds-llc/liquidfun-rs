use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{FloatBits, ScenarioId, Vec2Bits};

mod witness;
pub use witness::{
    Phase9WitnessBinding, Phase9WitnessBindingError, Phase9WitnessBindingErrorKind,
    validate_phase9_witness_bindings,
};
use witness::{observed_branch_kind, requires_specific_assertion};

/// Maximum particle systems declared by one bounded Phase 9 timeline.
pub const PHASE9_MAXIMUM_PARTICLE_SYSTEMS: usize = 16;
/// Maximum particles declared by one bounded Phase 9 timeline.
pub const PHASE9_MAXIMUM_PARTICLES: usize = 256;
/// Maximum stable identities carried by one Phase 9 range or observation.
pub const PHASE9_MAXIMUM_IDENTITIES: usize = 256;
/// Exact number of reviewed semantic branches in the Phase 9 evidence corpus.
pub const PHASE9_MAXIMUM_WITNESS_BINDINGS: usize = 58;
/// Maximum checkpoints addressable by one Phase 9 witness binding.
pub const PHASE9_MAXIMUM_WITNESS_CHECKPOINTS: usize = 64;

/// Exact reviewed Phase 9 semantic branch registry, one identifier per line.
pub const PHASE9_REQUIRED_BRANCH_IDS: &str = "multiple_systems\nnewest_first\npaused_system\nstable_ids_sort\nstable_ids_compact\noptional_lanes\nfixed_buffer\ngrowable_buffer\nfixed_full\nteardown\nfinite_lifetime\ninfinite_lifetime\nequal_lifetime\noldest_lifetime\nmaximum_lifetime\nrequested_destruction_callback\nunrequested_destruction_callback\nzombie_pending\ncapacity_eviction\nparticle_contact\nbody_contact\nstrict_contact_enabled\nstrict_contact_disabled\nlistener_flag_enabled\nlistener_flag_disabled\nfilter_flag_enabled\nfilter_flag_disabled\ncontact_order\ncontact_multiplicity\ncoupling_fields\ndynamic_body_reaction\nstatic_body_no_reaction\nforce_range\nimpulse_range\nstatistics_counts\ncollision_energy\nstuck_candidates\nsystem_aabb\nworld_aabb\nsystem_culling\nquery_continue\nquery_terminate\nsystem_ray\nworld_ray\nray_culling\nray_start_inside_exclusion\nray_ignore\nray_continue\nray_clip\nray_terminate\nretained_phase6_through_phase8\nphase10_rejection\nclosed_policy_registry\nreplay_identity\nminimization_identity\nfirst_divergence_stability\nd0_byte_identity\ndebug_release_agreement";

/// Closed Phase 9 observation kind used by semantic witness bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase9ObservationKind {
    /// Evidence established across complete persisted case results rather than one observation.
    CaseEvidence,
    System,
    Particle,
    Lifecycle,
    ParticleContact,
    BodyContact,
    Statistics,
    Query,
    RayCast,
    MixedState,
}

/// Closed semantic assertion surface for the Phase 9 evidence corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase9SemanticAssertion {
    ObservedSemantic { branch_id: ScenarioId },
    FiniteLifetimeExpired { particle_id: ScenarioId },
    InfiniteLifetimeSurvives { particle_id: ScenarioId },
    EqualExpirationOrder { particle_ids: Box<[ScenarioId]> },
    StrictContactCardinality { enabled: bool, contact_count: u32 },
    ListenerEventEffect { enabled: bool, event_count: u32 },
    FilterContactEffect { enabled: bool, contact_count: u32 },
    CollisionEnergyPositiveFinite { minimum_bits: FloatBits },
    StuckCandidatesNonempty { particle_ids: Box<[ScenarioId]> },
    ReplayResultDigestEquality,
    MinimizedFailureSignaturePreservation,
    DeliberateFirstDivergence,
    D0RepeatedResultDigestEquality,
    DebugReleaseResultDigestEquality,
}

impl Phase9SemanticAssertion {
    /// Returns the exact observation kind required by this assertion.
    #[must_use]
    pub fn expected_observation_kind(&self) -> Phase9ObservationKind {
        match self {
            Self::ObservedSemantic { branch_id } => {
                observed_branch_kind(branch_id.as_str()).unwrap_or(Phase9ObservationKind::Particle)
            }
            Self::FiniteLifetimeExpired { .. } | Self::InfiniteLifetimeSurvives { .. } => {
                Phase9ObservationKind::System
            }
            Self::ListenerEventEffect { enabled, .. } => {
                if *enabled {
                    Phase9ObservationKind::Lifecycle
                } else {
                    Phase9ObservationKind::Statistics
                }
            }
            Self::EqualExpirationOrder { .. } => Phase9ObservationKind::Lifecycle,
            Self::StrictContactCardinality { .. }
            | Self::FilterContactEffect { .. }
            | Self::CollisionEnergyPositiveFinite { .. }
            | Self::StuckCandidatesNonempty { .. } => Phase9ObservationKind::Statistics,
            Self::ReplayResultDigestEquality
            | Self::MinimizedFailureSignaturePreservation
            | Self::DeliberateFirstDivergence
            | Self::D0RepeatedResultDigestEquality
            | Self::DebugReleaseResultDigestEquality => Phase9ObservationKind::CaseEvidence,
        }
    }

    /// Returns whether this assertion is established by a typed persisted case proof.
    #[must_use]
    pub const fn requires_case_evidence(&self) -> bool {
        matches!(
            self,
            Self::ReplayResultDigestEquality
                | Self::MinimizedFailureSignaturePreservation
                | Self::DeliberateFirstDivergence
                | Self::D0RepeatedResultDigestEquality
                | Self::DebugReleaseResultDigestEquality
        )
    }

    fn branch_id(&self) -> &str {
        match self {
            Self::ObservedSemantic { branch_id } => branch_id.as_str(),
            Self::FiniteLifetimeExpired { .. } => "finite_lifetime",
            Self::InfiniteLifetimeSurvives { .. } => "infinite_lifetime",
            Self::EqualExpirationOrder { .. } => "equal_lifetime",
            Self::StrictContactCardinality { enabled, .. } => {
                if *enabled {
                    "strict_contact_enabled"
                } else {
                    "strict_contact_disabled"
                }
            }
            Self::ListenerEventEffect { enabled, .. } => {
                if *enabled {
                    "listener_flag_enabled"
                } else {
                    "listener_flag_disabled"
                }
            }
            Self::FilterContactEffect { enabled, .. } => {
                if *enabled {
                    "filter_flag_enabled"
                } else {
                    "filter_flag_disabled"
                }
            }
            Self::CollisionEnergyPositiveFinite { .. } => "collision_energy",
            Self::StuckCandidatesNonempty { .. } => "stuck_candidates",
            Self::ReplayResultDigestEquality => "replay_identity",
            Self::MinimizedFailureSignaturePreservation => "minimization_identity",
            Self::DeliberateFirstDivergence => "first_divergence_stability",
            Self::D0RepeatedResultDigestEquality => "d0_byte_identity",
            Self::DebugReleaseResultDigestEquality => "debug_release_agreement",
        }
    }

    fn is_valid(&self) -> bool {
        match self {
            Self::ObservedSemantic { branch_id } => {
                observed_branch_kind(branch_id.as_str()).is_some()
                    && !requires_specific_assertion(branch_id.as_str())
            }
            Self::EqualExpirationOrder { particle_ids } => {
                particle_ids.len() >= 2
                    && particle_ids.len() <= PHASE9_MAXIMUM_IDENTITIES
                    && particle_ids.iter().collect::<HashSet<_>>().len() == particle_ids.len()
            }
            Self::StrictContactCardinality {
                enabled,
                contact_count,
            } => !*enabled || *contact_count > 0,
            Self::ListenerEventEffect {
                enabled,
                event_count,
            } => (*enabled && *event_count > 0) || (!*enabled && *event_count == 0),
            Self::CollisionEnergyPositiveFinite { minimum_bits } => {
                let minimum = minimum_bits.to_f32();
                minimum.is_finite() && minimum > 0.0
            }
            Self::StuckCandidatesNonempty { particle_ids } => {
                !particle_ids.is_empty()
                    && particle_ids.len() <= PHASE9_MAXIMUM_IDENTITIES
                    && particle_ids.iter().collect::<HashSet<_>>().len() == particle_ids.len()
            }
            Self::FiniteLifetimeExpired { .. }
            | Self::InfiniteLifetimeSurvives { .. }
            | Self::FilterContactEffect { .. }
            | Self::ReplayResultDigestEquality
            | Self::MinimizedFailureSignaturePreservation
            | Self::DeliberateFirstDivergence
            | Self::D0RepeatedResultDigestEquality
            | Self::DebugReleaseResultDigestEquality => true,
        }
    }
}

/// Closed query callback control used by executable Phase 9 witnesses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase9QueryControl {
    #[default]
    Continue,
    Terminate,
}

/// Closed ray callback control used by executable Phase 9 witnesses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase9RayControl {
    Ignore,
    #[default]
    Continue,
    Clip,
    Terminate,
}

/// Caller-owned buffer capacity contract used by a Phase 9 particle system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase9ParticleBufferMode {
    Growable { initial_capacity: usize },
    Fixed { capacity: usize },
}

impl Phase9ParticleBufferMode {
    #[must_use]
    pub const fn capacity(self) -> usize {
        match self {
            Self::Growable { initial_capacity } => initial_capacity,
            Self::Fixed { capacity } => capacity,
        }
    }
}

/// Bounded declaration for one native particle system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9ParticleSystemDeclaration {
    pub system_id: ScenarioId,
    pub buffer_mode: Phase9ParticleBufferMode,
    pub paused: bool,
    pub strict_contact_check: bool,
    pub stuck_threshold: u32,
    pub density_bits: FloatBits,
    pub gravity_scale_bits: FloatBits,
    pub radius_bits: FloatBits,
    pub damping_bits: FloatBits,
    pub destruction_by_age: bool,
    pub lifetime_granularity_bits: FloatBits,
    pub maximum_count: Option<usize>,
}

/// Stable-ID declaration for one native particle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9ParticleDeclaration {
    pub particle_id: ScenarioId,
    pub system_id: ScenarioId,
    pub position: Vec2Bits,
    pub velocity: Vec2Bits,
    pub flags_bits: u32,
    pub color: [u8; 4],
    pub lifetime_bits: FloatBits,
}

/// Closed Phase 9 particle action surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase9ParticleAction {
    CreateSystem {
        system_id: ScenarioId,
    },
    DestroySystem {
        system_id: ScenarioId,
    },
    CreateParticle {
        particle_id: ScenarioId,
    },
    InspectSystem {
        system_id: ScenarioId,
    },
    InspectParticle {
        particle_id: ScenarioId,
    },
    InspectParticleContact {
        system_id: ScenarioId,
        contact_index: usize,
    },
    InspectBodyContact {
        system_id: ScenarioId,
        contact_index: usize,
    },
    InspectOccurrence {
        occurrence_index: usize,
    },
    SetPaused {
        system_id: ScenarioId,
        paused: bool,
    },
    SetPosition {
        particle_id: ScenarioId,
        position: Vec2Bits,
    },
    SetVelocity {
        particle_id: ScenarioId,
        velocity: Vec2Bits,
    },
    MarkForDestruction {
        particle_id: ScenarioId,
    },
    Compact {
        system_id: ScenarioId,
    },
    ApplyForce {
        particle_ids: Box<[ScenarioId]>,
        force: Vec2Bits,
    },
    ApplyImpulse {
        particle_ids: Box<[ScenarioId]>,
        impulse: Vec2Bits,
    },
    RequestStatistics {
        system_id: ScenarioId,
    },
    QueryAabb {
        system_id: Option<ScenarioId>,
        lower: Vec2Bits,
        upper: Vec2Bits,
        #[serde(default)]
        control: Phase9QueryControl,
    },
    RayCast {
        system_id: Option<ScenarioId>,
        start: Vec2Bits,
        end: Vec2Bits,
        #[serde(default)]
        control: Phase9RayControl,
    },
}

/// Semantic Phase 9 callback or lifecycle occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase9OccurrenceKind {
    FilterDecision,
    ContactCreated,
    ContactDestroyed,
    ParticleDestroyed,
    SystemDestroyed,
    QueryVisited,
    RayVisited,
}

/// One source-ordered Phase 9 lifecycle occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9Occurrence {
    pub ordinal: u32,
    pub kind: Phase9OccurrenceKind,
    pub system_id: ScenarioId,
    pub maybe_particle_id: Option<ScenarioId>,
    pub maybe_other_particle_id: Option<ScenarioId>,
    pub maybe_fixture_id: Option<ScenarioId>,
}

/// Stable semantic particle snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9ParticleSnapshot {
    pub particle_id: ScenarioId,
    pub system_id: ScenarioId,
    pub position: Vec2Bits,
    pub velocity: Vec2Bits,
    pub flags_bits: u32,
    pub color: [u8; 4],
    pub weight_bits: FloatBits,
    pub force: Vec2Bits,
    pub pending_destruction: bool,
}

/// Stable semantic particle-particle contact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9ParticleContactObservation {
    pub system_id: ScenarioId,
    pub particle_a_id: ScenarioId,
    pub particle_b_id: ScenarioId,
    pub flags_bits: u32,
    pub weight_bits: FloatBits,
    pub normal: Vec2Bits,
}

/// Stable semantic fixture-particle contact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9BodyContactObservation {
    pub system_id: ScenarioId,
    pub particle_id: ScenarioId,
    pub body_id: ScenarioId,
    pub fixture_id: ScenarioId,
    pub weight_bits: FloatBits,
    pub normal: Vec2Bits,
    pub mass_bits: FloatBits,
}

/// Owned statistics record without allocator or dense-index leakage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9StatisticsObservation {
    pub maybe_system_id: Option<ScenarioId>,
    pub system_count: u32,
    pub particle_count: u32,
    pub pending_particle_count: u32,
    pub particle_contact_count: u32,
    pub body_contact_count: u32,
    pub stuck_particle_ids: Box<[ScenarioId]>,
    pub collision_energy_bits: FloatBits,
    pub declared_capacity: u32,
    pub effective_capacity: u32,
}

/// Closed Phase 9 semantic observation registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase9ParticleObservation {
    System {
        system_id: ScenarioId,
        paused: bool,
        particle_ids: Box<[ScenarioId]>,
    },
    Particle {
        snapshot: Phase9ParticleSnapshot,
    },
    Lifecycle {
        occurrence: Phase9Occurrence,
    },
    ParticleContact {
        contact: Phase9ParticleContactObservation,
    },
    BodyContact {
        contact: Phase9BodyContactObservation,
    },
    Statistics {
        statistics: Phase9StatisticsObservation,
    },
    Query {
        terminated: bool,
        particle_ids: Box<[ScenarioId]>,
    },
    RayCast {
        terminated: bool,
        particle_ids: Box<[ScenarioId]>,
        fractions_bits: Box<[FloatBits]>,
    },
    MixedState {
        body_ids: Box<[ScenarioId]>,
        particle_ids: Box<[ScenarioId]>,
    },
}

impl Phase9ParticleObservation {
    /// Returns the closed witness kind for this semantic observation.
    #[must_use]
    pub const fn witness_kind(&self) -> Phase9ObservationKind {
        match self {
            Self::System { .. } => Phase9ObservationKind::System,
            Self::Particle { .. } => Phase9ObservationKind::Particle,
            Self::Lifecycle { .. } => Phase9ObservationKind::Lifecycle,
            Self::ParticleContact { .. } => Phase9ObservationKind::ParticleContact,
            Self::BodyContact { .. } => Phase9ObservationKind::BodyContact,
            Self::Statistics { .. } => Phase9ObservationKind::Statistics,
            Self::Query { .. } => Phase9ObservationKind::Query,
            Self::RayCast { .. } => Phase9ObservationKind::RayCast,
            Self::MixedState { .. } => Phase9ObservationKind::MixedState,
        }
    }
}
