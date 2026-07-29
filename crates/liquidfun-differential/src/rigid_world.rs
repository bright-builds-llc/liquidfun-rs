//! Native execution for the closed Phase 6 rigid-world timelines.

mod evidence;
mod model;
mod native;
pub(crate) mod phase10;
mod phase7;
mod phase8;
mod phase9;
mod state;
#[cfg(test)]
mod tests;
pub use native::{NativeRigidWorldError, NativeRigidWorldExecutor};
pub use phase9::{
    PHASE9_REGISTRY_ID, PHASE9_REQUIRED_POLICY_PATHS, Phase9CaseEvidenceError,
    Phase9ComparatorError, Phase9ComparisonOutcome, Phase9CrossRunProof, Phase9CrossRunProofRecord,
    Phase9EvidenceBindingError, Phase9EvidenceMismatch, Phase9EvidencePayloadRef, Phase9Mismatch,
    Phase9ObservationComparison, Phase9PolicyKind, compare_phase9_particle_observations,
    compare_phase9_rigid_world_results, phase9_observation_is_declared, phase9_policy_for_path,
    validate_phase9_cross_run_proofs, validate_phase9_evidence_bindings,
    validate_phase9_policy_registry,
};
pub use phase10::{
    PHASE10_EVIDENCE_SCHEMA_VERSION, PHASE10_POLICY_REGISTRY, PHASE10_REQUIRED_POLICY_PATHS,
    Phase10ComparatorError, Phase10ComparisonMode, Phase10ComparisonOutcome,
    Phase10EvidenceBinding, Phase10EvidenceContractError, Phase10EvidenceLeaf,
    Phase10EvidencePayloads, Phase10EvidenceTestRefs, Phase10EvidenceWitnessRef, Phase10Mismatch,
    Phase10Policy, Phase10PolicyCalibration, Phase10PolicyKind, compare_phase10_observations,
    phase10_policy_calibrations, required_phase10_evidence_leaves,
    validate_phase10_evidence_contract, validate_phase10_policy_registry,
};

use evidence::{
    capture_checkpoint, collect_direct_transitions, collect_mutation_report, collect_step_report,
    observe_step, remove_destroyed_mapping,
};
use model::{
    action_error, body_created_witness, body_declaration, body_type, checked_u32,
    declaration_error, feature, filter_data, fixture_declaration, fixture_definition,
    native_body_mass_data, rigid_body_kind, rigid_filter, transform_bits, vec2, vec2_bits,
};
use state::Observation;
pub(crate) use state::TimelineExecutor;

use liquidfun::collision::shape::{CircleShape, PolygonShape};
use liquidfun::collision::{FeatureKind, FilterData, ManifoldKind, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyMassData, BodyType, DestroyedId, DestructionCause, DestructionRecord,
    FixtureDef, FixtureId, ManagedContactSnapshot, StepConfiguration, StepLimits, StepReport,
};
use liquidfun_test_protocol::{
    FloatBits, HarnessLimits, Phase6PolicyProfile, Phase7PolicyProfile, Phase8PolicyProfile,
    RIGID_WORLD_POSITION_ITERATIONS, RIGID_WORLD_TIMESTEP_BITS, RIGID_WORLD_VELOCITY_ITERATIONS,
    RecordLimit, RigidBodyDeclaration, RigidBodyKind, RigidBodySnapshot, RigidContactEvent,
    RigidContactEventKind, RigidContactFeature, RigidContactResult, RigidDestructionRecord,
    RigidExpectedCheckpoint, RigidExpectedCounts, RigidFeatureKind, RigidFilterBits,
    RigidFixtureDeclaration, RigidFixtureShape, RigidFixtureSnapshot, RigidManifoldKind,
    RigidManifoldPoint, RigidManifoldResult, RigidWorldAction, RigidWorldActionRecord,
    RigidWorldRequestRecord, RigidWorldResultRecord, RigidWorldTimeline, RigidWorldTimelineResult,
    RigidWorldWitness, RigidWorldWitnessFamily, ScenarioId, Sha256Hex, TransformBits, Vec2Bits,
    decode_rigid_world_request_jsonl, encode_jsonl, validate_rigid_world_result_against_request,
};
use sha2::{Digest, Sha256};

