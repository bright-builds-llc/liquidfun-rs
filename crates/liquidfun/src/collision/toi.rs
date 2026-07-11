//! Checked time-of-impact operations over immutable shape-child proxies.
//!
//! Public results contain only a closed state and bounded time. Source branch,
//! support, separation, root-method, and termination evidence remains private.

mod separation;

use std::fmt;

use crate::collision::distance::{DistanceCache, distance};
use crate::collision::shape::Shape;
use crate::collision::{ChildIndex, CollisionError};
use crate::math::settings::{LINEAR_SLOP, MAX_POLYGON_VERTICES};
use crate::math::{Sweep, abs, max};

use separation::{SeparationFunction, SeparationIndices, SeparationKind, ToiProxy};

const MAX_OUTER_ITERATIONS: usize = 20;
const MAX_ROOT_ITERATIONS: usize = 50;

/// Checked immutable inputs for one time-of-impact query.
#[derive(Clone, Copy)]
pub struct TimeOfImpactInput<'a> {
    shape_a: &'a Shape,
    child_a: ChildIndex,
    sweep_a: Sweep,
    shape_b: &'a Shape,
    child_b: ChildIndex,
    sweep_b: Sweep,
    t_max: f32,
}

impl fmt::Debug for TimeOfImpactInput<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TimeOfImpactInput")
            .field("shape_a", self.shape_a)
            .field("sweep_a", &self.sweep_a)
            .field("shape_b", self.shape_b)
            .field("sweep_b", &self.sweep_b)
            .field("t_max", &self.t_max)
            .finish_non_exhaustive()
    }
}

impl<'a> TimeOfImpactInput<'a> {
    /// Creates a query from two checked shape children, copied sweeps, and an
    /// inclusive maximum time fraction.
    ///
    /// # Errors
    ///
    /// Returns a typed child-selection error, [`CollisionError::NonFiniteValue`]
    /// for a non-finite `t_max`, or [`CollisionError::FractionOutOfRange`] when
    /// `t_max` is outside `0.0..=1.0`.
    #[allow(clippy::too_many_arguments)] // Mirrors the two complete shape-sweep inputs.
    pub fn new(
        shape_a: &'a Shape,
        child_a: ChildIndex,
        sweep_a: Sweep,
        shape_b: &'a Shape,
        child_b: ChildIndex,
        sweep_b: Sweep,
        t_max: f32,
    ) -> Result<Self, CollisionError> {
        ChildIndex::new(child_a.get(), shape_a.child_count())?;
        ChildIndex::new(child_b.get(), shape_b.child_count())?;
        if !t_max.is_finite() {
            return Err(CollisionError::NonFiniteValue);
        }
        if !(0.0..=1.0).contains(&t_max) {
            return Err(CollisionError::FractionOutOfRange);
        }
        Ok(Self {
            shape_a,
            child_a,
            sweep_a,
            shape_b,
            child_b,
            sweep_b,
            t_max,
        })
    }

    /// Returns the inclusive upper bound of the query interval.
    #[must_use]
    pub const fn t_max(&self) -> f32 {
        self.t_max
    }
}

/// Closed source-compatible time-of-impact state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfImpactState {
    /// Shapes overlap at the initial time.
    Overlapped,
    /// Shapes reach the pinned target separation in the interval.
    Touching,
    /// Shapes remain separated through `t_max`.
    Separated,
    /// The bounded iterative algorithm did not make sufficient progress.
    Failed,
}

/// Public time-of-impact result with no iteration or support-coordinate API.
///
/// ```compile_fail
/// use liquidfun::collision::toi::{TimeOfImpactOutput, TimeOfImpactState};
///
/// let _output = TimeOfImpactOutput {
///     state: TimeOfImpactState::Touching,
///     time: 0.5,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeOfImpactOutput {
    state: TimeOfImpactState,
    time: f32,
}

impl TimeOfImpactOutput {
    /// Returns the closed termination state.
    #[must_use]
    pub const fn state(self) -> TimeOfImpactState {
        self.state
    }

