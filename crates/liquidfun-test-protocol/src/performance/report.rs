use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use super::{PerformanceEngineRole, PerformanceError, PerformanceErrorKind};
use crate::{EvidenceTier, Sha256Hex};

const MAXIMUM_IDENTITY_FIELD_BYTES: usize = 512;
const MAXIMUM_RAW_MEASUREMENTS: usize = 250_000;

/// Hardware session bound into an immutable performance report identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardwareSession {
    session_id: Box<str>,
    cpu_model: Box<str>,
    logical_cores: u16,
    memory_bytes: u64,
    operating_system: Box<str>,
}

impl HardwareSession {
    /// Validates one benchmark hardware session.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceError`] for empty text, impossible core counts, or
    /// zero memory.
    pub fn new(
        session_id: impl Into<String>,
        cpu_model: impl Into<String>,
        logical_cores: u16,
        memory_bytes: u64,
        operating_system: impl Into<String>,
    ) -> Result<Self, PerformanceError> {
        let session_id = identity_text(session_id)?;
        let cpu_model = identity_text(cpu_model)?;
        let operating_system = identity_text(operating_system)?;
        if logical_cores == 0 || memory_bytes == 0 {
            return Err(PerformanceError::new(
                PerformanceErrorKind::InvalidIdentityField,
            ));
        }
        Ok(Self {
            session_id,
            cpu_model,
            logical_cores,
            memory_bytes,
            operating_system,
        })
    }

    const fn is_valid(&self) -> bool {
        !self.session_id.is_empty()
            && self.session_id.len() <= MAXIMUM_IDENTITY_FIELD_BYTES
            && !self.cpu_model.is_empty()
            && self.cpu_model.len() <= MAXIMUM_IDENTITY_FIELD_BYTES
            && self.logical_cores > 0
            && self.memory_bytes > 0
            && !self.operating_system.is_empty()
            && self.operating_system.len() <= MAXIMUM_IDENTITY_FIELD_BYTES
    }
}

/// Raw fields required to reproduce a performance report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformanceReportIdentityFields {
    scenario_id: String,
    rust_revision: String,
    oracle_revision: String,
    rust_compiler: String,
    rust_linker: String,
    oracle_compiler: String,
    oracle_linker: String,
    target: String,
    rust_compile_flags: String,
    rust_link_flags: String,
    oracle_compile_flags: String,
    oracle_link_flags: String,
    hardware_session: HardwareSession,
    policy_sha256: Sha256Hex,
    matrix_sha256: Sha256Hex,
    catalog_sha256: Sha256Hex,
    resolved_sha256: Sha256Hex,
}

impl PerformanceReportIdentityFields {
    /// Collects the complete immutable report identity before validation.
    #[allow(
        clippy::too_many_arguments,
        reason = "all seventeen reproduction fields are intentionally mandatory"
    )]
    pub fn new(
        scenario_id: impl Into<String>,
        rust_revision: impl Into<String>,
        oracle_revision: impl Into<String>,
        rust_compiler: impl Into<String>,
        rust_linker: impl Into<String>,
        oracle_compiler: impl Into<String>,
        oracle_linker: impl Into<String>,
        target: impl Into<String>,
        rust_compile_flags: impl Into<String>,
        rust_link_flags: impl Into<String>,
        oracle_compile_flags: impl Into<String>,
        oracle_link_flags: impl Into<String>,
        hardware_session: HardwareSession,
        policy_sha256: Sha256Hex,
        matrix_sha256: Sha256Hex,
        catalog_sha256: Sha256Hex,
        resolved_sha256: Sha256Hex,
    ) -> Self {
        Self {
            scenario_id: scenario_id.into(),
            rust_revision: rust_revision.into(),
            oracle_revision: oracle_revision.into(),
            rust_compiler: rust_compiler.into(),
            rust_linker: rust_linker.into(),
            oracle_compiler: oracle_compiler.into(),
            oracle_linker: oracle_linker.into(),
            target: target.into(),
            rust_compile_flags: rust_compile_flags.into(),
            rust_link_flags: rust_link_flags.into(),
            oracle_compile_flags: oracle_compile_flags.into(),
            oracle_link_flags: oracle_link_flags.into(),
            hardware_session,
            policy_sha256,
            matrix_sha256,
            catalog_sha256,
            resolved_sha256,
        }
    }
}

