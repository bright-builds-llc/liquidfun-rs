use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    CheckpointId, CodecError, CodecErrorKind, FloatBits, HarnessLimits, ProtocolVersion,
    RecordLimit, RequestId, ScenarioId, ScenarioSchemaVersion, Sha256Hex, ToleranceProfileVersion,
    TraceSchemaVersion,
    codec::{BoundedString, BoundedVec, decode_jsonl},
};

const MAXIMUM_ID_BYTES: usize = 128;
const MAXIMUM_STRING_BYTES: usize = 4 * 1024;
const MAXIMUM_ENTITIES: usize = 4_096;
const MAXIMUM_COMMANDS: usize = 4_096;
const MAXIMUM_CHECKPOINTS: usize = 4_096;
const MAXIMUM_OBSERVABLES: usize = 128;
const MAXIMUM_SOLVER_ITERATIONS: u32 = 255;

mod reduction;
pub use reduction::*;

/// Stable semantic validation categories for a phase-2 scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioErrorKind {
    /// Phase 2 deliberately accepts no entity definitions.
    EntityDefinitionsNotSupported,
    /// At least one step command is required.
    NoCommands,
    /// Two commands use the same semantic identity.
    DuplicateCommandId,
    /// Two checkpoints use the same semantic identity.
    DuplicateCheckpointId,
    /// A checkpoint names a command that does not exist.
    UnknownCommandReference,
    /// Checkpoints do not follow command-boundary order.
    CheckpointOrderViolation,
    /// One checkpoint requests the same observable more than once.
    DuplicateObservable,
    /// A solver iteration count is zero.
    ZeroSolverIterations,
    /// A solver iteration count exceeds the reviewed phase-2 bound.
    SolverIterationsExceeded,
    /// A source identifier or generator version is invalid.
    InvalidSource,
    /// Combined scenario item counts exceed the reviewed aggregate bound.
    AggregateLimitExceeded,
    /// A typed identifier is malformed.
    InvalidIdentifier,
}

/// Error returned while decoding or validating one scenario request record.
#[derive(Debug, thiserror::Error)]
pub enum ScenarioDecodeError {
    /// Strict JSONL framing or typed decoding failed.
    #[error(transparent)]
    Codec(#[from] CodecError),
    /// Cross-field scenario validation failed.
    #[error("scenario validation failed: {0:?}")]
    Validation(ScenarioErrorKind),
}

impl ScenarioDecodeError {
    /// Returns the semantic failure category, when decoding reached scenario validation.
    #[must_use]
    pub const fn scenario_kind(&self) -> Option<ScenarioErrorKind> {
        match self {
            Self::Codec(_) => None,
            Self::Validation(kind) => Some(*kind),
        }
    }

    /// Reports whether a closed boundary object contained an unknown field.
    #[must_use]
    pub fn is_unknown_field(&self) -> bool {
        matches!(self, Self::Codec(error) if error.kind() == CodecErrorKind::UnknownField)
    }

    /// Reports whether a boundary object repeated a member.
    #[must_use]
    pub fn is_duplicate_member(&self) -> bool {
        matches!(self, Self::Codec(error) if error.kind() == CodecErrorKind::DuplicateMember)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
/// Validated semantic identity of one ordered scenario command.
pub struct CommandId(ScenarioId);

impl CommandId {
    fn new(value: String) -> Result<Self, ScenarioErrorKind> {
        ScenarioId::new(value)
            .map(Self)
            .map_err(|_| ScenarioErrorKind::InvalidIdentifier)
    }

    /// Returns the validated command identity text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Reproducible origin of a phase-2 semantic scenario.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScenarioSource {
    /// A checked-in or otherwise stable named scenario.
    Named {
        /// Stable human-reviewable scenario name.
        name: Box<str>,
    },
    /// A fully identified deterministic generator invocation.
    Seeded {
        /// Stable generator identity.
        generator_id: Box<str>,
        /// Version of the generator algorithm.
        generator_version: u32,
        /// Exact generator seed.
        seed: u64,
    },
}

/// A single bounded phase-2 world-step command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StepCommand {
    kind: StepCommandKind,
    command_id: CommandId,
    timestep_bits: FloatBits,
    velocity_iterations: u32,
    position_iterations: u32,
    particle_iterations: u32,
}

impl StepCommand {
    /// Returns the semantic command identity.
    #[must_use]
    pub const fn command_id(&self) -> &CommandId {
        &self.command_id
    }