use crate::rigid_evidence::{RigidComparisonOutcome, compare_phase8_rigid_world_results};
use crate::supervisor::{OracleExecutable, RigidWorldProcessError, execute_rigid_world_process};

const PHASE6_POLICY: &str = include_str!("../../../protocol/tolerances/phase6-v1.toml");
const PHASE7_POLICY: &str = include_str!("../../../protocol/tolerances/phase7-v1.toml");
const PHASE8_POLICY: &str = include_str!("../../../protocol/tolerances/phase8-v1.toml");
const PHASE6_POLICY_CONTENT_SHA256: &str =
    "7f10df148852866fd20d11b8d27adcddc0ad463ac3d3d716a8946ca5c8f1c63a";
const PHASE7_POLICY_CONTENT_SHA256: &str =
    "fd772b2cf523a6d40bf978bc4d0da18a4564181a93e6b2bdeb8e4d40d5613311";
const PHASE8_POLICY_CONTENT_SHA256: &str =
    "2843ca40bec5b1c680135664c58c12a8388a7a9e86ad77f8ef5a268f3f15a6bf";

struct RetainedPolicyProfiles {
    phase6: Phase6PolicyProfile,
    phase7: Phase7PolicyProfile,
    phase8: Phase8PolicyProfile,
}

/// Non-physics failure while running one exact Phase 9 request through both engines.
#[derive(Debug, thiserror::Error)]
pub enum Phase9DifferentialError {
    /// Canonical request serialization or revalidation failed before execution.
    #[error("Phase 9 canonical request failed: {message}")]
    Request {
        /// Bounded protocol diagnostic.
        message: Box<str>,
    },
    /// Native Rust execution failed before a comparison outcome existed.
    #[error(transparent)]
    Native(#[from] NativeRigidWorldError),
    /// The process-isolated C++ role failed at the harness boundary.
    #[error(transparent)]
    Oracle(#[from] RigidWorldProcessError),
    /// The closed comparator rejected policy, structure, or numeric validity.
    #[error(transparent)]
    Comparator(#[from] Phase9ComparatorError),
}

/// One-request/two-engine Phase 9 differential result.
#[derive(Debug)]
pub struct Phase9DifferentialRun {
    request_sha256: Sha256Hex,
    native_request_sha256: Sha256Hex,
    oracle_request_sha256: Sha256Hex,
    consumed_paths: Box<[&'static str]>,
    outcome: Phase9ComparisonOutcome,
}

impl Phase9DifferentialRun {
    /// Returns the digest of the one canonical JSONL request.
    #[must_use]
    pub const fn request_sha256(&self) -> &Sha256Hex {
        &self.request_sha256
    }

    /// Returns the exact request digest consumed by the native role.
    #[must_use]
    pub const fn native_request_sha256(&self) -> &Sha256Hex {
        &self.native_request_sha256
    }

    /// Returns the exact request digest consumed by the C++ role.
    #[must_use]
    pub const fn oracle_request_sha256(&self) -> &Sha256Hex {
        &self.oracle_request_sha256
    }

    /// Returns every closed policy path in reviewed source order.
    #[must_use]
    pub fn consumed_paths(&self) -> &[&'static str] {
        &self.consumed_paths
    }

    /// Returns match or first physics-mismatch evidence.
    #[must_use]
    pub const fn outcome(&self) -> &Phase9ComparisonOutcome {
        &self.outcome
    }
}

/// Executes one canonical request through native Rust and the selected pinned process oracle.
///
/// The request is serialized and hashed once, decoded back through the bounded protocol, and
/// that one validated value is supplied to both roles. Physics disagreement is returned as a
/// normal comparison outcome; process, validation, and comparator failures remain typed harness
/// errors.
///
/// # Errors
///
/// Returns [`Phase9DifferentialError`] before a physics outcome exists when canonical request
/// preparation, either engine, or the fail-closed comparator boundary fails.
pub fn run_phase9_differential(
    executable: &OracleExecutable,
    request: &RigidWorldRequestRecord,
    expected_oracle_revision: &str,
) -> Result<Phase9DifferentialRun, Phase9DifferentialError> {
    let limits = HarnessLimits::phase2_default_v1();
    let request_bytes = encode_jsonl(request, &limits, RecordLimit::Input).map_err(|error| {
        Phase9DifferentialError::Request {
            message: error.to_string().into(),
        }
    })?;
    let canonical_request =
        decode_rigid_world_request_jsonl(&request_bytes, &limits).map_err(|error| {
            Phase9DifferentialError::Request {
                message: error.to_string().into(),
            }
        })?;
    let request_sha256 = Sha256Hex::from_digest(Sha256::digest(&request_bytes).into());
    let native = NativeRigidWorldExecutor::execute(&canonical_request)?;
    let captured =
        execute_rigid_world_process(executable, &canonical_request, expected_oracle_revision)?;
    let outcome = compare_complete_phase9_rigid_world_results(
        &canonical_request,
        &native,
        captured.result(),
    )?;
    let consumed_paths = validate_phase9_policy_registry(PHASE9_REQUIRED_POLICY_PATHS)?;
    Ok(Phase9DifferentialRun {
        native_request_sha256: request_sha256.clone(),
        oracle_request_sha256: request_sha256.clone(),
        request_sha256,
        consumed_paths,
        outcome,
    })
}

/// Compares the complete retained Phase 6 through Phase 8 surface before Phase 9 particles.
///
/// The retained profiles are embedded from the checked-in canonical policy files and verified
/// against their reviewed content digests before parsing. Callers cannot inject substitute
/// policies or bypass retained first-divergence ordering.
///
/// # Errors
///
/// Returns [`Phase9ComparatorError`] when an embedded policy fails its digest or parser contract,
/// the retained comparator rejects the request/result boundary, or the particle comparator finds
/// invalid structure or numeric state.
pub fn compare_complete_phase9_rigid_world_results(
    request: &RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
) -> Result<Phase9ComparisonOutcome, Phase9ComparatorError> {
    let profiles = canonical_retained_profiles()?;
    compare_complete_phase9_rigid_world_results_with_profiles(request, native, oracle, &profiles)
}

fn compare_complete_phase9_rigid_world_results_with_profiles(
    request: &RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
    profiles: &RetainedPolicyProfiles,
) -> Result<Phase9ComparisonOutcome, Phase9ComparatorError> {
    match compare_phase8_rigid_world_results(
        request,
        native,
        oracle,
        &profiles.phase6,
        &profiles.phase7,
        &profiles.phase8,
    ) {
        Err(failure) => Err(Phase9ComparatorError::RetainedRigid(failure)),
        Ok(RigidComparisonOutcome::PhysicsMismatch(report)) => {
            Ok(Phase9ComparisonOutcome::RetainedRigidMismatch(report))
        }
        Ok(RigidComparisonOutcome::Match) => {
            compare_phase9_rigid_world_results(request, native, oracle)
        }
    }
}

fn canonical_retained_profiles() -> Result<RetainedPolicyProfiles, Phase9ComparatorError> {
    verify_policy_content_digest("phase6-v1", PHASE6_POLICY, PHASE6_POLICY_CONTENT_SHA256)?;
    verify_policy_content_digest("phase7-v1", PHASE7_POLICY, PHASE7_POLICY_CONTENT_SHA256)?;
    verify_policy_content_digest("phase8-v1", PHASE8_POLICY, PHASE8_POLICY_CONTENT_SHA256)?;
    let phase6 = Phase6PolicyProfile::parse_toml(PHASE6_POLICY)
        .map_err(|error| retained_policy_error("phase6-v1", error))?;
    let phase7 = Phase7PolicyProfile::parse_toml(PHASE7_POLICY)
        .map_err(|error| retained_policy_error("phase7-v1", error))?;
    let phase8 = Phase8PolicyProfile::parse_toml(PHASE8_POLICY)
        .map_err(|error| retained_policy_error("phase8-v1", error))?;
    Ok(RetainedPolicyProfiles {
        phase6,
        phase7,
        phase8,
    })
}

fn verify_policy_content_digest(
    profile_id: &str,
    contents: &str,
    expected: &str,
) -> Result<(), Phase9ComparatorError> {
    let actual = Sha256Hex::from_digest(Sha256::digest(contents.as_bytes()).into());
    if actual.as_str() == expected {
        return Ok(());
    }
    Err(Phase9ComparatorError::PolicyRegistry {
        reason: format!(
            "embedded {profile_id} content digest mismatch: expected {expected}, found {}",
            actual.as_str()
        )
        .into(),
    })
}

fn retained_policy_error(profile_id: &str, error: impl std::fmt::Display) -> Phase9ComparatorError {
    Phase9ComparatorError::PolicyRegistry {
        reason: format!("embedded {profile_id} failed to parse: {error}").into(),
    }
}

/// Revalidates a native result against its complete request declaration.
///
/// # Errors
///
/// Returns [`NativeRigidWorldError::Declaration`] when identity, timeline, checkpoint, count,
/// or declaration order differs.
pub fn validate_native_rigid_world_result(
    request: &RigidWorldRequestRecord,
    result: &RigidWorldResultRecord,
) -> Result<(), NativeRigidWorldError> {
    validate_rigid_world_result_against_request(request, result).map_err(|error| {
        NativeRigidWorldError::Declaration {
            checkpoint_id: request.scenario().scenario_id().as_str().into(),
            message: error.to_string().into(),
        }
    })
}

fn execute_timeline(
    timeline: &RigidWorldTimeline,
) -> Result<RigidWorldTimelineResult, NativeRigidWorldError> {
    let timeline_id = timeline.actions().first().map_or_else(
        || "empty-timeline".into(),
        |action| action.action_id().as_str().into(),
    );
    catch_native_timeline_panic(timeline_id, || execute_timeline_inner(timeline))
}

fn catch_native_timeline_panic<T>(
    timeline_id: Box<str>,
    execute: impl FnOnce() -> Result<T, NativeRigidWorldError>,
) -> Result<T, NativeRigidWorldError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(execute))
        .map_err(|_payload| NativeRigidWorldError::Panic { timeline_id })?
}

fn execute_timeline_inner(
    timeline: &RigidWorldTimeline,
) -> Result<RigidWorldTimelineResult, NativeRigidWorldError> {
    let mut executor = TimelineExecutor::new(timeline.witness_family())?;
    let mut checkpoints = Vec::with_capacity(timeline.checkpoints().len());
    for action in timeline.actions() {
        execute_action(&mut executor, timeline, action)?;
        for expected in timeline
            .checkpoints()
            .iter()
            .filter(|checkpoint| checkpoint.after_action_id() == action.action_id())
        {
            checkpoints.push(capture_checkpoint(&mut executor, timeline, expected)?);
        }
    }
    if !executor.bodies.is_empty()
        || !executor.fixtures.is_empty()
        || !executor.joints.is_empty()
        || !executor.ropes.is_empty()
        || !executor.particle_systems.is_empty()
        || !executor.particles.is_empty()
        || !executor.phase10.is_empty()
        || executor.world.joint_count() != 0
        || executor.world.contact_count() != 0
    {
        return Err(NativeRigidWorldError::Reset {
            family: executor.family,
        });
    }
    Ok(RigidWorldTimelineResult {
        witness_family: timeline.witness_family(),
        checkpoints: checkpoints.into_boxed_slice(),
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "closed action dispatch remains auditable in one place"
)]
fn execute_action(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    if phase10::execute_action(executor, timeline, record)? {
        return Ok(());
    }
    if phase9::execute_action(executor, timeline, record)? {
        executor.phase10.retain_live(&executor.world);
        return Ok(());
    }
    if phase8::execute_action(executor, timeline, record)? {
        return Ok(());
    }
    if phase7::execute_action(executor, record)? {
        return Ok(());
    }
    match record.action() {
        RigidWorldAction::CreateBody { body_id } => {
            let declaration = body_declaration(timeline, body_id, record)?;
            let definition = BodyDef::new(
                body_type(declaration.body_kind()),
                vec2(declaration.transform().position),
                declaration.transform().angle_bits.to_f32(),
                declaration.active(),
            )
            .map_err(|error| action_error(record, error))?;
            let body = executor
                .world
                .create_body(&definition)
                .map_err(|error| action_error(record, error))?;
            executor.bodies.push((body_id.clone(), body));
            executor.push_observation(body_created_witness(declaration.body_kind()), None);
        }
        RigidWorldAction::CreateFixture { fixture_id } => {
            let declaration = fixture_declaration(timeline, fixture_id, record)?;
            let owner = executor.body(declaration.owner_body_id(), record)?;
            let definition =
                fixture_definition(declaration).map_err(|message| action_error(record, message))?;
            let fixture = executor
                .world
                .create_fixture(owner, &definition)
                .map_err(|error| action_error(record, error))?;
            executor.fixtures.push((fixture_id.clone(), fixture));
            executor.fixture_owners.push((fixture, owner));
            executor.push_observation(RigidWorldWitness::FixturesCreated, None);
        }
        RigidWorldAction::InspectBody { body_id } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .body_snapshot(body)
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::BodyInspected, None);
        }
        RigidWorldAction::InspectFixture { fixture_id } => {
            let fixture = executor.fixture(fixture_id, record)?;
            executor
                .world
                .fixture_snapshot(fixture)
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::FixtureInspected, None);
        }
        RigidWorldAction::SetBodyTransform { body_id, transform } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_transform(
                    body,
                    vec2(transform.position),
                    transform.angle_bits.to_f32(),
                )
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::BodyTransformChanged, None);
        }
        RigidWorldAction::SetBodyType { body_id, body_kind } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_type(body, body_type(*body_kind))
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::BodyTypeChanged, None);
        }
        RigidWorldAction::SetBodyActive { body_id, active } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_active(body, *active)
                .map_err(|error| action_error(record, error))?;
            collect_direct_transitions(executor)?;
            let witness = match (record.phase(), active) {
                ("deactivate", false) => Some(RigidWorldWitness::DeactivationDestroyedContact),
                (_, false) => Some(RigidWorldWitness::BodyDeactivated),
                ("reactivate", true) => None,
                (_, true) => Some(RigidWorldWitness::BodyReactivated),
            };
            if let Some(witness) = witness {
                executor.push_observation(witness, executor.maybe_last_contact.clone());
            }
        }
        RigidWorldAction::SetFixtureSensor { fixture_id, sensor } => {
            let fixture = executor.fixture(fixture_id, record)?;
            executor
                .world
                .set_fixture_sensor(fixture, *sensor)
                .map_err(|error| action_error(record, error))?;
            if executor.family == RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle {
                executor.push_observation(
                    if *sensor {
                        RigidWorldWitness::SensorEnabled
                    } else {
                        RigidWorldWitness::SensorDisabled
                    },
                    None,
                );
            }
        }
        RigidWorldAction::SetFixtureMaterial {
            fixture_id,
            friction_bits,
            restitution_bits,
        } => {
            let fixture = executor.fixture(fixture_id, record)?;
            executor
                .world
                .set_fixture_friction(fixture, friction_bits.to_f32())
                .and_then(|()| {
                    executor
                        .world
                        .set_fixture_restitution(fixture, restitution_bits.to_f32())
                })
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::MaterialChanged, None);
        }
        RigidWorldAction::SetFixtureFilter { fixture_id, filter } => {
            let fixture = executor.fixture(fixture_id, record)?;
            executor
                .world
                .set_fixture_filter(fixture, filter_data(*filter))
                .map_err(|error| action_error(record, error))?;
            if executor.family == RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle {
                executor.push_observation(RigidWorldWitness::FilterChanged, None);
            }
        }
        RigidWorldAction::SetFixtureDensity {
            fixture_id,
            density_bits,
        } => {
            let fixture = executor.fixture(fixture_id, record)?;
            executor
                .world
                .set_fixture_density(fixture, density_bits.to_f32())
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::DensityChangedWithoutMassReset, None);
        }
        RigidWorldAction::ResetMassData { body_id } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .reset_body_mass_data(body)
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::MassReset, None);
        }
        RigidWorldAction::SetCustomMassData {
            body_id,
            mass_bits,
            center,
            inertia_bits,
        } => {
            let body = executor.body(body_id, record)?;
            let data = native_body_mass_data(*mass_bits, *center, *inertia_bits)
                .map_err(|error| action_error(record, error))?;
            executor
                .world
                .set_body_mass_data(body, data)
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::CustomMassSet, None);
        }
        RigidWorldAction::Step {
            timestep_bits,
            velocity_iterations,
            position_iterations,
        } => {
            if timestep_bits.bits() != RIGID_WORLD_TIMESTEP_BITS
                || *velocity_iterations != RIGID_WORLD_VELOCITY_ITERATIONS
                || *position_iterations != RIGID_WORLD_POSITION_ITERATIONS
            {
                return Err(action_error(record, "unsupported Phase 6 step tuple"));
            }
            let configuration = StepConfiguration::new(
                timestep_bits.to_f32(),
                *velocity_iterations,
                *position_iterations,
            )
            .map_err(|error| action_error(record, error))?;
            let report = phase8::step(executor, configuration, StepLimits::default())
                .map_err(|error| action_error(record, error))?;
            phase9::collect_step_occurrences(executor, &report)?;
            collect_step_report(executor, &report)?;
            observe_step(executor, record.phase());
            phase8::collect_step_lifecycle(executor, &report)?;
        }
        RigidWorldAction::DestroyFixture { fixture_id } => {
            let fixture = executor.fixture(fixture_id, record)?;
            let record_result = executor
                .world
                .destroy_fixture(fixture)
                .map_err(|error| action_error(record, error))?;
            collect_mutation_report(executor, &record_result)?;
            phase8::collect_mutation_lifecycle(executor, record_result.lifecycle())?;
            executor
                .fixtures
                .retain(|(_, candidate)| *candidate != fixture);
            executor
                .fixture_owners
                .retain(|(candidate, _owner)| *candidate != fixture);
            let witness = if executor.family == RigidWorldWitnessFamily::SingleContactLifecycle {
                RigidWorldWitness::FixtureDestroyedContact
            } else {
                RigidWorldWitness::FixtureDestroyed
            };
            executor.push_observation(witness, executor.maybe_last_contact.clone());
        }
        RigidWorldAction::DestroyBody { body_id } => {
            let body = executor.body(body_id, record)?;
            let records = executor
                .world
                .destroy_body(body)
                .map_err(|error| action_error(record, error))?;
            collect_mutation_report(executor, &records)?;
            phase8::collect_mutation_lifecycle(executor, records.lifecycle())?;
            for destruction in &records {
                remove_destroyed_mapping(executor, destruction.destroyed());
                phase8::remove_destroyed_mapping(executor, destruction.destroyed());
            }
            let witness = if executor.family == RigidWorldWitnessFamily::SingleContactLifecycle {
                RigidWorldWitness::BodyCascadeEndOrdered
            } else {
                RigidWorldWitness::BodyDestroyed
            };
            executor.push_observation(witness, executor.maybe_last_contact.clone());
        }
        _ => {
            return Err(action_error(
                record,
                "unsupported rigid-world action reached the Phase 6 dispatcher",
            ));
        }
    }
    Ok(())
}
