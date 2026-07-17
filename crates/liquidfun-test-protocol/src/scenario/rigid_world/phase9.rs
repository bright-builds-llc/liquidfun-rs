use serde::{Deserialize, Serialize};

use crate::{FloatBits, ScenarioId, Vec2Bits};

/// Maximum particle systems declared by one bounded Phase 9 timeline.
pub const PHASE9_MAXIMUM_PARTICLE_SYSTEMS: usize = 16;
/// Maximum particles declared by one bounded Phase 9 timeline.
pub const PHASE9_MAXIMUM_PARTICLES: usize = 256;
/// Maximum stable identities carried by one Phase 9 range or observation.
pub const PHASE9_MAXIMUM_IDENTITIES: usize = 256;

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
