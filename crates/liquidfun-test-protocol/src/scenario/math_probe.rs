#![allow(
    missing_docs,
    reason = "closed private-harness wire variants are self-describing"
)]

use std::{collections::HashSet, fmt, marker::PhantomData};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    CodecError, FloatBits, HarnessLimits, ProtocolVersion, RecordLimit, RequestId, ScenarioId,
    ScenarioSchemaVersion, ScenarioSource, Sha256Hex, ToleranceProfileVersion, TraceSchemaVersion,
    codec::{BoundedString, BoundedVec, decode_jsonl},
};

const MAXIMUM_ID_BYTES: usize = 128;
const MAXIMUM_STRING_BYTES: usize = 4 * 1024;
const MAXIMUM_CASES: usize = 256;
const MAXIMUM_SCENARIO_STEPS: usize = 32;

/// Stable validation failures for the bounded Phase 4 math-probe contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathProbeErrorKind {
    NoCases,
    DuplicateCaseId,
    InvalidIdentifier,
    InvalidSource,
    OperationInputMismatch,
    PolicyPathMismatch,
    InvalidHorizon,
}

/// Strict decoding or semantic validation failure for one math-probe request.
#[derive(Debug, thiserror::Error)]
pub enum MathProbeDecodeError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error("math probe validation failed: {0:?}")]
    Validation(MathProbeErrorKind),
}

/// Closed pure operation identifiers accepted by native and reference adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MathProbeOperation {
    IsValid,
    Abs,
    Min,
    Max,
    Clamp,
    InvSqrt,
    VecLength,
    VecNormalize,
    Dot,
    Cross,
    Mat22Solve,
    Mat33Solve,
    Mat22Inverse,
    Mat33SymInverse,
    Rotation,
    Transform,
    SweepTransform,
    SweepAdvance,
    SweepNormalize,
    Cancellation,
    HalfwayRounding,
    Overflow,
    Underflow,
    FmaWitness,
}

/// Closed registry paths from the checked-in `phase4-v1` policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MathProbePolicyPath {
    #[serde(rename = "math.branch.is_valid")]
    MathBranchIsValid,
    #[serde(rename = "math.operation.abs")]
    MathOperationAbs,
    #[serde(rename = "math.operation.min")]
    MathOperationMin,
    #[serde(rename = "math.pass_through.max")]
    MathPassThroughMax,
    #[serde(rename = "math.operation.clamp")]
    MathOperationClamp,
    #[serde(rename = "math.operation.inv_sqrt")]
    MathOperationInvSqrt,
    #[serde(rename = "math.vector.length")]
    MathVectorLength,
    #[serde(rename = "math.vector.normalize")]
    MathVectorNormalize,
    #[serde(rename = "math.vector.dot")]
    MathVectorDot,
    #[serde(rename = "math.vector.cross")]
    MathVectorCross,
    #[serde(rename = "math.matrix22.solve")]
    MathMatrix22Solve,
    #[serde(rename = "math.matrix33.solve")]
    MathMatrix33Solve,
    #[serde(rename = "math.matrix22.inverse")]
    MathMatrix22Inverse,
    #[serde(rename = "math.matrix33.symmetric_inverse")]
    MathMatrix33SymmetricInverse,
    #[serde(rename = "math.rotation")]
    MathRotation,
    #[serde(rename = "math.transform.operation")]
    MathTransformOperation,
    #[serde(rename = "math.transform.steps_32")]
    MathTransformSteps32,
    #[serde(rename = "math.sweep.transform")]
    MathSweepTransform,
    #[serde(rename = "math.sweep.advance_steps_4")]
    MathSweepAdvanceSteps4,
    #[serde(rename = "math.sweep.normalize")]
    MathSweepNormalize,
    #[serde(rename = "math.arithmetic.cancellation")]
    MathArithmeticCancellation,
    #[serde(rename = "math.arithmetic.halfway_rounding")]
    MathArithmeticHalfwayRounding,
    #[serde(rename = "math.arithmetic.overflow")]
    MathArithmeticOverflow,
    #[serde(rename = "math.arithmetic.underflow")]
    MathArithmeticUnderflow,
    #[serde(rename = "math.arithmetic.fma_witness")]
    MathArithmeticFmaWitness,
}

