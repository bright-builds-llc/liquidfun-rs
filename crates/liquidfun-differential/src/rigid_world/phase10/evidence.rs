//! Closed, typed binding contract for Phase 10 semantic evidence.

use std::collections::{HashMap, HashSet};

use liquidfun_test_protocol::{
    PHASE9_REQUIRED_BRANCH_IDS, Phase10BehaviorLeaf, ScenarioId, WitnessRole,
};
use serde::{Deserialize, Serialize};

use super::PHASE10_REQUIRED_POLICY_PATHS;

/// Version of the closed Phase 10 leaf-binding contract.
pub const PHASE10_EVIDENCE_SCHEMA_VERSION: u32 = 1;

const PHASE10_BEHAVIOR_LEAVES: [Phase10BehaviorLeaf; 22] = [
    Phase10BehaviorLeaf::GroupCreate,
    Phase10BehaviorLeaf::GroupAppend,
    Phase10BehaviorLeaf::GroupJoin,
    Phase10BehaviorLeaf::GroupSplit,
    Phase10BehaviorLeaf::GroupFlags,
    Phase10BehaviorLeaf::GroupDestroy,
    Phase10BehaviorLeaf::Water,
    Phase10BehaviorLeaf::Zombie,
    Phase10BehaviorLeaf::Wall,
    Phase10BehaviorLeaf::Spring,
    Phase10BehaviorLeaf::Elastic,
    Phase10BehaviorLeaf::Viscous,
    Phase10BehaviorLeaf::Powder,
    Phase10BehaviorLeaf::Tensile,
    Phase10BehaviorLeaf::ColorMixing,
    Phase10BehaviorLeaf::Barrier,
    Phase10BehaviorLeaf::StaticPressure,
    Phase10BehaviorLeaf::Reactive,
    Phase10BehaviorLeaf::Repulsive,
    Phase10BehaviorLeaf::SolidGroup,
    Phase10BehaviorLeaf::RigidGroup,
    Phase10BehaviorLeaf::BodyInteraction,
];

/// One closed Phase 10 behavior leaf or one retained Phase 9 branch family.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase10EvidenceLeaf {
    /// Portable Phase 10 public behavior, never a private solver pass.
    Phase10 {
        /// Closed public behavior identity.
        behavior: Phase10BehaviorLeaf,
    },
    /// Retained Phase 6-9 evidence identified by the closed Phase 9 registry.
    Inherited {
        /// Closed Phase 9 branch identity retaining Phase 6-9 proof.
        branch_id: ScenarioId,
    },
}

/// Complete expected leaf inventory in reviewed order.
#[must_use]
pub fn required_phase10_evidence_leaves() -> Vec<Phase10EvidenceLeaf> {
    let mut leaves = PHASE10_BEHAVIOR_LEAVES
        .into_iter()
        .map(|behavior| Phase10EvidenceLeaf::Phase10 { behavior })
        .collect::<Vec<_>>();
    leaves.extend(PHASE9_REQUIRED_BRANCH_IDS.lines().map(|branch_id| {
        Phase10EvidenceLeaf::Inherited {
            branch_id: ScenarioId::new(branch_id).expect("reviewed Phase 9 branch IDs are valid"),
        }
    }));
    leaves
}

/// Indexed semantic witness owned by one request checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10EvidenceWitnessRef {
    /// Typed proof role.
    pub role: WitnessRole,
    /// Bound action in the case request.
    pub action_index: usize,
    /// Bound checkpoint in the case request.
    pub checkpoint_index: usize,
    /// Bound semantic observation in the checkpoint result.
    pub observation_index: usize,
}

/// Required native test authorities for one leaf.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10EvidenceTestRefs {
    /// Focused unit or kernel witness.
    pub focused: Box<str>,
    /// Public integration witness.
    pub integration: Box<str>,
    /// Reproducible property-model witness.
    pub property: Box<str>,
}