    /// Returns the exact authoritative timestep bits.
    #[must_use]
    pub const fn timestep_bits(&self) -> FloatBits {
        self.timestep_bits
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StepCommandKind {
    Step,
}

/// Closed semantic observable requests supported by scenario schema 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestedObservable {
    /// Exact empty-world object and particle counts.
    WorldCounts,
    /// Exact accumulated simulation-time bits.
    SimulationTime,
}

/// A uniquely identified checkpoint at one ordered command boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckpointRequest {
    checkpoint_id: CheckpointId,
    after_command_id: CommandId,
    phase: Box<str>,
    observables: Vec<RequestedObservable>,
}

impl CheckpointRequest {
    /// Returns the checkpoint identity.
    #[must_use]
    pub const fn checkpoint_id(&self) -> &CheckpointId {
        &self.checkpoint_id
    }

    /// Returns the referenced command boundary.
    #[must_use]
    pub const fn after_command_id(&self) -> &CommandId {
        &self.after_command_id
    }

    /// Returns the stable phase label.
    #[must_use]
    pub fn phase(&self) -> &str {
        &self.phase
    }

    /// Returns the closed ordered observable requests.
    #[must_use]
    pub fn observables(&self) -> &[RequestedObservable] {
        &self.observables
    }
}

/// Validated engine-neutral scenario schema 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidatedScenarioV1 {
    scenario_id: ScenarioId,
    source: ScenarioSource,
    gravity_x_bits: FloatBits,
    gravity_y_bits: FloatBits,
    entities: Vec<NeverEntityDefinition>,
    commands: Vec<StepCommand>,
    checkpoints: Vec<CheckpointRequest>,
}

impl ValidatedScenarioV1 {
    /// Returns the stable scenario identity.
    #[must_use]
    pub const fn scenario_id(&self) -> &ScenarioId {
        &self.scenario_id
    }

    /// Returns the named or seeded reproducibility source.
    #[must_use]
    pub const fn source(&self) -> &ScenarioSource {
        &self.source
    }

    /// Returns the exact horizontal gravity bits.
    #[must_use]
    pub const fn gravity_x_bits(&self) -> FloatBits {
        self.gravity_x_bits
    }

    /// Returns the exact vertical gravity bits.
    #[must_use]
    pub const fn gravity_y_bits(&self) -> FloatBits {
        self.gravity_y_bits
    }

    /// Returns commands in solver-significant order.
    #[must_use]
    pub fn commands(&self) -> &[StepCommand] {
        &self.commands
    }

    /// Returns checkpoint requests in protocol order.
    #[must_use]
    pub fn checkpoints(&self) -> &[CheckpointRequest] {
        &self.checkpoints
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NeverEntityDefinition {}

/// A fully validated versioned scenario request record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScenarioRequestRecord {
    protocol_version: ProtocolVersion,
    record_kind: ScenarioRequestKind,
    request_id: RequestId,
    scenario_schema_version: ScenarioSchemaVersion,
    requested_trace_schema_version: TraceSchemaVersion,
    tolerance_profile_version: ToleranceProfileVersion,
    tolerance_profile_sha256: Sha256Hex,
    scenario: ValidatedScenarioV1,
}

impl ScenarioRequestRecord {
    /// Returns the transport protocol version.
    #[must_use]
    pub const fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    /// Returns the request identity.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Returns the validated scenario.
    #[must_use]
    pub const fn scenario(&self) -> &ValidatedScenarioV1 {
        &self.scenario
    }

    /// Returns the validated scenario schema version.
    #[must_use]
    pub const fn scenario_schema_version(&self) -> ScenarioSchemaVersion {
        self.scenario_schema_version
    }

    /// Returns the requested trace schema.
    #[must_use]
    pub const fn requested_trace_schema_version(&self) -> TraceSchemaVersion {
        self.requested_trace_schema_version
    }

    /// Returns the selected tolerance profile identity.
    #[must_use]
    pub const fn tolerance_profile_sha256(&self) -> &Sha256Hex {
        &self.tolerance_profile_sha256
    }

