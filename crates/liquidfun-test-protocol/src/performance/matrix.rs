use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use super::{
    PerformanceError, PerformanceErrorKind, PerformancePolicy, PerformanceVersion,
    policy::render_json,
};
use crate::{ResolveRequest, resolve_catalog, reviewed_scenario_catalog};
use crate::{Sha256Hex, render_scenario_catalog_projection};

const FIXED_TIMESTEP_BITS: u32 = 0x3c88_8889;

/// Closed workload vocabulary required by the Phase 12 performance matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceWorkloadKind {
    /// Complete world-step overhead.
    WorldStep,
    /// Broad-phase proxy and pair processing.
    BroadPhase,
    /// Narrow-phase collision evaluation.
    NarrowPhase,
    /// Contact constraint solving.
    ContactSolve,
    /// Continuous collision detection.
    Ccd,
    /// Joint constraint solving.
    Joints,
    /// Particle creation, destruction, and pause lifecycle.
    ParticleLifecycle,
    /// Particle-particle and particle-body contact generation.
    ParticleContacts,
    /// Particle ordering and proxy sorting.
    ParticleSort,
    /// Particle pressure solving.
    ParticlePressure,
    /// Large particle-system stepping.
    LargeParticleSystem,
    /// Mixed rigid-body and particle stepping.
    MixedWorld,
    /// AABB query traversal.
    AabbQuery,
    /// Ray-cast traversal and callback control.
    RayCast,
}

impl PerformanceWorkloadKind {
    /// Every workload in stable review order.
    pub const ALL: [Self; 14] = [
        Self::WorldStep,
        Self::BroadPhase,
        Self::NarrowPhase,
        Self::ContactSolve,
        Self::Ccd,
        Self::Joints,
        Self::ParticleLifecycle,
        Self::ParticleContacts,
        Self::ParticleSort,
        Self::ParticlePressure,
        Self::LargeParticleSystem,
        Self::MixedWorld,
        Self::AabbQuery,
        Self::RayCast,
    ];

    /// Returns the stable workload token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorldStep => "world_step",
            Self::BroadPhase => "broad_phase",
            Self::NarrowPhase => "narrow_phase",
            Self::ContactSolve => "contact_solve",
            Self::Ccd => "ccd",
            Self::Joints => "joints",
            Self::ParticleLifecycle => "particle_lifecycle",
            Self::ParticleContacts => "particle_contacts",
            Self::ParticleSort => "particle_sort",
            Self::ParticlePressure => "particle_pressure",
            Self::LargeParticleSystem => "large_particle_system",
            Self::MixedWorld => "mixed_world",
            Self::AabbQuery => "aabb_query",
            Self::RayCast => "ray_cast",
        }
    }

    const fn is_scalable(self) -> bool {
        !matches!(
            self,
            Self::WorldStep | Self::NarrowPhase | Self::ContactSolve | Self::Ccd | Self::Joints
        )
    }
}

/// Reviewed complete-workload execution count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceSizePoint {
    /// Workload has a fixed semantic cardinality.
    Fixed,
    /// Repeat the bound workload 128 times per measured sample.
    WorkUnits128,
    /// Repeat the bound workload 1,024 times per measured sample.
    WorkUnits1024,
    /// Repeat the bound workload 8,192 times per measured sample.
    WorkUnits8192,
}

impl PerformanceSizePoint {
    const SWEEP: [Self; 3] = [Self::WorkUnits128, Self::WorkUnits1024, Self::WorkUnits8192];

    const fn as_id(self) -> &'static str {
        match self {
            Self::Fixed => "fixed",
            Self::WorkUnits128 => "128",
            Self::WorkUnits1024 => "1024",
            Self::WorkUnits8192 => "8192",
        }
    }

    /// Returns the exact number of complete workload executions measured per sample.
    #[must_use]
    pub const fn execution_units(self) -> u32 {
        match self {
            Self::Fixed => 1,
            Self::WorkUnits128 => 128,
            Self::WorkUnits1024 => 1_024,
            Self::WorkUnits8192 => 8_192,
        }
    }
}

/// Engine side producing a timing sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceEngineRole {
    /// Native Rust implementation.
    NativeRust,
    /// Pinned upstream C++ oracle.
    PinnedCppOracle,
}

/// Optimization mode permitted for authoritative scalar comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarOptimizationMode {
    /// Release optimization without native-CPU, SIMD, or fast-math tuning.
    ReleaseScalar,
}

