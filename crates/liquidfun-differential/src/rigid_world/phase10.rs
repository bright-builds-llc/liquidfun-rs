//! Native Phase 10 particle-group adapter.

mod comparator;
mod evidence;
mod native;

pub use comparator::{
    PHASE10_POLICY_REGISTRY, PHASE10_REQUIRED_POLICY_PATHS, Phase10ComparatorError,
    Phase10ComparisonMode, Phase10ComparisonOutcome, Phase10Mismatch, Phase10Policy,
    Phase10PolicyCalibration, Phase10PolicyKind, compare_phase10_observations,
    phase10_policy_calibrations, validate_phase10_policy_registry,
};
pub use evidence::{
    PHASE10_EVIDENCE_SCHEMA_VERSION, Phase10EvidenceBinding, Phase10EvidenceContractError,
    Phase10EvidenceLeaf, Phase10EvidencePayloads, Phase10EvidenceTestRefs,
    Phase10EvidenceWitnessRef, required_phase10_evidence_leaves,
    validate_phase10_evidence_contract,
};
pub(super) use native::{NativePhase10State, execute_action};