/// Canonical, independently named persisted proof roles for one case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10EvidencePayloads {
    /// Native semantic result.
    pub native: Box<str>,
    /// Pinned-oracle semantic result.
    pub oracle: Box<str>,
    /// Complete comparator outcome.
    pub comparison: Box<str>,
    /// Independent native replay.
    pub replay_native: Box<str>,
    /// Independent oracle replay.
    pub replay_oracle: Box<str>,
    /// Debug oracle execution.
    pub debug_oracle: Box<str>,
    /// Release oracle execution.
    pub release_oracle: Box<str>,
    /// Minimized deliberate divergence.
    pub minimized: Box<str>,
    /// Independent copied divergence.
    pub copied: Box<str>,
    /// Retained Phase 6-9 semantic proof.
    pub inherited: Box<str>,
}

impl Phase10EvidencePayloads {
    fn entries(&self) -> [(&'static str, &str); 10] {
        [
            ("native", &self.native),
            ("oracle", &self.oracle),
            ("comparison", &self.comparison),
            ("replay-native", &self.replay_native),
            ("replay-oracle", &self.replay_oracle),
            ("debug-oracle", &self.debug_oracle),
            ("release-oracle", &self.release_oracle),
            ("minimized", &self.minimized),
            ("copied", &self.copied),
            ("inherited", &self.inherited),
        ]
    }
}

/// Complete implementation, test, witness, policy, and payload binding for one leaf.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10EvidenceBinding {
    /// Closed semantic identity.
    pub leaf: Phase10EvidenceLeaf,
    /// Owning bounded corpus case.
    pub case_id: ScenarioId,
    /// Native implementation reference.
    pub implementation: Box<str>,
    /// Three independent native test authorities.
    pub tests: Phase10EvidenceTestRefs,
    /// Inactive control witness.
    pub control: Phase10EvidenceWitnessRef,
    /// Active semantic witness.
    pub activation: Phase10EvidenceWitnessRef,
    /// Required multi-flag or ordering interaction, when applicable.
    pub maybe_interaction: Option<Phase10EvidenceWitnessRef>,
    /// Exact semantic result path observed by the witness.
    pub observation_path: Box<str>,
    /// Closed comparator policy path.
    pub policy_path: Box<str>,
    /// Canonical independently persisted proof roles.
    pub payloads: Phase10EvidencePayloads,
}

/// One malformed or incomplete leaf contract was rejected before evidence promotion.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid Phase 10 evidence contract: {reason}")]
pub struct Phase10EvidenceContractError {
    reason: Box<str>,
}

/// Validates exact inventory closure and every executable binding.
///
/// # Errors
///
/// Fails for unknown, missing, duplicate, declaration-only, out-of-range,
/// aliased, unsafe, unbound-policy, or private-pass evidence.
pub fn validate_phase10_evidence_contract(
    bindings: &[Phase10EvidenceBinding],
    case_bounds: &HashMap<ScenarioId, (usize, usize, usize)>,
) -> Result<(), Phase10EvidenceContractError> {
    let required = required_phase10_evidence_leaves();
    if bindings.len() != required.len() {
        return Err(contract_error(format!(
            "expected {} leaves, found {}",
            required.len(),
            bindings.len()
        )));
    }
    let mut seen = HashSet::with_capacity(bindings.len());
    for binding in bindings {
        if !seen.insert(binding.leaf.clone()) {
            return Err(contract_error("duplicate semantic leaf"));
        }
        validate_binding(binding, case_bounds)?;
    }
    let actual = bindings
        .iter()
        .map(|binding| &binding.leaf)
        .collect::<HashSet<_>>();
    if required.iter().any(|leaf| !actual.contains(leaf)) {
        return Err(contract_error("missing or unknown semantic leaf"));
    }
    Ok(())
}