    /// Returns the selected tolerance policy version.
    #[must_use]
    pub const fn tolerance_profile_version(&self) -> ToleranceProfileVersion {
        self.tolerance_profile_version
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ScenarioRequestKind {
    ScenarioRequest,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScenarioRequestRecord {
    protocol_version: ProtocolVersion,
    record_kind: ScenarioRequestKind,
    request_id: BoundedString<MAXIMUM_ID_BYTES>,
    scenario_schema_version: ScenarioSchemaVersion,
    requested_trace_schema_version: TraceSchemaVersion,
    tolerance_profile_version: ToleranceProfileVersion,
    tolerance_profile_sha256: Sha256Hex,
    scenario: RawScenarioV1,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScenarioV1 {
    scenario_id: BoundedString<MAXIMUM_ID_BYTES>,
    source: RawScenarioSource,
    gravity_x_bits: FloatBits,
    gravity_y_bits: FloatBits,
    entities: BoundedVec<RawEntityDefinition, MAXIMUM_ENTITIES>,
    commands: BoundedVec<RawStepCommand, MAXIMUM_COMMANDS>,
    checkpoints: BoundedVec<RawCheckpointRequest, MAXIMUM_CHECKPOINTS>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawScenarioSource {
    Named {
        name: BoundedString<MAXIMUM_STRING_BYTES>,
    },
    Seeded {
        generator_id: BoundedString<MAXIMUM_STRING_BYTES>,
        generator_version: u32,
        seed: u64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntityDefinition {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStepCommand {
    kind: StepCommandKind,
    command_id: BoundedString<MAXIMUM_ID_BYTES>,
    timestep_bits: FloatBits,
    velocity_iterations: u32,
    position_iterations: u32,
    particle_iterations: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCheckpointRequest {
    checkpoint_id: BoundedString<MAXIMUM_ID_BYTES>,
    after_command_id: BoundedString<MAXIMUM_ID_BYTES>,
    phase: BoundedString<MAXIMUM_STRING_BYTES>,
    observables: BoundedVec<RequestedObservable, MAXIMUM_OBSERVABLES>,
}

/// Decodes and validates one strict phase-2 scenario request record.
///
/// # Errors
///
/// Returns [`ScenarioDecodeError`] unless framing, shape, versions, limits, IDs, references,
/// ordering, and the phase-2 empty-entity restriction all validate.
pub fn decode_scenario_request_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<ScenarioRequestRecord, ScenarioDecodeError> {
    let raw = decode_jsonl::<RawScenarioRequestRecord>(bytes, limits, RecordLimit::Input)?;
    validate_request(raw)
}

fn validate_request(
    raw: RawScenarioRequestRecord,
) -> Result<ScenarioRequestRecord, ScenarioDecodeError> {
    let request_id = RequestId::new(raw.request_id.into_string())
        .map_err(|_| ScenarioDecodeError::Validation(ScenarioErrorKind::InvalidIdentifier))?;
    let scenario = validate_scenario(raw.scenario)?;
    Ok(ScenarioRequestRecord {
        protocol_version: raw.protocol_version,
        record_kind: raw.record_kind,
        request_id,
        scenario_schema_version: raw.scenario_schema_version,
        requested_trace_schema_version: raw.requested_trace_schema_version,
        tolerance_profile_version: raw.tolerance_profile_version,
        tolerance_profile_sha256: raw.tolerance_profile_sha256,
        scenario,
    })
}

fn validate_scenario(raw: RawScenarioV1) -> Result<ValidatedScenarioV1, ScenarioDecodeError> {
    let scenario_id = ScenarioId::new(raw.scenario_id.into_string())
        .map_err(|_| ScenarioDecodeError::Validation(ScenarioErrorKind::InvalidIdentifier))?;
    let source = validate_source(raw.source)?;
    if !raw.entities.into_vec().is_empty() {
        return Err(ScenarioDecodeError::Validation(
            ScenarioErrorKind::EntityDefinitionsNotSupported,
        ));
    }
    let raw_commands = raw.commands.into_vec();
    if raw_commands.is_empty() {
        return Err(ScenarioDecodeError::Validation(
            ScenarioErrorKind::NoCommands,
        ));
    }
    let mut command_ids = HashSet::with_capacity(raw_commands.len());
    let mut commands = Vec::with_capacity(raw_commands.len());
    for raw_command in raw_commands {
        let command = validate_command(raw_command)?;
        if !command_ids.insert(command.command_id.as_str().to_owned()) {
            return Err(ScenarioDecodeError::Validation(
                ScenarioErrorKind::DuplicateCommandId,
            ));
        }
        commands.push(command);
    }

    let raw_checkpoints = raw.checkpoints.into_vec();
    validate_aggregate_counts(commands.len(), &raw_checkpoints)?;
    let mut checkpoint_ids = HashSet::with_capacity(raw_checkpoints.len());
    let mut previous_command_index = 0_usize;
    let mut checkpoints = Vec::with_capacity(raw_checkpoints.len());
    for raw_checkpoint in raw_checkpoints {
        let checkpoint = validate_checkpoint(raw_checkpoint)?;
        if !checkpoint_ids.insert(checkpoint.checkpoint_id.as_str().to_owned()) {
            return Err(ScenarioDecodeError::Validation(
                ScenarioErrorKind::DuplicateCheckpointId,
            ));
        }
        let maybe_command_index = commands.iter().position(|command| {
            command.command_id.as_str() == checkpoint.after_command_id.as_str()
        });
        let Some(command_index) = maybe_command_index else {
            return Err(ScenarioDecodeError::Validation(
                ScenarioErrorKind::UnknownCommandReference,
            ));
        };
        if !checkpoints.is_empty() && command_index < previous_command_index {
            return Err(ScenarioDecodeError::Validation(
                ScenarioErrorKind::CheckpointOrderViolation,
            ));
        }
        previous_command_index = command_index;
        checkpoints.push(checkpoint);
    }

    Ok(ValidatedScenarioV1 {
        scenario_id,
        source,
        gravity_x_bits: raw.gravity_x_bits,
        gravity_y_bits: raw.gravity_y_bits,
        entities: Vec::new(),
        commands,
        checkpoints,
    })
}

fn validate_source(raw: RawScenarioSource) -> Result<ScenarioSource, ScenarioDecodeError> {
    match raw {
        RawScenarioSource::Named { name } => {
            let name = name.into_string();
            if name.trim().is_empty() {
                return Err(ScenarioDecodeError::Validation(
                    ScenarioErrorKind::InvalidSource,
                ));
            }
            Ok(ScenarioSource::Named {
                name: name.into_boxed_str(),
            })
        }
        RawScenarioSource::Seeded {
            generator_id,
            generator_version,
            seed,
        } => {
            let generator_id = generator_id.into_string();
            if generator_id.trim().is_empty() || generator_version == 0 {
                return Err(ScenarioDecodeError::Validation(
                    ScenarioErrorKind::InvalidSource,
                ));
            }
            Ok(ScenarioSource::Seeded {
                generator_id: generator_id.into_boxed_str(),
                generator_version,
                seed,
            })
        }
    }
}

fn validate_command(raw: RawStepCommand) -> Result<StepCommand, ScenarioDecodeError> {
    for iterations in [
        raw.velocity_iterations,
        raw.position_iterations,
        raw.particle_iterations,
    ] {
        if iterations == 0 {
            return Err(ScenarioDecodeError::Validation(
                ScenarioErrorKind::ZeroSolverIterations,
            ));
        }
        if iterations > MAXIMUM_SOLVER_ITERATIONS {
            return Err(ScenarioDecodeError::Validation(
                ScenarioErrorKind::SolverIterationsExceeded,
            ));
        }
    }
    Ok(StepCommand {
        kind: raw.kind,
        command_id: CommandId::new(raw.command_id.into_string())
            .map_err(ScenarioDecodeError::Validation)?,
        timestep_bits: raw.timestep_bits,
        velocity_iterations: raw.velocity_iterations,
        position_iterations: raw.position_iterations,
        particle_iterations: raw.particle_iterations,
    })
}

fn validate_checkpoint(
    raw: RawCheckpointRequest,
) -> Result<CheckpointRequest, ScenarioDecodeError> {
    let checkpoint_id = CheckpointId::new(raw.checkpoint_id.into_string())
        .map_err(|_| ScenarioDecodeError::Validation(ScenarioErrorKind::InvalidIdentifier))?;
    let after_command_id = CommandId::new(raw.after_command_id.into_string())
        .map_err(ScenarioDecodeError::Validation)?;
    let observables = raw.observables.into_vec();
    let unique: HashSet<_> = observables.iter().copied().collect();
    if unique.len() != observables.len() {
        return Err(ScenarioDecodeError::Validation(
            ScenarioErrorKind::DuplicateObservable,
        ));
    }
    Ok(CheckpointRequest {
        checkpoint_id,
        after_command_id,
        phase: raw.phase.into_string().into_boxed_str(),
        observables,
    })
}

fn validate_aggregate_counts(
    command_count: usize,
    checkpoints: &[RawCheckpointRequest],
) -> Result<(), ScenarioDecodeError> {
    let observable_count = checkpoints.iter().try_fold(0_usize, |count, checkpoint| {
        count.checked_add(checkpoint.observables.len())
    });
    let maybe_total = observable_count
        .and_then(|count| count.checked_add(command_count))
        .and_then(|count| count.checked_add(checkpoints.len()));
    let maximum =
        MAXIMUM_COMMANDS + MAXIMUM_CHECKPOINTS + MAXIMUM_CHECKPOINTS * MAXIMUM_OBSERVABLES;
    if maybe_total.is_none_or(|total| total > maximum) {
        return Err(ScenarioDecodeError::Validation(
            ScenarioErrorKind::AggregateLimitExceeded,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