    /// Returns the time in the inclusive query interval.
    #[must_use]
    pub const fn time(self) -> f32 {
        self.time
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RootMethod {
    Bisection,
    Secant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToiBranch {
    DistanceOverlapped,
    DistanceTouching,
    FinalSeparated,
    AdvanceSweep,
    InitialFailed,
    InitialTouching,
    RootConverged,
    RootCap,
    PushBackCap,
    OuterCap,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RootDiagnosticStep {
    method: RootMethod,
    indices: SeparationIndices,
    time: f32,
    separation: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ToiDiagnosticTrace {
    target: f32,
    tolerance: f32,
    separation_kinds: Vec<SeparationKind>,
    root_steps: Vec<RootDiagnosticStep>,
    branches: Vec<ToiBranch>,
    outer_iterations: usize,
    maximum_push_back_iterations: usize,
}

struct ToiRun {
    output: TimeOfImpactOutput,
    diagnostics: ToiDiagnosticTrace,
}

/// Computes the upper bound on impact time with the selected source-ordered,
/// fixed-cap algorithm.
///
/// The input sweeps are copied and normalized internally. Caller-owned sweep
/// values are never mutated.
///
/// # Errors
///
/// Returns a typed error if accepted finite inputs produce non-finite transform,
/// distance, separation, or root geometry.
pub fn time_of_impact(input: &TimeOfImpactInput<'_>) -> Result<TimeOfImpactOutput, CollisionError> {
    let run = run_time_of_impact(input)?;
    debug_assert!(run.diagnostics.is_bounded());
    Ok(run.output)
}

impl ToiDiagnosticTrace {
    fn is_bounded(&self) -> bool {
        self.separation_kinds.len() <= MAX_OUTER_ITERATIONS
            && self.root_steps.len()
                <= MAX_OUTER_ITERATIONS * MAX_POLYGON_VERTICES * MAX_ROOT_ITERATIONS
            && self.outer_iterations <= MAX_OUTER_ITERATIONS
            && self.maximum_push_back_iterations <= MAX_POLYGON_VERTICES
    }
}

fn run_time_of_impact(input: &TimeOfImpactInput<'_>) -> Result<ToiRun, CollisionError> {
    let proxy_a = ToiProxy::new(input.shape_a, input.child_a)?;
    let proxy_b = ToiProxy::new(input.shape_b, input.child_b)?;
    let mut sweep_a = input.sweep_a;
    let mut sweep_b = input.sweep_b;
    sweep_a.normalize();
    sweep_b.normalize();

    let total_radius = proxy_a.radius() + proxy_b.radius();
    let target = max(LINEAR_SLOP, total_radius - 3.0 * LINEAR_SLOP);
    let tolerance = 0.25 * LINEAR_SLOP;
    let mut diagnostics = ToiDiagnosticTrace {
        target,
        tolerance,
        separation_kinds: Vec::with_capacity(MAX_OUTER_ITERATIONS),
        root_steps: Vec::new(),
        branches: Vec::new(),
        outer_iterations: 0,
        maximum_push_back_iterations: 0,
    };
    let mut time1 = 0.0;
    let mut cache = DistanceCache::empty();

    loop {
        let transform_a = sweep_a
            .transform_at(time1)
            .map_err(|_error| CollisionError::NonFiniteValue)?;
        let transform_b = sweep_b
            .transform_at(time1)
            .map_err(|_error| CollisionError::NonFiniteValue)?;
        let distance_result = distance(
            input.shape_a,
            input.child_a,
            transform_a,
            input.shape_b,
            input.child_b,
            transform_b,
            false,
            Some(&cache),
        )?;
        cache = distance_result.cache().clone();

        if distance_result.distance() <= 0.0 {
            diagnostics.branches.push(ToiBranch::DistanceOverlapped);
            return Ok(finish(TimeOfImpactState::Overlapped, 0.0, diagnostics));
        }
        if distance_result.distance() < target + tolerance {
            diagnostics.branches.push(ToiBranch::DistanceTouching);
            return Ok(finish(TimeOfImpactState::Touching, time1, diagnostics));
        }

        let snapshot = cache.snapshot();
        let (separation, _initial_value) = SeparationFunction::initialize(
            snapshot.support_pairs(),
            &proxy_a,
            sweep_a,
            &proxy_b,
            sweep_b,
            time1,
        )?;
        diagnostics.separation_kinds.push(separation.kind());
        let inner = solve_separating_axis(
            &separation,
            input.t_max,
            target,
            tolerance,
            time1,
            &mut diagnostics,
        )?;
        diagnostics.outer_iterations += 1;
        diagnostics.maximum_push_back_iterations = diagnostics
            .maximum_push_back_iterations
            .max(inner.push_back_iterations);

        if let Some(output) = inner.maybe_output {
            return Ok(ToiRun {
                output,
                diagnostics,
            });
        }
        time1 = inner.next_time;
        if diagnostics.outer_iterations == MAX_OUTER_ITERATIONS {
            diagnostics.branches.push(ToiBranch::OuterCap);
            return Ok(finish(TimeOfImpactState::Failed, time1, diagnostics));
        }
    }
}

struct AxisResult {
    maybe_output: Option<TimeOfImpactOutput>,
    next_time: f32,
    push_back_iterations: usize,
}

fn solve_separating_axis(
    separation: &SeparationFunction<'_, '_>,
    t_max: f32,
    target: f32,
    tolerance: f32,
    time1: f32,
    diagnostics: &mut ToiDiagnosticTrace,
) -> Result<AxisResult, CollisionError> {
    let mut time2 = t_max;
    let mut push_back_iterations = 0;
    loop {
        let minimum = separation.find_minimum(time2)?;
        let mut separation2 = minimum.value;
        if separation2 > target + tolerance {
            diagnostics.branches.push(ToiBranch::FinalSeparated);
            return Ok(AxisResult {
                maybe_output: Some(output(TimeOfImpactState::Separated, t_max)),
                next_time: time1,
                push_back_iterations,
            });
        }
        if separation2 > target - tolerance {
            diagnostics.branches.push(ToiBranch::AdvanceSweep);
            return Ok(AxisResult {
                maybe_output: None,
                next_time: time2,
                push_back_iterations,
            });
        }

        let mut separation1 = separation.evaluate(minimum.indices, time1)?;
        if separation1 < target - tolerance {
            diagnostics.branches.push(ToiBranch::InitialFailed);
            return Ok(AxisResult {
                maybe_output: Some(output(TimeOfImpactState::Failed, time1)),
                next_time: time1,
                push_back_iterations,
            });
        }
        if separation1 <= target + tolerance {
            diagnostics.branches.push(ToiBranch::InitialTouching);
            return Ok(AxisResult {
                maybe_output: Some(output(TimeOfImpactState::Touching, time1)),
                next_time: time1,
                push_back_iterations,
            });
        }

        let mut lower_time = time1;
        let mut upper_time = time2;
        for root_iteration in 0..MAX_ROOT_ITERATIONS {
            let (method, candidate_time) = root_candidate(
                root_iteration,
                lower_time,
                upper_time,
                separation1,
                separation2,
                target,
            )?;
            let candidate_separation = separation.evaluate(minimum.indices, candidate_time)?;
            diagnostics.root_steps.push(RootDiagnosticStep {
                method,
                indices: minimum.indices,
                time: candidate_time,
                separation: candidate_separation,
            });
            if abs(candidate_separation - target) < tolerance {
                time2 = candidate_time;
                diagnostics.branches.push(ToiBranch::RootConverged);
                break;
            }
            if candidate_separation > target {
                lower_time = candidate_time;
                separation1 = candidate_separation;
            } else {
                upper_time = candidate_time;
                separation2 = candidate_separation;
            }
            if root_iteration + 1 == MAX_ROOT_ITERATIONS {
                diagnostics.branches.push(ToiBranch::RootCap);
            }
        }

        push_back_iterations += 1;
        if push_back_iterations == MAX_POLYGON_VERTICES {
            diagnostics.branches.push(ToiBranch::PushBackCap);
            return Ok(AxisResult {
                maybe_output: None,
                next_time: time1,
                push_back_iterations,
            });
        }
    }
}

fn root_candidate(
    root_iteration: usize,
    lower_time: f32,
    upper_time: f32,
    lower_separation: f32,
    upper_separation: f32,
    target: f32,
) -> Result<(RootMethod, f32), CollisionError> {
    let (method, candidate) = if root_iteration & 1 == 1 {
        (
            RootMethod::Secant,
            lower_time
                + (target - lower_separation) * (upper_time - lower_time)
                    / (upper_separation - lower_separation),
        )
    } else {
        (RootMethod::Bisection, 0.5 * (lower_time + upper_time))
    };
    if !candidate.is_finite() || !(lower_time..=upper_time).contains(&candidate) {
        return Err(CollisionError::NonFiniteValue);
    }
    Ok((method, candidate))
}

const fn output(state: TimeOfImpactState, time: f32) -> TimeOfImpactOutput {
    TimeOfImpactOutput { state, time }
}

const fn finish(state: TimeOfImpactState, time: f32, diagnostics: ToiDiagnosticTrace) -> ToiRun {
    ToiRun {
        output: output(state, time),
        diagnostics,
    }
}
