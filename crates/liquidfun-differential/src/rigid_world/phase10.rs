//! Native Phase 10 particle-group adapter.

mod comparator;
mod native;

pub use comparator::{
    PHASE10_POLICY_REGISTRY, PHASE10_REQUIRED_POLICY_PATHS, Phase10ComparatorError,
    Phase10ComparisonMode, Phase10ComparisonOutcome, Phase10Mismatch, Phase10Policy,
    Phase10PolicyKind, compare_phase10_observations, validate_phase10_policy_registry,
};
pub(super) use native::{NativePhase10State, execute_action};
