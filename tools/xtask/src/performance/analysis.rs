//! Pure statistical and optimization-admission rules for Phase 12 evidence.

use std::collections::{BTreeMap, BTreeSet};

use liquidfun_test_protocol::performance::{PerformanceMatrix, PerformanceSizePoint};
use serde::{Deserialize, Serialize};

const PRACTICAL_FLOOR_BASIS_POINTS: u16 = 300;
const MINIMUM_PROFILE_BASIS_POINTS: u16 = 1_000;
const STUDENT_T_95_FOUR_DEGREES_FREEDOM: f64 = 2.776_445_105_197_798_7;

/// Closed bottleneck evidence accepted when a profile share is below 10%.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BottleneckKind {
    /// Allocation traffic or allocator contention.
    Allocation,
    /// Cache locality or cache-miss behavior.
    Cache,
    /// Workload scaling complexity.
    Scaling,
}

/// One 95% confidence interval over a signed relative timing delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    clippy::struct_field_names,
    reason = "the unit suffix is explicit in the evidence wire contract"
)]
pub(crate) struct WorkloadInterval {
    pub(crate) lower_basis_points: i32,
    pub(crate) estimate_basis_points: i32,
    pub(crate) upper_basis_points: i32,
    pub(crate) noise_floor_basis_points: u16,
}

impl WorkloadInterval {
    pub(crate) const fn is_ordered(self) -> bool {
        self.lower_basis_points <= self.estimate_basis_points
            && self.estimate_basis_points <= self.upper_basis_points
            && self.noise_floor_basis_points <= 10_000
    }

    pub(crate) const fn threshold_basis_points(self) -> u16 {
        if self.noise_floor_basis_points > PRACTICAL_FLOOR_BASIS_POINTS {
            self.noise_floor_basis_points
        } else {
            PRACTICAL_FLOOR_BASIS_POINTS
        }
    }
}

/// Required correctness evidence hashes for an optimization claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CorrectnessHashes {
    pub(crate) differential: Option<String>,
    pub(crate) determinism: Option<String>,
    pub(crate) safety: Option<String>,
    pub(crate) public_api: Option<String>,
}

impl CorrectnessHashes {
    fn is_complete(&self) -> bool {
        [
            self.differential.as_deref(),
            self.determinism.as_deref(),
            self.safety.as_deref(),
            self.public_api.as_deref(),
        ]
        .into_iter()
        .all(|maybe_hash| maybe_hash.is_some_and(valid_hash_binding))
    }
}

/// Build and timing authority bound to every optimization measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DisallowedBuildMode {
    Simd,
    Parallel,
    FastMath,
    UnsafeCode,
    ProfiledTotals,
}

/// Closed build configuration bound to every optimization measurement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OptimizationBuild {
    pub(crate) scalar_release: bool,
    pub(crate) disallowed_modes: BTreeSet<DisallowedBuildMode>,
}

impl OptimizationBuild {
    fn is_reviewed(&self) -> bool {
        self.scalar_release && self.disallowed_modes.is_empty()
    }
}

/// Immutable inputs to the fail-closed optimization decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OptimizationCandidate {
    pub(crate) before_commit: String,
    pub(crate) after_commit: String,
    pub(crate) build: OptimizationBuild,
    pub(crate) profile_basis_points: u16,
    pub(crate) maybe_bottleneck: Option<BottleneckKind>,
    pub(crate) workloads: BTreeMap<String, WorkloadInterval>,
    pub(crate) correctness: CorrectnessHashes,
}

/// Closed disposition for an optimization candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OptimizationDecision {
    /// Every reviewed admission rule passed.
    Admit,
    /// Source revisions are empty, malformed, or identical.
    RejectCommitIdentity,
    /// The build was not the reviewed scalar release configuration.
    RejectOptimizationMode,
    /// Neither a 10% profile share nor typed bottleneck evidence was present.
    RejectProfileOrBottleneck,
    /// A mandatory correctness evidence hash was absent.
    RejectCorrectnessGate,
    /// The mandatory workload matrix was incomplete or contained invalid intervals.
    RejectWorkloadCoverage,
    /// At least one mandatory workload regressed beyond its calibrated threshold.
    RejectWorkloadRegression,
    /// No workload's 95% interval cleared its calibrated practical threshold.
    RejectImprovementInterval,
}