fn validate_binding(
    binding: &Phase10EvidenceBinding,
    case_bounds: &HashMap<ScenarioId, (usize, usize, usize)>,
) -> Result<(), Phase10EvidenceContractError> {
    let Some(&(actions, checkpoints, observations)) = case_bounds.get(&binding.case_id) else {
        return Err(contract_error("leaf refers to an unknown case"));
    };
    if contains_private_pass(&binding.implementation)
        || contains_private_pass(&binding.observation_path)
        || contains_private_pass(&binding.policy_path)
    {
        return Err(contract_error("private solver pass identity is forbidden"));
    }
    validate_repository_reference(&binding.implementation, false)?;
    validate_repository_reference(&binding.tests.focused, true)?;
    validate_repository_reference(&binding.tests.integration, true)?;
    validate_repository_reference(&binding.tests.property, true)?;
    let tests = [
        binding.tests.focused.as_ref(),
        binding.tests.integration.as_ref(),
        binding.tests.property.as_ref(),
    ];
    if tests.into_iter().collect::<HashSet<_>>().len() != tests.len() {
        return Err(contract_error(
            "focused, integration, and property tests must be distinct",
        ));
    }
    validate_witness(
        &binding.control,
        WitnessRole::Control,
        actions,
        checkpoints,
        observations,
    )?;
    validate_witness(
        &binding.activation,
        WitnessRole::Activation,
        actions,
        checkpoints,
        observations,
    )?;
    if binding.control == binding.activation {
        return Err(contract_error("control and activation witnesses alias"));
    }
    if let Some(interaction) = &binding.maybe_interaction {
        validate_witness(
            interaction,
            WitnessRole::Interaction,
            actions,
            checkpoints,
            observations,
        )?;
        if interaction == &binding.control || interaction == &binding.activation {
            return Err(contract_error(
                "interaction witness aliases another proof role",
            ));
        }
    }
    if binding.observation_path.is_empty()
        || binding.observation_path.contains('*')
        || binding.observation_path.contains('?')
    {
        return Err(contract_error(
            "observation path is open or declaration-only",
        ));
    }
    if !PHASE10_REQUIRED_POLICY_PATHS.contains(&binding.policy_path.as_ref()) {
        return Err(contract_error(
            "leaf policy is absent from the closed registry",
        ));
    }
    validate_payloads(&binding.case_id, &binding.payloads)
}

fn validate_witness(
    witness: &Phase10EvidenceWitnessRef,
    expected_role: WitnessRole,
    actions: usize,
    checkpoints: usize,
    observations: usize,
) -> Result<(), Phase10EvidenceContractError> {
    if witness.role != expected_role {
        return Err(contract_error(
            "witness role differs from its typed binding",
        ));
    }
    if witness.action_index >= actions
        || witness.checkpoint_index >= checkpoints
        || witness.observation_index >= observations
    {
        return Err(contract_error("witness index is out of range"));
    }
    Ok(())
}

fn validate_repository_reference(
    reference: &str,
    test: bool,
) -> Result<(), Phase10EvidenceContractError> {
    if reference.is_empty()
        || reference.starts_with('/')
        || reference.contains("..")
        || reference.contains('\\')
        || reference.contains('*')
        || reference.contains('?')
    {
        return Err(contract_error("unsafe or open repository reference"));
    }
    let valid = if test {
        reference.contains("tests/") || reference.contains("#[test]")
    } else {
        reference.starts_with("crates/liquidfun/src/")
            || reference.starts_with("crates/liquidfun-differential/src/")
    };
    if !valid {
        return Err(contract_error(
            "declaration-only implementation or test reference",
        ));
    }
    Ok(())
}

fn validate_payloads(
    case_id: &ScenarioId,
    payloads: &Phase10EvidencePayloads,
) -> Result<(), Phase10EvidenceContractError> {
    let mut paths = HashSet::new();
    for (role, path) in payloads.entries() {
        let expected = format!("cases/{}/proofs/{role}.json", case_id.as_str());
        if path != expected || !paths.insert(path) {
            return Err(contract_error(
                "proof payload path is noncanonical or aliased",
            ));
        }
    }
    Ok(())
}

fn contains_private_pass(value: &str) -> bool {
    ["PassId", "pass_id", "pass_trace", "pass_inventory"]
        .iter()
        .any(|forbidden| value.contains(forbidden))
}

fn contract_error(reason: impl Into<Box<str>>) -> Phase10EvidenceContractError {
    Phase10EvidenceContractError {
        reason: reason.into(),
    }
}