/// Immutable report identity binding source, tools, flags, hardware, and policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceReportIdentity {
    scenario_id: Box<str>,
    rust_revision: Box<str>,
    oracle_revision: Box<str>,
    rust_compiler: Box<str>,
    rust_linker: Box<str>,
    oracle_compiler: Box<str>,
    oracle_linker: Box<str>,
    target: Box<str>,
    rust_compile_flags: Box<str>,
    rust_link_flags: Box<str>,
    oracle_compile_flags: Box<str>,
    oracle_link_flags: Box<str>,
    hardware_session: HardwareSession,
    policy_sha256: Sha256Hex,
    matrix_sha256: Sha256Hex,
    catalog_sha256: Sha256Hex,
    resolved_sha256: Sha256Hex,
    identity_sha256: Sha256Hex,
}

impl PerformanceReportIdentity {
    /// Validates and hashes all report reproduction fields.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceError`] when required identity text is empty,
    /// oversized, or cannot be encoded canonically.
    pub fn new(fields: PerformanceReportIdentityFields) -> Result<Self, PerformanceError> {
        if !fields.hardware_session.is_valid() {
            return Err(PerformanceError::new(
                PerformanceErrorKind::InvalidIdentityField,
            ));
        }
        let mut identity = Self {
            scenario_id: identity_text(fields.scenario_id)?,
            rust_revision: identity_text(fields.rust_revision)?,
            oracle_revision: identity_text(fields.oracle_revision)?,
            rust_compiler: identity_text(fields.rust_compiler)?,
            rust_linker: identity_text(fields.rust_linker)?,
            oracle_compiler: identity_text(fields.oracle_compiler)?,
            oracle_linker: identity_text(fields.oracle_linker)?,
            target: identity_text(fields.target)?,
            rust_compile_flags: identity_text(fields.rust_compile_flags)?,
            rust_link_flags: identity_text(fields.rust_link_flags)?,
            oracle_compile_flags: identity_text(fields.oracle_compile_flags)?,
            oracle_link_flags: identity_text(fields.oracle_link_flags)?,
            hardware_session: fields.hardware_session,
            policy_sha256: fields.policy_sha256,
            matrix_sha256: fields.matrix_sha256,
            catalog_sha256: fields.catalog_sha256,
            resolved_sha256: fields.resolved_sha256,
            identity_sha256: Sha256Hex::from_digest([0; 32]),
        };
        identity.identity_sha256 = hash_identity(&identity)?;
        Ok(identity)
    }

    /// Returns SHA-256 over all immutable reproduction fields.
    #[must_use]
    pub const fn identity_sha256(&self) -> &Sha256Hex {
        &self.identity_sha256
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPerformanceReportIdentity {
    scenario_id: String,
    rust_revision: String,
    oracle_revision: String,
    rust_compiler: String,
    rust_linker: String,
    oracle_compiler: String,
    oracle_linker: String,
    target: String,
    rust_compile_flags: String,
    rust_link_flags: String,
    oracle_compile_flags: String,
    oracle_link_flags: String,
    hardware_session: HardwareSession,
    policy_sha256: Sha256Hex,
    matrix_sha256: Sha256Hex,
    catalog_sha256: Sha256Hex,
    resolved_sha256: Sha256Hex,
    identity_sha256: Sha256Hex,
}

impl<'de> Deserialize<'de> for PerformanceReportIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawPerformanceReportIdentity::deserialize(deserializer)?;
        let reported_hash = raw.identity_sha256;
        let identity = Self::new(PerformanceReportIdentityFields::new(
            raw.scenario_id,
            raw.rust_revision,
            raw.oracle_revision,
            raw.rust_compiler,
            raw.rust_linker,
            raw.oracle_compiler,
            raw.oracle_linker,
            raw.target,
            raw.rust_compile_flags,
            raw.rust_link_flags,
            raw.oracle_compile_flags,
            raw.oracle_link_flags,
            raw.hardware_session,
            raw.policy_sha256,
            raw.matrix_sha256,
            raw.catalog_sha256,
            raw.resolved_sha256,
        ))
        .map_err(serde::de::Error::custom)?;
        if identity.identity_sha256 != reported_hash {
            return Err(serde::de::Error::custom(
                "performance report identity hash mismatch",
            ));
        }
        Ok(identity)
    }
}

