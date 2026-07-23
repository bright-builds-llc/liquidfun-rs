use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, json};

use super::{PerformanceError, PerformanceErrorKind};

const PERFORMANCE_VERSION: &str = "phase12-performance-v1";
const MINIMUM_BASELINE_RUNS: u8 = 5;
const MAXIMUM_BASELINE_RUNS: u8 = 64;
const MINIMUM_CONFIDENCE_PERCENT: u8 = 95;
const MINIMUM_PRACTICAL_FLOOR_BASIS_POINTS: u16 = 300;
const MAXIMUM_BASIS_POINTS: u16 = 10_000;

/// Exact version of the reviewed Phase 12 performance contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PerformanceVersion;

impl PerformanceVersion {
    /// Returns the stable wire token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        PERFORMANCE_VERSION
    }
}

impl Serialize for PerformanceVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(PERFORMANCE_VERSION)
    }
}

impl<'de> Deserialize<'de> for PerformanceVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == PERFORMANCE_VERSION {
            return Ok(Self);
        }
        Err(serde::de::Error::custom(
            "unsupported performance contract version",
        ))
    }
}

/// Closed benchmark execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkRunOrder {
    /// Alternate native Rust and pinned C++ oracle runs within one session.
    InterleavedRustCpp,
}

/// Authority allowed to support regression decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimingAuthority {
    /// Unprofiled wall-clock samples collected under the reviewed policy.
    UnprofiledWallClock,
}

/// Validated sample and regression-decision policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PerformancePolicy {
    version: PerformanceVersion,
    baseline_runs: u8,
    confidence_percent: u8,
    practical_floor_basis_points: u16,
    warmup_runs: u8,
    samples_per_engine: u16,
    run_order: BenchmarkRunOrder,
    timing_authority: TimingAuthority,
}

impl PerformancePolicy {
    /// Creates the reviewed policy with caller-supplied statistical bounds.
    ///
    /// # Errors
    ///
    /// Returns [`PerformanceError`] when a bound is weaker than the reviewed
    /// minimum or exceeds its resource limit.
    pub const fn new(
        baseline_runs: u8,
        confidence_percent: u8,
        practical_floor_basis_points: u16,
    ) -> Result<Self, PerformanceError> {
        if baseline_runs < MINIMUM_BASELINE_RUNS {
            return Err(PerformanceError::new(
                PerformanceErrorKind::BaselineRunsBelowMinimum,
            ));
        }
        if baseline_runs > MAXIMUM_BASELINE_RUNS {
            return Err(PerformanceError::new(
                PerformanceErrorKind::BaselineRunsAboveMaximum,
            ));
        }
        if confidence_percent < MINIMUM_CONFIDENCE_PERCENT {
            return Err(PerformanceError::new(
                PerformanceErrorKind::ConfidenceBelowMinimum,
            ));
        }
        if confidence_percent > 100 {
            return Err(PerformanceError::new(
                PerformanceErrorKind::ConfidenceAboveMaximum,
            ));
        }
        if practical_floor_basis_points < MINIMUM_PRACTICAL_FLOOR_BASIS_POINTS {
            return Err(PerformanceError::new(
                PerformanceErrorKind::PracticalFloorBelowMinimum,
            ));
        }
        if practical_floor_basis_points > MAXIMUM_BASIS_POINTS {
            return Err(PerformanceError::new(
                PerformanceErrorKind::PercentageAboveMaximum,
            ));
        }
        Ok(Self {
            version: PerformanceVersion,
            baseline_runs,
            confidence_percent,
            practical_floor_basis_points,
            warmup_runs: 1,
            samples_per_engine: 30,
            run_order: BenchmarkRunOrder::InterleavedRustCpp,
            timing_authority: TimingAuthority::UnprofiledWallClock,
        })
    }

    /// Returns the exact reviewed Phase 12 policy.
    #[must_use]
    pub const fn reviewed_v1() -> Self {
        Self {
            version: PerformanceVersion,
            baseline_runs: MINIMUM_BASELINE_RUNS,
            confidence_percent: MINIMUM_CONFIDENCE_PERCENT,
            practical_floor_basis_points: MINIMUM_PRACTICAL_FLOOR_BASIS_POINTS,
            warmup_runs: 1,
            samples_per_engine: 30,
            run_order: BenchmarkRunOrder::InterleavedRustCpp,
            timing_authority: TimingAuthority::UnprofiledWallClock,
        }
    }