/// Explicit lifecycle region around one measured benchmark action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasuredRegion {
    /// Allocate and resolve the benchmark scenario.
    Setup,
    /// Execute the excluded warm-up run.
    Warmup,
    /// Execute only the actions measured by wall clock.
    MeasuredActions,
    /// Release the benchmark world outside the measured region.
    Teardown,
}

/// Complete ordered setup/warm-up/measured/teardown boundary declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasuredRegions {
    ordered: [MeasuredRegion; 4],
}

impl MeasuredRegions {
    const COMPLETE: Self = Self {
        ordered: [
            MeasuredRegion::Setup,
            MeasuredRegion::Warmup,
            MeasuredRegion::MeasuredActions,
            MeasuredRegion::Teardown,
        ],
    };

    /// Reports whether every lifecycle boundary appears in reviewed order.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        matches!(
            self.ordered,
            [
                MeasuredRegion::Setup,
                MeasuredRegion::Warmup,
                MeasuredRegion::MeasuredActions,
                MeasuredRegion::Teardown
            ]
        )
    }
}

/// Exact solver and timestep settings bound into every workload case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerformanceRunSettings {
    timestep_bits: u32,
    velocity_iterations: u32,
    position_iterations: u32,
    particle_iterations: u32,
}

impl PerformanceRunSettings {
    const fn reviewed(particle_iterations: u32) -> Self {
        Self {
            timestep_bits: FIXED_TIMESTEP_BITS,
            velocity_iterations: 8,
            position_iterations: 3,
            particle_iterations,
        }
    }
}

/// Immutable resolved binding for one workload and size point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceCase {
    case_id: Box<str>,
    workload: PerformanceWorkloadKind,
    size_point: PerformanceSizePoint,
    scenario_id: Box<str>,
    catalog_sha256: Sha256Hex,
    resolved_sha256: Sha256Hex,
    execution_sha256: Sha256Hex,
    settings: PerformanceRunSettings,
    logical_horizon: u32,
    optimization_mode: ScalarOptimizationMode,
    engine_roles: [PerformanceEngineRole; 2],
    regions: MeasuredRegions,
}

impl PerformanceCase {
    /// Returns the workload kind.
    #[must_use]
    pub const fn workload(&self) -> PerformanceWorkloadKind {
        self.workload
    }

    /// Returns the workload cardinality point.
    #[must_use]
    pub const fn size_point(&self) -> PerformanceSizePoint {
        self.size_point
    }

    /// Returns the resolved scenario selected for this measured workload.
    #[must_use]
    pub const fn scenario_id(&self) -> &str {
        &self.scenario_id
    }

    /// Returns the catalog projection hash bound to this case.
    #[must_use]
    pub const fn catalog_sha256(&self) -> &Sha256Hex {
        &self.catalog_sha256
    }

    /// Returns the exact resolved-scenario byte hash.
    #[must_use]
    pub const fn resolved_sha256(&self) -> &Sha256Hex {
        &self.resolved_sha256
    }

    /// Returns the hash of the resolved bytes plus the measured workload driver parameters.
    #[must_use]
    pub const fn execution_sha256(&self) -> &Sha256Hex {
        &self.execution_sha256
    }

    /// Returns the fixed logical measurement horizon.
    #[must_use]
    pub const fn logical_horizon(&self) -> u32 {
        self.logical_horizon
    }

    /// Returns the reviewed scalar optimization mode.
    #[must_use]
    pub const fn optimization_mode(&self) -> ScalarOptimizationMode {
        self.optimization_mode
    }

    /// Returns the explicit measured-region boundary declaration.
    #[must_use]
    pub const fn regions(&self) -> MeasuredRegions {
        self.regions
    }
}

/// Complete reviewed workload matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceMatrix {
    version: PerformanceVersion,
    policy: PerformancePolicy,
    policy_sha256: Sha256Hex,
    catalog_sha256: Sha256Hex,
    cases: Box<[PerformanceCase]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPerformanceMatrix {
    version: PerformanceVersion,
    policy: PerformancePolicy,
    policy_sha256: Sha256Hex,
    catalog_sha256: Sha256Hex,
    cases: Vec<PerformanceCase>,
}