#[derive(Serialize)]
struct IdentityHashPayload<'a> {
    scenario_id: &'a str,
    rust_revision: &'a str,
    oracle_revision: &'a str,
    rust_compiler: &'a str,
    rust_linker: &'a str,
    oracle_compiler: &'a str,
    oracle_linker: &'a str,
    target: &'a str,
    rust_compile_flags: &'a str,
    rust_link_flags: &'a str,
    oracle_compile_flags: &'a str,
    oracle_link_flags: &'a str,
    hardware_session: &'a HardwareSession,
    policy_sha256: &'a Sha256Hex,
    matrix_sha256: &'a Sha256Hex,
    catalog_sha256: &'a Sha256Hex,
    resolved_sha256: &'a Sha256Hex,
}

fn hash_identity(identity: &PerformanceReportIdentity) -> Result<Sha256Hex, PerformanceError> {
    let payload = IdentityHashPayload {
        scenario_id: &identity.scenario_id,
        rust_revision: &identity.rust_revision,
        oracle_revision: &identity.oracle_revision,
        rust_compiler: &identity.rust_compiler,
        rust_linker: &identity.rust_linker,
        oracle_compiler: &identity.oracle_compiler,
        oracle_linker: &identity.oracle_linker,
        target: &identity.target,
        rust_compile_flags: &identity.rust_compile_flags,
        rust_link_flags: &identity.rust_link_flags,
        oracle_compile_flags: &identity.oracle_compile_flags,
        oracle_link_flags: &identity.oracle_link_flags,
        hardware_session: &identity.hardware_session,
        policy_sha256: &identity.policy_sha256,
        matrix_sha256: &identity.matrix_sha256,
        catalog_sha256: &identity.catalog_sha256,
        resolved_sha256: &identity.resolved_sha256,
    };
    let bytes = serde_json::to_vec(&payload)
        .map_err(|_| PerformanceError::new(PerformanceErrorKind::CanonicalEncoding))?;
    Ok(Sha256Hex::from_digest(Sha256::digest(bytes).into()))
}

/// Compatibility classification allowed on a performance report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    /// Byte-identical replay, without fixture promotion.
    D0Replay,
    /// Supported-platform physics compatibility established elsewhere.
    D2Supported,
    /// Diagnostic-only experimental compatibility.
    D3Exploratory,
}

impl TryFrom<EvidenceTier> for CompatibilityStatus {
    type Error = PerformanceError;

    fn try_from(value: EvidenceTier) -> Result<Self, Self::Error> {
        match value {
            EvidenceTier::D0Replay => Ok(Self::D0Replay),
            EvidenceTier::D1Canonical => Err(PerformanceError::new(
                PerformanceErrorKind::FixturePromotionForbidden,
            )),
            EvidenceTier::D2Supported => Ok(Self::D2Supported),
            EvidenceTier::D3Exploratory => Ok(Self::D3Exploratory),
        }
    }
}

/// One immutable unaggregated wall-clock sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPerformanceMeasurement {
    engine_role: PerformanceEngineRole,
    baseline_run: u8,
    sample_index: u16,
    elapsed_nanoseconds: u64,
}

impl RawPerformanceMeasurement {
    /// Creates one nonzero raw sample.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceError`] when an index or duration is zero.
    pub const fn new(
        engine_role: PerformanceEngineRole,
        baseline_run: u8,
        sample_index: u16,
        elapsed_nanoseconds: u64,
    ) -> Result<Self, PerformanceError> {
        if baseline_run == 0 || sample_index == 0 || elapsed_nanoseconds == 0 {
            return Err(PerformanceError::new(
                PerformanceErrorKind::InvalidMeasurement,
            ));
        }
        Ok(Self {
            engine_role,
            baseline_run,
            sample_index,
            elapsed_nanoseconds,
        })
    }