    /// Returns the closed contract version.
    #[must_use]
    pub const fn version(&self) -> PerformanceVersion {
        self.version
    }

    /// Returns the independent baseline-run count.
    #[must_use]
    pub const fn baseline_runs(&self) -> u8 {
        self.baseline_runs
    }

    /// Returns the confidence level in whole percent.
    #[must_use]
    pub const fn confidence_percent(&self) -> u8 {
        self.confidence_percent
    }

    /// Returns the practical regression floor in basis points.
    #[must_use]
    pub const fn practical_floor_basis_points(&self) -> u16 {
        self.practical_floor_basis_points
    }

    /// Returns the warm-up count excluded from measurements.
    #[must_use]
    pub const fn warmup_runs(&self) -> u8 {
        self.warmup_runs
    }

    /// Returns the bounded samples collected per engine and run.
    #[must_use]
    pub const fn samples_per_engine(&self) -> u16 {
        self.samples_per_engine
    }

    /// Returns the only authoritative timing source.
    #[must_use]
    pub const fn timing_authority(&self) -> TimingAuthority {
        self.timing_authority
    }

    /// Reports whether the required Rust/C++ run order is interleaved.
    #[must_use]
    pub const fn is_interleaved(&self) -> bool {
        matches!(self.run_order, BenchmarkRunOrder::InterleavedRustCpp)
    }

    /// Applies the reviewed `max(3%, noise floor)` decision rule.
    #[must_use]
    pub const fn regression_threshold_basis_points(&self, noise_floor: u16) -> u16 {
        if noise_floor > self.practical_floor_basis_points {
            noise_floor
        } else {
            self.practical_floor_basis_points
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPerformancePolicy {
    version: PerformanceVersion,
    baseline_runs: u8,
    confidence_percent: u8,
    practical_floor_basis_points: u16,
    warmup_runs: u8,
    samples_per_engine: u16,
    run_order: BenchmarkRunOrder,
    timing_authority: TimingAuthority,
}

impl<'de> Deserialize<'de> for PerformancePolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawPerformancePolicy::deserialize(deserializer)?;
        let policy = Self::new(
            raw.baseline_runs,
            raw.confidence_percent,
            raw.practical_floor_basis_points,
        )
        .map_err(serde::de::Error::custom)?;
        if raw.version != policy.version
            || raw.warmup_runs != policy.warmup_runs
            || raw.samples_per_engine != policy.samples_per_engine
            || raw.run_order != policy.run_order
            || raw.timing_authority != policy.timing_authority
        {
            return Err(serde::de::Error::custom(
                "performance policy differs from reviewed execution bounds",
            ));
        }
        Ok(policy)
    }
}

/// Renders the byte-stable JSON Schema for the performance policy.
#[must_use]
pub fn render_performance_policy_schema() -> String {
    let document = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://liquidfun-rs.invalid/schemas/performance-policy-v1.schema.json",
        "title": "liquidfun-rs Phase 12 performance policy",
        "description": "Closed statistical policy for unprofiled wall-clock performance evidence. Timing never promotes semantic physics fixtures.",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "version": { "const": PERFORMANCE_VERSION },
            "baseline_runs": { "type": "integer", "minimum": 5, "maximum": 64 },
            "confidence_percent": { "type": "integer", "minimum": 95, "maximum": 100 },
            "practical_floor_basis_points": { "type": "integer", "minimum": 300, "maximum": 10000 },
            "warmup_runs": { "const": 1 },
            "samples_per_engine": { "const": 30 },
            "run_order": { "const": "interleaved_rust_cpp" },
            "timing_authority": { "const": "unprofiled_wall_clock" }
        },
        "required": [
            "version",
            "baseline_runs",
            "confidence_percent",
            "practical_floor_basis_points",
            "warmup_runs",
            "samples_per_engine",
            "run_order",
            "timing_authority"
        ],
        "examples": [PerformancePolicy::reviewed_v1()]
    });
    render_json(&document)
}

pub(super) fn render_json(value: &Value) -> String {
    let mut rendered = serde_json::to_string_pretty(value)
        .expect("JSON values always serialize deterministically");
    rendered.push('\n');
    rendered
}
