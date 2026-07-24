//! Complete matrix and measured-region contracts for paired native benchmarks.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use liquidfun_benchmarks::{BenchmarkExecutionBounds, PairedEngineOrder, paired_benchmark_cases};
use liquidfun_differential::{
    NativeBenchmarkClock, NativeBenchmarkDriver, PerformanceExecutionError,
    benchmark_semantics_match, measure_native_actions,
};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, HarnessLimits, PerformanceSizePoint, PerformanceWorkloadKind,
    decode_canonical_checkpoint_jsonl,
};

#[test]
fn complete_matrix_registers_every_workload_and_reviewed_size_point() {
    // Arrange / Act
    let cases = paired_benchmark_cases().expect("reviewed paired cases should prepare");

    // Assert
    assert_eq!(cases.len(), 32);
    assert_eq!(
        cases
            .iter()
            .map(liquidfun_benchmarks::PairedBenchmarkCase::workload)
            .collect::<std::collections::BTreeSet<_>>(),
        PerformanceWorkloadKind::ALL.into_iter().collect()
    );
    for workload in PerformanceWorkloadKind::ALL {
        let size_points = cases
            .iter()
            .filter(|case| case.workload() == workload)
            .map(liquidfun_benchmarks::PairedBenchmarkCase::size_point)
            .collect::<Vec<_>>();
        let expected = match workload {
            PerformanceWorkloadKind::WorldStep
            | PerformanceWorkloadKind::NarrowPhase
            | PerformanceWorkloadKind::ContactSolve
            | PerformanceWorkloadKind::Ccd
            | PerformanceWorkloadKind::Joints => vec![PerformanceSizePoint::Fixed],
            _ => vec![
                PerformanceSizePoint::WorkUnits128,
                PerformanceSizePoint::WorkUnits1024,
                PerformanceSizePoint::WorkUnits8192,
            ],
        };
        assert_eq!(size_points, expected, "size-point drift for {workload:?}");
    }
}

#[test]
fn paired_cases_bind_exact_canonical_bytes_and_alternate_caller_order() {
    // Arrange
    let cases = paired_benchmark_cases().expect("reviewed paired cases should prepare");

    // Act / Assert
    for case in &cases {
        assert_eq!(
            case.resolved_sha256(),
            case.resolved().identity().content_sha256()
        );
        assert_eq!(case.sample_order(0), PairedEngineOrder::NativeThenOracle);
        assert_eq!(case.sample_order(1), PairedEngineOrder::OracleThenNative);
    }
}

#[test]
fn resource_bounds_reject_first_values_above_reviewed_limits() {
    // Arrange / Act
    let warmups = BenchmarkExecutionBounds::new(101, 30, 1);
    let samples = BenchmarkExecutionBounds::new(1, 10_001, 1);
    let actions = BenchmarkExecutionBounds::new(1, 30, 1_000_001);

    // Assert
    assert!(warmups.is_err());
    assert!(samples.is_err());
    assert!(actions.is_err());
    assert!(BenchmarkExecutionBounds::new(100, 10_000, 1_000_000).is_ok());
}

#[test]
fn injected_clock_proves_only_declared_actions_are_timed() {
    // Arrange
    let events = Rc::new(RefCell::new(vec!["setup"]));
    let mut driver = CountingDriver::new(Rc::clone(&events));
    let mut clock = CountingClock::new(Rc::clone(&events));

    // Act
    let duration = measure_native_actions(&mut driver, &mut clock, 3)
        .expect("bounded fake sample should succeed");

    // Assert
    assert_eq!(duration, Duration::from_nanos(17));
    assert_eq!(
        *events.borrow(),
        [
            "setup",
            "restart",
            "timer_start",
            "action",
            "action",
            "action",
            "timer_stop",
            "capture",
            "validate",
            "teardown",
        ]
    );
}

#[test]
fn scalable_injected_clock_proves_every_unit_is_prepared_before_timing() {
    // Arrange
    let state = Rc::new(RefCell::new(ScalableTimingState::default()));
    let mut driver = ScalableCountingDriver::new(Rc::clone(&state));
    let mut clock = ScalableCountingClock::new(Rc::clone(&state));

    // Act
    let duration = measure_native_actions(&mut driver, &mut clock, 128)
        .expect("scalable fake sample should succeed");

    // Assert
    let state = state.borrow();
    assert_eq!(duration, Duration::from_nanos(23));
    assert_eq!(state.prepared_units, 128);
    assert_eq!(state.executed_actions, 128);
    assert!(!state.setup_during_timer);
}

#[test]
fn independent_preparations_have_identical_semantic_authority() {
    // Arrange / Act
    let first = paired_benchmark_cases().expect("first preparation should pass");
    let second = paired_benchmark_cases().expect("second preparation should pass");

    // Assert
    assert_eq!(first.len(), second.len());
    for (left, right) in first.iter().zip(&second) {
        assert_eq!(left.resolved_sha256(), right.resolved_sha256());
        assert!(benchmark_semantics_match(
            left.expected_checkpoint(),
            right.expected_checkpoint()
        ));
    }
}

