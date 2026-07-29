//! Closed Phase 9 policy registry.

use liquidfun_test_protocol::Phase9ParticleObservation;

/// Closed identity of the reviewed Phase 9 declaration and policy registry.
pub const PHASE9_REGISTRY_ID: &str = "phase9-v1";

/// Named comparison class assigned to a reviewed Phase 9 semantic path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase9PolicyKind {
    /// Identity, order, multiplicity, branch, or count equality.
    ExactDiscrete,
    /// IEEE-754 or byte field equality.
    ExactBits,
    /// Reviewed ULP distance for iterative vector state.
    Ulps,
    /// Reviewed absolute-relative bound for accumulated values.
    AbsoluteRelative,
    /// Unit-specific absolute bound for ray or mass values.
    DimensionedAbsolute,
}

/// Every required Phase 9 policy path. Absence from this list fails closed.
pub const PHASE9_REQUIRED_POLICY_PATHS: &[&str] = &[
    "particle.storage.identity",
    "particle.capacity.mode",
    "particle.permutation.order",
    "particle.lifetime.order",
    "particle.zombie.lifecycle",
    "particle.contact.identity",
    "particle.strict_contact.branch",
    "particle.filter.decision",
    "particle.listener.occurrence",
    "particle.force.range",
    "particle.statistics.counts",
    "particle.query.order",
    "particle.query.culling",
    "particle.coupling.identity",
    "particle.configuration.bits",
    "particle.position",
    "particle.velocity",
    "particle.contact.normal",
    "particle.contact.weight",
    "particle.statistics.collision_energy",
    "particle.ray.fraction",
    "particle.body_contact.mass",
];

/// Returns the reviewed policy for a closed Phase 9 path.
#[must_use]
pub fn phase9_policy_for_path(path: &str) -> Option<Phase9PolicyKind> {
    match path {
        "particle.storage.identity"
        | "particle.capacity.mode"
        | "particle.permutation.order"
        | "particle.lifetime.order"
        | "particle.zombie.lifecycle"
        | "particle.contact.identity"
        | "particle.strict_contact.branch"
        | "particle.filter.decision"
        | "particle.listener.occurrence"
        | "particle.force.range"
        | "particle.statistics.counts"
        | "particle.query.order"
        | "particle.query.culling"
        | "particle.coupling.identity" => Some(Phase9PolicyKind::ExactDiscrete),
        "particle.configuration.bits" => Some(Phase9PolicyKind::ExactBits),
        "particle.position" | "particle.velocity" | "particle.contact.normal" => {
            Some(Phase9PolicyKind::Ulps)
        }
        "particle.contact.weight" | "particle.statistics.collision_energy" => {
            Some(Phase9PolicyKind::AbsoluteRelative)
        }
        "particle.ray.fraction" | "particle.body_contact.mass" => {
            Some(Phase9PolicyKind::DimensionedAbsolute)
        }
        _ => None,
    }
}

/// Returns whether an observation belongs to the closed Phase 9 registry.
#[must_use]
pub const fn phase9_observation_is_declared(observation: &Phase9ParticleObservation) -> bool {
    match observation {
        Phase9ParticleObservation::System { .. }
        | Phase9ParticleObservation::Particle { .. }
        | Phase9ParticleObservation::Lifecycle { .. }
        | Phase9ParticleObservation::ParticleContact { .. }
        | Phase9ParticleObservation::BodyContact { .. }
        | Phase9ParticleObservation::Statistics { .. }
        | Phase9ParticleObservation::Query { .. }
        | Phase9ParticleObservation::RayCast { .. }
        | Phase9ParticleObservation::MixedState { .. } => true,
    }
}