impl<'de> Deserialize<'de> for PerformanceMatrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawPerformanceMatrix::deserialize(deserializer)?;
        let matrix = Self::new(raw.cases).map_err(serde::de::Error::custom)?;
        if raw.version != matrix.version
            || raw.policy != matrix.policy
            || raw.policy_sha256 != matrix.policy_sha256
            || raw.catalog_sha256 != matrix.catalog_sha256
        {
            return Err(serde::de::Error::custom(
                "performance matrix identity does not match its typed cases",
            ));
        }
        Ok(matrix)
    }
}

impl PerformanceMatrix {
    /// Validates a complete workload matrix.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceError`] for duplicates, incomplete coverage, or
    /// inconsistent catalog bindings.
    pub fn new(cases: Vec<PerformanceCase>) -> Result<Self, PerformanceError> {
        let policy = PerformancePolicy::reviewed_v1();
        let policy_bytes = serde_json::to_vec(&policy).map_err(|_| encoding_error())?;
        let policy_sha256 = Sha256Hex::from_digest(Sha256::digest(policy_bytes).into());
        let catalog_bytes = render_scenario_catalog_projection()
            .map_err(|_| PerformanceError::new(PerformanceErrorKind::CatalogProjection))?;
        let catalog_sha256 = Sha256Hex::from_digest(Sha256::digest(catalog_bytes).into());
        validate_cases(&cases, &catalog_sha256)?;
        Ok(Self {
            version: PerformanceVersion,
            policy,
            policy_sha256,
            catalog_sha256,
            cases: cases.into_boxed_slice(),
        })
    }

    /// Builds the reviewed Phase 12 matrix from sealed typed rows.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceError`] if catalog projection or a sealed row drifts.
    pub fn reviewed_v1() -> Result<Self, PerformanceError> {
        let catalog_bytes = render_scenario_catalog_projection()
            .map_err(|_| PerformanceError::new(PerformanceErrorKind::CatalogProjection))?;
        let catalog_sha256 = Sha256Hex::from_digest(Sha256::digest(catalog_bytes).into());
        let mut cases = Vec::with_capacity(32);
        for workload in PerformanceWorkloadKind::ALL {
            if workload.is_scalable() {
                for size_point in PerformanceSizePoint::SWEEP {
                    cases.push(case_for(workload, size_point, catalog_sha256.clone())?);
                }
            } else {
                cases.push(case_for(
                    workload,
                    PerformanceSizePoint::Fixed,
                    catalog_sha256.clone(),
                )?);
            }
        }
        Self::new(cases)
    }

    /// Returns cases in stable workload then size-point order.
    #[must_use]
    pub fn cases(&self) -> &[PerformanceCase] {
        &self.cases
    }
}

/// Renders the byte-stable reviewed workload matrix.
///
/// # Errors
///
/// Returns [`PerformanceError`] if the catalog or canonical JSON encoding fails.
pub fn render_performance_matrix() -> Result<String, PerformanceError> {
    let matrix = PerformanceMatrix::reviewed_v1()?;
    let value = serde_json::to_value(matrix).map_err(|_| encoding_error())?;
    Ok(render_json(&value))
}

fn validate_cases(
    cases: &[PerformanceCase],
    catalog_sha256: &Sha256Hex,
) -> Result<(), PerformanceError> {
    let actual = cases
        .iter()
        .map(|case| (case.workload, case.size_point))
        .collect::<BTreeSet<_>>();
    if actual.len() != cases.len() {
        return Err(PerformanceError::new(
            PerformanceErrorKind::DuplicateCaseIdentity,
        ));
    }
    if actual != expected_identities() {
        return Err(PerformanceError::new(
            PerformanceErrorKind::IncompleteWorkloadMatrix,
        ));
    }
    for case in cases {
        let expected = case_for(case.workload, case.size_point, catalog_sha256.clone())?;
        if *case != expected {
            return Err(PerformanceError::new(
                PerformanceErrorKind::InvalidCaseBinding,
            ));
        }
    }
    if cases.iter().any(|case| {
        !case.regions.is_complete()
            || case.engine_roles
                != [
                    PerformanceEngineRole::NativeRust,
                    PerformanceEngineRole::PinnedCppOracle,
                ]
    }) {
        return Err(PerformanceError::new(
            PerformanceErrorKind::InvalidCaseBinding,
        ));
    }
    Ok(())
}

fn expected_identities() -> BTreeSet<(PerformanceWorkloadKind, PerformanceSizePoint)> {
    let mut expected = BTreeSet::new();
    for workload in PerformanceWorkloadKind::ALL {
        if workload.is_scalable() {
            for size_point in PerformanceSizePoint::SWEEP {
                expected.insert((workload, size_point));
            }
        } else {
            expected.insert((workload, PerformanceSizePoint::Fixed));
        }
    }
    expected
}