    const fn is_valid(self) -> bool {
        self.baseline_run > 0 && self.sample_index > 0 && self.elapsed_nanoseconds > 0
    }
}

/// Reviewed confidence interval over a relative timing delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceInterval {
    confidence_percent: u8,
    lower_basis_points: i32,
    estimate_basis_points: i32,
    upper_basis_points: i32,
    noise_floor_basis_points: u16,
}

impl PerformanceInterval {
    /// Creates an ordered 95% or stronger interval.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceError`] for weak confidence, inverted bounds, or an
    /// invalid noise floor.
    pub const fn new(
        confidence_percent: u8,
        lower_basis_points: i32,
        estimate_basis_points: i32,
        upper_basis_points: i32,
        noise_floor_basis_points: u16,
    ) -> Result<Self, PerformanceError> {
        if confidence_percent < 95
            || confidence_percent > 100
            || lower_basis_points > estimate_basis_points
            || estimate_basis_points > upper_basis_points
            || noise_floor_basis_points > 10_000
        {
            return Err(PerformanceError::new(
                PerformanceErrorKind::InvalidMeasurement,
            ));
        }
        Ok(Self {
            confidence_percent,
            lower_basis_points,
            estimate_basis_points,
            upper_basis_points,
            noise_floor_basis_points,
        })
    }

    const fn is_valid(self) -> bool {
        self.confidence_percent >= 95
            && self.confidence_percent <= 100
            && self.lower_basis_points <= self.estimate_basis_points
            && self.estimate_basis_points <= self.upper_basis_points
            && self.noise_floor_basis_points <= 10_000
    }
}

/// Complete immutable report retaining raw measurements and derived interval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceReport {
    identity: PerformanceReportIdentity,
    compatibility_status: CompatibilityStatus,
    raw_measurements: Box<[RawPerformanceMeasurement]>,
    interval: PerformanceInterval,
}

impl PerformanceReport {
    /// Validates a bounded non-empty performance report.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceError`] when raw evidence is empty or exceeds the
    /// reviewed resource limit.
    pub fn new(
        identity: PerformanceReportIdentity,
        compatibility_status: CompatibilityStatus,
        raw_measurements: Vec<RawPerformanceMeasurement>,
        interval: PerformanceInterval,
    ) -> Result<Self, PerformanceError> {
        if raw_measurements.is_empty()
            || raw_measurements.len() > MAXIMUM_RAW_MEASUREMENTS
            || raw_measurements
                .iter()
                .any(|measurement| !measurement.is_valid())
            || !interval.is_valid()
        {
            return Err(PerformanceError::new(
                PerformanceErrorKind::InvalidMeasurement,
            ));
        }
        Ok(Self {
            identity,
            compatibility_status,
            raw_measurements: raw_measurements.into_boxed_slice(),
            interval,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPerformanceReport {
    identity: PerformanceReportIdentity,
    compatibility_status: CompatibilityStatus,
    raw_measurements: Vec<RawPerformanceMeasurement>,
    interval: PerformanceInterval,
}

impl<'de> Deserialize<'de> for PerformanceReport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawPerformanceReport::deserialize(deserializer)?;
        Self::new(
            raw.identity,
            raw.compatibility_status,
            raw.raw_measurements,
            raw.interval,
        )
        .map_err(serde::de::Error::custom)
    }
}

fn identity_text(value: impl Into<String>) -> Result<Box<str>, PerformanceError> {
    let value = value.into();
    if value.is_empty() || value.len() > MAXIMUM_IDENTITY_FIELD_BYTES {
        return Err(PerformanceError::new(
            PerformanceErrorKind::InvalidIdentityField,
        ));
    }
    Ok(value.into_boxed_str())
}