/// Applies the reviewed optimization-admission policy without I/O.
#[must_use]
pub(crate) fn evaluate_optimization(candidate: &OptimizationCandidate) -> OptimizationDecision {
    if !valid_commit(&candidate.before_commit)
        || !valid_commit(&candidate.after_commit)
        || candidate.before_commit == candidate.after_commit
    {
        return OptimizationDecision::RejectCommitIdentity;
    }
    if !candidate.build.is_reviewed() {
        return OptimizationDecision::RejectOptimizationMode;
    }
    if candidate.profile_basis_points < MINIMUM_PROFILE_BASIS_POINTS
        && candidate.maybe_bottleneck.is_none()
    {
        return OptimizationDecision::RejectProfileOrBottleneck;
    }
    if !candidate.correctness.is_complete() {
        return OptimizationDecision::RejectCorrectnessGate;
    }
    let Some(expected_cases) = expected_case_ids() else {
        return OptimizationDecision::RejectWorkloadCoverage;
    };
    if candidate.workloads.len() != expected_cases.len()
        || expected_cases
            .iter()
            .any(|case_id| !candidate.workloads.contains_key(case_id))
        || candidate
            .workloads
            .values()
            .any(|interval| !interval.is_ordered())
    {
        return OptimizationDecision::RejectWorkloadCoverage;
    }
    if candidate
        .workloads
        .values()
        .any(|interval| interval.upper_basis_points < -i32::from(interval.threshold_basis_points()))
    {
        return OptimizationDecision::RejectWorkloadRegression;
    }
    if !candidate
        .workloads
        .values()
        .any(|interval| interval.lower_basis_points > i32::from(interval.threshold_basis_points()))
    {
        return OptimizationDecision::RejectImprovementInterval;
    }
    OptimizationDecision::Admit
}

fn expected_case_ids() -> Option<Vec<String>> {
    let matrix = PerformanceMatrix::reviewed_v1().ok()?;
    Some(
        matrix
            .cases()
            .iter()
            .map(|case| {
                format!(
                    "{}-{}",
                    case.workload().as_str(),
                    size_point_id(case.size_point())
                )
            })
            .collect(),
    )
}

const fn size_point_id(size: PerformanceSizePoint) -> &'static str {
    match size {
        PerformanceSizePoint::Fixed => "fixed",
        PerformanceSizePoint::WorkUnits128 => "128",
        PerformanceSizePoint::WorkUnits1024 => "1024",
        PerformanceSizePoint::WorkUnits8192 => "8192",
    }
}

/// Computes a conservative two-sided Student 95% interval over five independent runs.
pub(crate) fn student_95_interval(
    run_deltas_basis_points: &[i32],
) -> Result<WorkloadInterval, &'static str> {
    if run_deltas_basis_points.len() != 5 {
        return Err("calibration requires exactly five independent runs");
    }
    let count = 5.0;
    let mean = run_deltas_basis_points
        .iter()
        .map(|value| f64::from(*value))
        .sum::<f64>()
        / count;
    let squared_deviation = run_deltas_basis_points
        .iter()
        .map(|value| {
            let deviation = f64::from(*value) - mean;
            deviation * deviation
        })
        .sum::<f64>();
    let sample_standard_deviation = (squared_deviation / (count - 1.0)).sqrt();
    let margin = STUDENT_T_95_FOUR_DEGREES_FREEDOM * sample_standard_deviation / count.sqrt();
    let noise_floor = bounded_noise_floor(margin);
    Ok(WorkloadInterval {
        lower_basis_points: bounded_basis_points((mean - margin).floor()),
        estimate_basis_points: bounded_basis_points(mean.round()),
        upper_basis_points: bounded_basis_points((mean + margin).ceil()),
        noise_floor_basis_points: noise_floor,
    })
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "the value is explicitly clamped to the destination evidence range"
)]
fn bounded_basis_points(value: f64) -> i32 {
    value.clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the nonnegative value is explicitly clamped to 0..=10000"
)]
fn bounded_noise_floor(value: f64) -> u16 {
    value.ceil().clamp(0.0, 10_000.0) as u16
}

fn valid_commit(value: &str) -> bool {
    (7..=64).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_hash_binding(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::{WorkloadInterval, student_95_interval};

    #[test]
    fn calibration_retains_five_runs_and_uses_student_interval() {
        // Arrange
        let deltas = [310, 320, 300, 315, 305];

        // Act
        let interval = student_95_interval(&deltas).expect("five runs are valid");

        // Assert
        assert!(interval.lower_basis_points < interval.estimate_basis_points);
        assert!(interval.estimate_basis_points < interval.upper_basis_points);
        assert!(interval.noise_floor_basis_points > 0);
    }

    #[test]
    fn calibration_rejects_any_run_count_other_than_five() {
        // Arrange
        let deltas = [300, 301, 302, 303];

        // Act
        let result = student_95_interval(&deltas);

        // Assert
        assert_eq!(
            result,
            Err("calibration requires exactly five independent runs")
        );
    }

    #[test]
    fn threshold_uses_noise_floor_when_it_dominates_three_percent() {
        // Arrange
        let interval = WorkloadInterval {
            lower_basis_points: 0,
            estimate_basis_points: 0,
            upper_basis_points: 0,
            noise_floor_basis_points: 451,
        };

        // Act
        let threshold = interval.threshold_basis_points();

        // Assert
        assert_eq!(threshold, 451);
    }
}