fn case_for(
    workload: PerformanceWorkloadKind,
    size_point: PerformanceSizePoint,
    catalog_sha256: Sha256Hex,
) -> Result<PerformanceCase, PerformanceError> {
    let binding = scenario_binding(workload);
    let catalog = reviewed_scenario_catalog()
        .map_err(|_| PerformanceError::new(PerformanceErrorKind::CatalogProjection))?;
    let definition = catalog
        .definitions()
        .iter()
        .find(|definition| definition.slug().as_str() == binding.scenario_id)
        .ok_or_else(|| PerformanceError::new(PerformanceErrorKind::InvalidCaseBinding))?;
    let metadata = definition
        .metadata()
        .ok_or_else(|| PerformanceError::new(PerformanceErrorKind::InvalidCaseBinding))?;
    let resolved = resolve_catalog(
        catalog.definitions(),
        &ResolveRequest::new(definition.slug().clone(), None, metadata.default_settings()),
    )
    .map_err(|_| PerformanceError::new(PerformanceErrorKind::InvalidCaseBinding))?;
    let logical_horizon = u32::try_from(resolved.checkpoints().len())
        .map_err(|_| PerformanceError::new(PerformanceErrorKind::InvalidCaseBinding))?;
    let resolved_sha256 = resolved.identity().content_sha256().clone();
    let mut execution_hasher = Sha256::new();
    execution_hasher.update(resolved_sha256.as_str().as_bytes());
    execution_hasher.update([0]);
    execution_hasher.update(workload.as_str().as_bytes());
    execution_hasher.update([0]);
    execution_hasher.update(size_point.as_id().as_bytes());
    execution_hasher.update([0]);
    execution_hasher.update(size_point.execution_units().to_le_bytes());
    Ok(PerformanceCase {
        case_id: format!("{}-{}", workload.as_str(), size_point.as_id()).into_boxed_str(),
        workload,
        size_point,
        scenario_id: binding.scenario_id.into(),
        catalog_sha256,
        resolved_sha256,
        execution_sha256: Sha256Hex::from_digest(execution_hasher.finalize().into()),
        settings: PerformanceRunSettings::reviewed(
            metadata.default_settings().particle_iterations(),
        ),
        logical_horizon,
        optimization_mode: ScalarOptimizationMode::ReleaseScalar,
        engine_roles: [
            PerformanceEngineRole::NativeRust,
            PerformanceEngineRole::PinnedCppOracle,
        ],
        regions: MeasuredRegions::COMPLETE,
    })
}

struct ScenarioBinding {
    scenario_id: &'static str,
}

const fn scenario_binding(workload: PerformanceWorkloadKind) -> ScenarioBinding {
    match workload {
        PerformanceWorkloadKind::WorldStep => binding("rigid-runtime-mutation"),
        PerformanceWorkloadKind::BroadPhase => binding("rigid-world-queries"),
        PerformanceWorkloadKind::NarrowPhase => binding("rigid-contact-lifecycle"),
        PerformanceWorkloadKind::ContactSolve => binding("rigid-stack-stability"),
        PerformanceWorkloadKind::Ccd => binding("rigid-continuous-collision"),
        PerformanceWorkloadKind::Joints => binding("joint-distance-behavior"),
        PerformanceWorkloadKind::ParticleLifecycle => binding("particle-system-pause-action"),
        PerformanceWorkloadKind::ParticleContacts => binding("particle-contacts-and-coupling"),
        PerformanceWorkloadKind::ParticleSort => binding("particle-mutations"),
        PerformanceWorkloadKind::ParticlePressure => binding("particle-flags-pressure-repulsive"),
        PerformanceWorkloadKind::LargeParticleSystem => {
            binding("particle-group-construction-append")
        }
        PerformanceWorkloadKind::MixedWorld => binding("particle-group-solid-rigid"),
        PerformanceWorkloadKind::AabbQuery => binding("particle-aabb-query-controls"),
        PerformanceWorkloadKind::RayCast => binding("particle-ray-callback-controls"),
    }
}

const fn binding(scenario_id: &'static str) -> ScenarioBinding {
    ScenarioBinding { scenario_id }
}

const fn encoding_error() -> PerformanceError {
    PerformanceError::new(PerformanceErrorKind::CanonicalEncoding)
}