impl MathProbePolicyPath {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MathBranchIsValid => "math.branch.is_valid",
            Self::MathOperationAbs => "math.operation.abs",
            Self::MathOperationMin => "math.operation.min",
            Self::MathPassThroughMax => "math.pass_through.max",
            Self::MathOperationClamp => "math.operation.clamp",
            Self::MathOperationInvSqrt => "math.operation.inv_sqrt",
            Self::MathVectorLength => "math.vector.length",
            Self::MathVectorNormalize => "math.vector.normalize",
            Self::MathVectorDot => "math.vector.dot",
            Self::MathVectorCross => "math.vector.cross",
            Self::MathMatrix22Solve => "math.matrix22.solve",
            Self::MathMatrix33Solve => "math.matrix33.solve",
            Self::MathMatrix22Inverse => "math.matrix22.inverse",
            Self::MathMatrix33SymmetricInverse => "math.matrix33.symmetric_inverse",
            Self::MathRotation => "math.rotation",
            Self::MathTransformOperation => "math.transform.operation",
            Self::MathTransformSteps32 => "math.transform.steps_32",
            Self::MathSweepTransform => "math.sweep.transform",
            Self::MathSweepAdvanceSteps4 => "math.sweep.advance_steps_4",
            Self::MathSweepNormalize => "math.sweep.normalize",
            Self::MathArithmeticCancellation => "math.arithmetic.cancellation",
            Self::MathArithmeticHalfwayRounding => "math.arithmetic.halfway_rounding",
            Self::MathArithmeticOverflow => "math.arithmetic.overflow",
            Self::MathArithmeticUnderflow => "math.arithmetic.underflow",
            Self::MathArithmeticFmaWitness => "math.arithmetic.fma_witness",
        }
    }
}

/// Fixed comparison horizon for one probe case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MathProbeHorizon {
    Operation,
    ScenarioSteps { steps: u32 },
}