#[test]
fn visual_primitive_order_and_ordinal_drift_do_not_reject_physics_semantics() {
    // Arrange
    let cases = paired_benchmark_cases().expect("reviewed paired cases should prepare");
    let expected = cases
        .iter()
        .find(|case| !case.expected_checkpoint().debug_primitives().is_empty())
        .expect("at least one reviewed case should expose debug primitives")
        .expected_checkpoint();
    let mut value = serde_json::to_value(expected).expect("checkpoint should serialize");
    let primitives = value["debug_primitives"]
        .as_array_mut()
        .expect("debug primitive field should remain an array");
    primitives.reverse();
    primitives[0]["primitive"]["value"]["metadata"]["key"]["ordinal"] =
        serde_json::Value::from(999);
    let visual_only = decode_checkpoint(&value);

    // Act / Assert
    assert_ne!(expected, &visual_only);
    assert!(benchmark_semantics_match(expected, &visual_only));
}

#[test]
fn authoritative_observation_drift_rejects_physics_semantics() {
    // Arrange
    let cases = paired_benchmark_cases().expect("reviewed paired cases should prepare");
    let expected = cases[0].expected_checkpoint();
    let mut value = serde_json::to_value(expected).expect("checkpoint should serialize");
    value["observations"][0]["value"]["value"] = serde_json::Value::from(999);
    let changed = decode_checkpoint(&value);

    // Act / Assert
    assert!(!benchmark_semantics_match(expected, &changed));
}

fn decode_checkpoint(value: &serde_json::Value) -> CanonicalCheckpoint {
    let mut bytes = serde_json::to_vec(value).expect("checkpoint JSON should encode");
    bytes.push(b'\n');
    decode_canonical_checkpoint_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded checkpoint mutation should remain valid")
}

struct CountingClock {
    events: Rc<RefCell<Vec<&'static str>>>,
}

impl CountingClock {
    fn new(events: Rc<RefCell<Vec<&'static str>>>) -> Self {
        Self { events }
    }
}

impl NativeBenchmarkClock for CountingClock {
    type Stamp = ();

    fn start(&mut self) -> Self::Stamp {
        self.events.borrow_mut().push("timer_start");
    }

    fn elapsed(&mut self, (): Self::Stamp) -> Duration {
        self.events.borrow_mut().push("timer_stop");
        Duration::from_nanos(17)
    }
}

struct CountingDriver {
    events: Rc<RefCell<Vec<&'static str>>>,
}

impl CountingDriver {
    fn new(events: Rc<RefCell<Vec<&'static str>>>) -> Self {
        Self { events }
    }
}

impl NativeBenchmarkDriver for CountingDriver {
    type Checkpoint = ();

    fn restart(&mut self) -> Result<(), PerformanceExecutionError> {
        self.events.borrow_mut().push("restart");
        Ok(())
    }

    fn execute_action(&mut self) -> Result<(), PerformanceExecutionError> {
        self.events.borrow_mut().push("action");
        Ok(())
    }

    fn capture(&mut self) -> Result<Self::Checkpoint, PerformanceExecutionError> {
        self.events.borrow_mut().push("capture");
        Ok(())
    }

    fn validate(&mut self, (): &Self::Checkpoint) -> Result<(), PerformanceExecutionError> {
        self.events.borrow_mut().push("validate");
        Ok(())
    }

    fn teardown(&mut self) {
        self.events.borrow_mut().push("teardown");
    }
}

#[derive(Default)]
struct ScalableTimingState {
    prepared_units: u32,
    executed_actions: u32,
    timer_running: bool,
    setup_during_timer: bool,
}

struct ScalableCountingClock {
    state: Rc<RefCell<ScalableTimingState>>,
}

impl ScalableCountingClock {
    fn new(state: Rc<RefCell<ScalableTimingState>>) -> Self {
        Self { state }
    }
}

impl NativeBenchmarkClock for ScalableCountingClock {
    type Stamp = ();

    fn start(&mut self) -> Self::Stamp {
        let mut state = self.state.borrow_mut();
        assert_eq!(state.prepared_units, 128);
        state.timer_running = true;
    }

    fn elapsed(&mut self, (): Self::Stamp) -> Duration {
        self.state.borrow_mut().timer_running = false;
        Duration::from_nanos(23)
    }
}

struct ScalableCountingDriver {
    state: Rc<RefCell<ScalableTimingState>>,
}

impl ScalableCountingDriver {
    fn new(state: Rc<RefCell<ScalableTimingState>>) -> Self {
        Self { state }
    }
}

impl NativeBenchmarkDriver for ScalableCountingDriver {
    type Checkpoint = ();

    fn restart(&mut self) -> Result<(), PerformanceExecutionError> {
        let mut state = self.state.borrow_mut();
        state.setup_during_timer |= state.timer_running;
        state.prepared_units = 128;
        Ok(())
    }

    fn execute_action(&mut self) -> Result<(), PerformanceExecutionError> {
        let mut state = self.state.borrow_mut();
        assert!(state.timer_running);
        assert_eq!(state.prepared_units, 128);
        state.executed_actions += 1;
        Ok(())
    }

    fn capture(&mut self) -> Result<Self::Checkpoint, PerformanceExecutionError> {
        assert!(!self.state.borrow().timer_running);
        Ok(())
    }

    fn validate(&mut self, (): &Self::Checkpoint) -> Result<(), PerformanceExecutionError> {
        Ok(())
    }

    fn teardown(&mut self) {}
}