impl MathProbeHorizon {
    #[must_use]
    pub const fn steps(self) -> u32 {
        match self {
            Self::Operation => 1,
            Self::ScenarioSteps { steps } => steps,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vec2Bits {
    pub x_bits: FloatBits,
    pub y_bits: FloatBits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vec3Bits {
    pub x_bits: FloatBits,
    pub y_bits: FloatBits,
    pub z_bits: FloatBits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mat22Bits {
    pub first: Vec2Bits,
    pub second: Vec2Bits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mat33Bits {
    pub first: Vec3Bits,
    pub second: Vec3Bits,
    pub third: Vec3Bits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformBits {
    pub position: Vec2Bits,
    pub angle_bits: FloatBits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SweepBits {
    pub local_center: Vec2Bits,
    pub initial_center: Vec2Bits,
    pub center: Vec2Bits,
    pub initial_angle_bits: FloatBits,
    pub angle_bits: FloatBits,
    pub initial_fraction_bits: FloatBits,
}

/// A sequence decoded with a fixed pre-allocation bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeSteps(Vec<FloatBits>);

impl ProbeSteps {
    #[must_use]
    pub fn as_slice(&self) -> &[FloatBits] {
        &self.0
    }
}

impl Serialize for ProbeSteps {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProbeSteps {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor(PhantomData<FloatBits>);

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ProbeSteps;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    formatter,
                    "one to {MAXIMUM_SCENARIO_STEPS} exact float-bit steps"
                )
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let capacity = sequence
                    .size_hint()
                    .unwrap_or(0)
                    .min(MAXIMUM_SCENARIO_STEPS);
                let mut values = Vec::with_capacity(capacity);
                while let Some(value) = sequence.next_element()? {
                    if values.len() == MAXIMUM_SCENARIO_STEPS {
                        return Err(serde::de::Error::custom(
                            "collection exceeds reviewed limit",
                        ));
                    }
                    values.push(value);
                }
                Ok(ProbeSteps(values))
            }
        }

        deserializer.deserialize_seq(Visitor(PhantomData))
    }
}

/// Closed structured operands for every math-probe operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MathProbeInput {
    Scalar {
        value_bits: FloatBits,
    },
    Binary {
        a_bits: FloatBits,
        b_bits: FloatBits,
    },
    Clamp {
        value_bits: FloatBits,
        low_bits: FloatBits,
        high_bits: FloatBits,
    },
    Vector2 {
        vector: Vec2Bits,
    },
    VectorPair {
        a: Vec2Bits,
        b: Vec2Bits,
    },
    Mat22Solve {
        matrix: Mat22Bits,
        right: Vec2Bits,
    },
    Mat33Solve {
        matrix: Mat33Bits,
        right: Vec3Bits,
    },
    Mat22 {
        matrix: Mat22Bits,
    },
    Mat33 {
        matrix: Mat33Bits,
    },
    Rotation {
        angle_bits: FloatBits,
    },
    Transform {
        left: TransformBits,
        right: TransformBits,
        point: Vec2Bits,
    },
    SweepTransform {
        sweep: SweepBits,
        fraction_bits: FloatBits,
    },
    SweepAdvance {
        sweep: SweepBits,
        fractions_bits: ProbeSteps,
    },
    Sweep {
        sweep: SweepBits,
    },
    Cancellation {
        large_bits: FloatBits,
        opposite_bits: FloatBits,
        tail_bits: FloatBits,
    },
    HalfwayRounding {
        even_bits: FloatBits,
        odd_bits: FloatBits,
        half_ulp_bits: FloatBits,
    },
    Scale {
        value_bits: FloatBits,
        factor_bits: FloatBits,
    },
    FmaWitness {
        a_bits: FloatBits,
        b_bits: FloatBits,
        c_bits: FloatBits,
    },
}

/// One validated stable operation and its exact operands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MathProbeCase {
    case_id: Box<str>,
    operation: MathProbeOperation,
    policy_path: MathProbePolicyPath,
    horizon: MathProbeHorizon,
    input: MathProbeInput,
}

impl MathProbeCase {
    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
    }
    #[must_use]
    pub const fn operation(&self) -> MathProbeOperation {
        self.operation
    }
    #[must_use]
    pub const fn policy_path(&self) -> MathProbePolicyPath {
        self.policy_path
    }
    #[must_use]
    pub const fn horizon(&self) -> MathProbeHorizon {
        self.horizon
    }
    #[must_use]
    pub const fn input(&self) -> &MathProbeInput {
        &self.input
    }
}

/// Deterministic checked-in Phase 4 corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MathProbeScenario {
    scenario_id: ScenarioId,
    source: ScenarioSource,
    cases: Vec<MathProbeCase>,
}

impl MathProbeScenario {
    #[must_use]
    pub const fn scenario_id(&self) -> &ScenarioId {
        &self.scenario_id
    }
    #[must_use]
    pub const fn source(&self) -> &ScenarioSource {
        &self.source
    }
    #[must_use]
    pub fn cases(&self) -> &[MathProbeCase] {
        &self.cases
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MathProbeRequestRecord {
    protocol_version: ProtocolVersion,
    record_kind: MathProbeRequestKind,
    request_id: RequestId,
    scenario_schema_version: ScenarioSchemaVersion,
    requested_trace_schema_version: TraceSchemaVersion,
    tolerance_profile_version: ToleranceProfileVersion,
    tolerance_profile_sha256: Sha256Hex,
    scenario: MathProbeScenario,
}

impl MathProbeRequestRecord {
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    #[must_use]
    pub const fn scenario(&self) -> &MathProbeScenario {
        &self.scenario
    }
    #[must_use]
    pub const fn tolerance_profile_sha256(&self) -> &Sha256Hex {
        &self.tolerance_profile_sha256
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MathProbeRequestKind {
    MathProbeRequest,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequest {
    protocol_version: ProtocolVersion,
    record_kind: MathProbeRequestKind,
    request_id: BoundedString<MAXIMUM_ID_BYTES>,
    scenario_schema_version: ScenarioSchemaVersion,
    requested_trace_schema_version: TraceSchemaVersion,
    tolerance_profile_version: ToleranceProfileVersion,
    tolerance_profile_sha256: Sha256Hex,
    scenario: RawScenario,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScenario {
    scenario_id: BoundedString<MAXIMUM_ID_BYTES>,
    source: RawSource,
    cases: BoundedVec<RawCase, MAXIMUM_CASES>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawSource {
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
struct RawCase {
    case_id: BoundedString<MAXIMUM_ID_BYTES>,
    operation: MathProbeOperation,
    policy_path: MathProbePolicyPath,
    horizon: MathProbeHorizon,
    input: MathProbeInput,
}

/// Decodes one newline-complete, strict, bounded math-probe request.
pub fn decode_math_probe_request_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<MathProbeRequestRecord, MathProbeDecodeError> {
    let raw = decode_jsonl::<RawRequest>(bytes, limits, RecordLimit::Input)?;
    validate_request(raw)
}

fn validate_request(raw: RawRequest) -> Result<MathProbeRequestRecord, MathProbeDecodeError> {
    let request_id = RequestId::new(raw.request_id.into_string())
        .map_err(|_| MathProbeDecodeError::Validation(MathProbeErrorKind::InvalidIdentifier))?;
    let scenario_id = ScenarioId::new(raw.scenario.scenario_id.into_string())
        .map_err(|_| MathProbeDecodeError::Validation(MathProbeErrorKind::InvalidIdentifier))?;
    let source = validate_source(raw.scenario.source)?;
    let raw_cases = raw.scenario.cases.into_vec();
    if raw_cases.is_empty() {
        return Err(MathProbeDecodeError::Validation(
            MathProbeErrorKind::NoCases,
        ));
    }
    let mut ids = HashSet::with_capacity(raw_cases.len());
    let mut cases = Vec::with_capacity(raw_cases.len());
    for raw_case in raw_cases {
        validate_case(&raw_case)?;
        let case_id = raw_case.case_id.into_string();
        ScenarioId::new(case_id.clone())
            .map_err(|_| MathProbeDecodeError::Validation(MathProbeErrorKind::InvalidIdentifier))?;
        if !ids.insert(case_id.clone()) {
            return Err(MathProbeDecodeError::Validation(
                MathProbeErrorKind::DuplicateCaseId,
            ));
        }
        cases.push(MathProbeCase {
            case_id: case_id.into_boxed_str(),
            operation: raw_case.operation,
            policy_path: raw_case.policy_path,
            horizon: raw_case.horizon,
            input: raw_case.input,
        });
    }
    Ok(MathProbeRequestRecord {
        protocol_version: raw.protocol_version,
        record_kind: raw.record_kind,
        request_id,
        scenario_schema_version: raw.scenario_schema_version,
        requested_trace_schema_version: raw.requested_trace_schema_version,
        tolerance_profile_version: raw.tolerance_profile_version,
        tolerance_profile_sha256: raw.tolerance_profile_sha256,
        scenario: MathProbeScenario {
            scenario_id,
            source,
            cases,
        },
    })
}

fn validate_source(raw: RawSource) -> Result<ScenarioSource, MathProbeDecodeError> {
    match raw {
        RawSource::Named { name } => {
            let name = name.into_string();
            if name.trim().is_empty() {
                return Err(MathProbeDecodeError::Validation(
                    MathProbeErrorKind::InvalidSource,
                ));
            }
            Ok(ScenarioSource::Named {
                name: name.into_boxed_str(),
            })
        }
        RawSource::Seeded {
            generator_id,
            generator_version,
            seed,
        } => {
            let generator_id = generator_id.into_string();
            if generator_id.trim().is_empty() || generator_version == 0 {
                return Err(MathProbeDecodeError::Validation(
                    MathProbeErrorKind::InvalidSource,
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

fn validate_case(raw: &RawCase) -> Result<(), MathProbeDecodeError> {
    let input_matches = matches!(
        (raw.operation, &raw.input),
        (
            MathProbeOperation::IsValid | MathProbeOperation::Abs | MathProbeOperation::InvSqrt,
            MathProbeInput::Scalar { .. }
        ) | (
            MathProbeOperation::Min | MathProbeOperation::Max,
            MathProbeInput::Binary { .. }
        ) | (MathProbeOperation::Clamp, MathProbeInput::Clamp { .. })
            | (
                MathProbeOperation::VecLength | MathProbeOperation::VecNormalize,
                MathProbeInput::Vector2 { .. }
            )
            | (
                MathProbeOperation::Dot | MathProbeOperation::Cross,
                MathProbeInput::VectorPair { .. }
            )
            | (
                MathProbeOperation::Mat22Solve,
                MathProbeInput::Mat22Solve { .. }
            )
            | (
                MathProbeOperation::Mat33Solve,
                MathProbeInput::Mat33Solve { .. }
            )
            | (
                MathProbeOperation::Mat22Inverse,
                MathProbeInput::Mat22 { .. }
            )
            | (
                MathProbeOperation::Mat33SymInverse,
                MathProbeInput::Mat33 { .. }
            )
            | (
                MathProbeOperation::Rotation,
                MathProbeInput::Rotation { .. }
            )
            | (
                MathProbeOperation::Transform,
                MathProbeInput::Transform { .. }
            )
            | (
                MathProbeOperation::SweepTransform,
                MathProbeInput::SweepTransform { .. }
            )
            | (
                MathProbeOperation::SweepAdvance,
                MathProbeInput::SweepAdvance { .. }
            )
            | (
                MathProbeOperation::SweepNormalize,
                MathProbeInput::Sweep { .. }
            )
            | (
                MathProbeOperation::Cancellation,
                MathProbeInput::Cancellation { .. }
            )
            | (
                MathProbeOperation::HalfwayRounding,
                MathProbeInput::HalfwayRounding { .. }
            )
            | (
                MathProbeOperation::Overflow | MathProbeOperation::Underflow,
                MathProbeInput::Scale { .. }
            )
            | (
                MathProbeOperation::FmaWitness,
                MathProbeInput::FmaWitness { .. }
            )
    );
    if !input_matches {
        return Err(MathProbeDecodeError::Validation(
            MathProbeErrorKind::OperationInputMismatch,
        ));
    }
    let expected_path = match (raw.operation, raw.horizon) {
        (MathProbeOperation::IsValid, _) => MathProbePolicyPath::MathBranchIsValid,
        (MathProbeOperation::Abs, _) => MathProbePolicyPath::MathOperationAbs,
        (MathProbeOperation::Min, _) => MathProbePolicyPath::MathOperationMin,
        (MathProbeOperation::Max, _) => MathProbePolicyPath::MathPassThroughMax,
        (MathProbeOperation::Clamp, _) => MathProbePolicyPath::MathOperationClamp,
        (MathProbeOperation::InvSqrt, _) => MathProbePolicyPath::MathOperationInvSqrt,
        (MathProbeOperation::VecLength, _) => MathProbePolicyPath::MathVectorLength,
        (MathProbeOperation::VecNormalize, _) => MathProbePolicyPath::MathVectorNormalize,
        (MathProbeOperation::Dot, _) => MathProbePolicyPath::MathVectorDot,
        (MathProbeOperation::Cross, _) => MathProbePolicyPath::MathVectorCross,
        (MathProbeOperation::Mat22Solve, _) => MathProbePolicyPath::MathMatrix22Solve,
        (MathProbeOperation::Mat33Solve, _) => MathProbePolicyPath::MathMatrix33Solve,
        (MathProbeOperation::Mat22Inverse, _) => MathProbePolicyPath::MathMatrix22Inverse,
        (MathProbeOperation::Mat33SymInverse, _) => {
            MathProbePolicyPath::MathMatrix33SymmetricInverse
        }
        (MathProbeOperation::Rotation, _) => MathProbePolicyPath::MathRotation,
        (MathProbeOperation::Transform, MathProbeHorizon::Operation) => {
            MathProbePolicyPath::MathTransformOperation
        }
        (MathProbeOperation::Transform, MathProbeHorizon::ScenarioSteps { steps: 32 }) => {
            MathProbePolicyPath::MathTransformSteps32
        }
        (MathProbeOperation::Transform, _) => {
            return Err(MathProbeDecodeError::Validation(
                MathProbeErrorKind::InvalidHorizon,
            ));
        }
        (MathProbeOperation::SweepTransform, _) => MathProbePolicyPath::MathSweepTransform,
        (MathProbeOperation::SweepAdvance, MathProbeHorizon::ScenarioSteps { steps: 4 }) => {
            MathProbePolicyPath::MathSweepAdvanceSteps4
        }
        (MathProbeOperation::SweepAdvance, _) => {
            return Err(MathProbeDecodeError::Validation(
                MathProbeErrorKind::InvalidHorizon,
            ));
        }
        (MathProbeOperation::SweepNormalize, _) => MathProbePolicyPath::MathSweepNormalize,
        (MathProbeOperation::Cancellation, _) => MathProbePolicyPath::MathArithmeticCancellation,
        (MathProbeOperation::HalfwayRounding, _) => {
            MathProbePolicyPath::MathArithmeticHalfwayRounding
        }
        (MathProbeOperation::Overflow, _) => MathProbePolicyPath::MathArithmeticOverflow,
        (MathProbeOperation::Underflow, _) => MathProbePolicyPath::MathArithmeticUnderflow,
        (MathProbeOperation::FmaWitness, _) => MathProbePolicyPath::MathArithmeticFmaWitness,
    };
    if raw.policy_path != expected_path {
        return Err(MathProbeDecodeError::Validation(
            MathProbeErrorKind::PolicyPathMismatch,
        ));
    }
    let steps = raw.horizon.steps();
    if steps == 0 || usize::try_from(steps).map_or(true, |value| value > MAXIMUM_SCENARIO_STEPS) {
        return Err(MathProbeDecodeError::Validation(
            MathProbeErrorKind::InvalidHorizon,
        ));
    }
    match (&raw.horizon, &raw.input) {
        (
            MathProbeHorizon::ScenarioSteps { steps },
            MathProbeInput::SweepAdvance { fractions_bits, .. },
        ) if usize::try_from(*steps).ok() == Some(fractions_bits.as_slice().len())
            && !fractions_bits.as_slice().is_empty() => {}
        (MathProbeHorizon::ScenarioSteps { .. }, MathProbeInput::Transform { .. }) => {}
        (MathProbeHorizon::Operation, MathProbeInput::SweepAdvance { fractions_bits, .. })
            if fractions_bits.as_slice().len() == 1 => {}
        (MathProbeHorizon::Operation, _) => {}
        _ => {
            return Err(MathProbeDecodeError::Validation(
                MathProbeErrorKind::InvalidHorizon,
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode_jsonl;

    const REQUEST: &[u8] =
        include_bytes!("../../../../protocol/fixtures/accepted/math-probe-request.jsonl");
    const SCENARIO: &[u8] = include_bytes!("../../../../scenarios/phase-04/math-probes.json");

    #[test]
    fn math_probe_scenario_is_byte_stable() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();
        let request = decode_math_probe_request_jsonl(REQUEST, &limits)
            .expect("checked-in request should decode");

        // Act
        let mut encoded =
            serde_json::to_vec(request.scenario()).expect("validated scenario should encode");
        encoded.push(b'\n');

        // Assert
        assert_eq!(encoded, SCENARIO);
        assert_eq!(
            encode_jsonl(&request, &limits, RecordLimit::Input).expect("request should encode"),
            REQUEST
        );
    }

    #[test]
    fn math_probe_codec_rejects_unknown_duplicate_oversize_and_malformed_input() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();
        let text = std::str::from_utf8(REQUEST).expect("fixture should be UTF-8");
        let unknown = text.replacen("\"is_valid\"", "\"run_function\"", 1);
        let duplicate = text.replacen(
            "\"case_id\":\"negative-zero-valid\"",
            "\"case_id\":\"positive-zero-valid\"",
            1,
        );
        let malformed = text.replacen("\"value_bits\":0", "\"value_bits\":-1", 1);
        let first_case = text
            .find("{\"case_id\"")
            .expect("fixture should contain a case");
        let last = text.rfind("]}}").expect("fixture should end with cases");
        let case = &text[first_case
            ..text[first_case..]
                .find("},{\"case_id\"")
                .expect("fixture should have two cases")
                + first_case];
        let oversized_cases = std::iter::repeat_n(case, MAXIMUM_CASES + 1)
            .collect::<Vec<_>>()
            .join(",");
        let oversized = format!(
            "{}{}{}",
            &text[..first_case],
            oversized_cases,
            &text[last..]
        );

        // Act
        let errors = [unknown, duplicate, malformed, oversized].map(|record| {
            decode_math_probe_request_jsonl(record.as_bytes(), &limits)
                .expect_err("invalid request should fail")
        });

        // Assert
        assert!(matches!(errors[0], MathProbeDecodeError::Codec(_)));
        assert!(matches!(
            errors[1],
            MathProbeDecodeError::Validation(MathProbeErrorKind::DuplicateCaseId)
        ));
        assert!(matches!(errors[2], MathProbeDecodeError::Codec(_)));
        assert!(matches!(errors[3], MathProbeDecodeError::Codec(_)));
    }
}
