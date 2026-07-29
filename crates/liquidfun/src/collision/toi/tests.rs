use super::*;
use crate::collision::shape::CircleShape;
use crate::math::Vec2;

fn circle(radius: f32) -> Shape {
    CircleShape::new(Vec2::ZERO, radius)
        .expect("test circle should be valid")
        .into()
}

fn sweep(initial_center: Vec2, center: Vec2) -> Sweep {
    Sweep::new(Vec2::ZERO, initial_center, center, 0.0, 0.0, 0.0)
        .expect("test sweep should be valid")
}

#[test]
fn toi_target_and_tolerance_preserve_pinned_formula() {
    // Arrange
    let total_radius = 2.0 * 2.0 * LINEAR_SLOP;

    // Act
    let (target, tolerance) = target_and_tolerance(total_radius);

    // Assert
    assert_eq!(target.to_bits(), LINEAR_SLOP.to_bits());
    assert_eq!(tolerance.to_bits(), (0.25 * LINEAR_SLOP).to_bits());
}

#[test]
fn toi_target_uses_radius_minus_three_slops_when_larger() {
    // Arrange
    let total_radius = 2.0;

    // Act
    let (target, _tolerance) = target_and_tolerance(total_radius);

    // Assert
    assert_eq!(target.to_bits(), (2.0 - 3.0 * LINEAR_SLOP).to_bits());
}

#[test]
fn toi_root_method_starts_with_bisection_then_alternates() {
    // Arrange
    let bracket = (0.0, 1.0, 3.0, 1.0, 2.0);

    // Act
    let first = root_candidate(0, bracket.0, bracket.1, bracket.2, bracket.3, bracket.4)
        .expect("first root candidate should be valid");
    let second = root_candidate(1, bracket.0, bracket.1, bracket.2, bracket.3, bracket.4)
        .expect("second root candidate should be valid");
    let third = root_candidate(2, bracket.0, bracket.1, bracket.2, bracket.3, bracket.4)
        .expect("third root candidate should be valid");

    // Assert
    assert_eq!(first.0, RootMethod::Bisection);
    assert_eq!(second.0, RootMethod::Secant);
    assert_eq!(third.0, RootMethod::Bisection);
}

#[test]
fn toi_caps_trigger_at_exact_source_counts() {
    // Arrange
    let below_outer = MAX_OUTER_ITERATIONS - 1;
    let below_root = MAX_ROOT_ITERATIONS - 1;
    let below_push_back = MAX_POLYGON_VERTICES - 1;

    // Act
    let outer_before = iteration_reached_cap(below_outer, MAX_OUTER_ITERATIONS);
    let outer_at = iteration_reached_cap(MAX_OUTER_ITERATIONS, MAX_OUTER_ITERATIONS);
    let root_before = iteration_reached_cap(below_root, MAX_ROOT_ITERATIONS);
    let root_at = iteration_reached_cap(MAX_ROOT_ITERATIONS, MAX_ROOT_ITERATIONS);
    let push_back_before = iteration_reached_cap(below_push_back, MAX_POLYGON_VERTICES);
    let push_back_at = iteration_reached_cap(MAX_POLYGON_VERTICES, MAX_POLYGON_VERTICES);

    // Assert
    assert!(!outer_before);
    assert!(outer_at);
    assert!(!root_before);
    assert!(root_at);
    assert!(!push_back_before);
    assert!(push_back_at);
}

#[test]
fn toi_cap_diagnostics_record_exact_termination_causes() {
    // Arrange
    let mut diagnostics = ToiDiagnosticTrace::new(2.0);
    let cases = [
        (MAX_OUTER_ITERATIONS, ToiBranch::OuterCap),
        (MAX_ROOT_ITERATIONS, ToiBranch::RootCap),
        (MAX_POLYGON_VERTICES, ToiBranch::PushBackCap),
    ];

    // Act
    for (cap, branch) in cases {
        assert!(!record_cap_if_reached(
            &mut diagnostics,
            cap - 1,
            cap,
            branch,
        ));
        assert!(record_cap_if_reached(&mut diagnostics, cap, cap, branch,));
    }

    // Assert
    assert_eq!(
        diagnostics.branches,
        [
            ToiBranch::OuterCap,
            ToiBranch::RootCap,
            ToiBranch::PushBackCap,
        ]
    );
}

#[test]
fn toi_outer_cap_finishes_with_closed_failed_state() {
    // Arrange
    let time = 0.75;
    let mut diagnostics = ToiDiagnosticTrace::new(2.0);
    diagnostics.outer_iterations = MAX_OUTER_ITERATIONS;
    let reached = record_cap_if_reached(
        &mut diagnostics,
        MAX_OUTER_ITERATIONS,
        MAX_OUTER_ITERATIONS,
        ToiBranch::OuterCap,
    );

    // Act
    let run = finish(TimeOfImpactState::Failed, time, diagnostics);

    // Assert
    assert!(reached);
    assert_eq!(run.output.state(), TimeOfImpactState::Failed);
    assert_eq!(run.output.time().to_bits(), time.to_bits());
    assert!(run.diagnostics.is_bounded());
    assert_eq!(run.diagnostics.branches, [ToiBranch::OuterCap]);
}

#[test]
fn toi_diagnostic_branch_history_has_an_explicit_bound() {
    // Arrange
    let mut diagnostics = ToiDiagnosticTrace::new(2.0);
    diagnostics.branches = vec![ToiBranch::RootConverged; MAX_DIAGNOSTIC_BRANCHES];

    // Act
    let at_bound = diagnostics.is_bounded();
    diagnostics.branches.push(ToiBranch::OuterCap);
    let above_bound = diagnostics.is_bounded();

    // Assert
    assert!(at_bound);
    assert!(!above_bound);
}

#[test]
fn toi_translation_diagnostics_preserve_root_order_and_bounds() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let input = TimeOfImpactInput::new(
        &shape_a,
        child_a,
        sweep(Vec2::ZERO, Vec2::ZERO),
        &shape_b,
        child_b,
        sweep(Vec2::new(5.0, 0.0), Vec2::ZERO),
        1.0,
    )
    .expect("TOI input should be valid");

    // Act
    let run = run_time_of_impact(&input).expect("finite TOI query should execute");
    let methods: Vec<_> = run
        .diagnostics
        .root_steps
        .iter()
        .map(|step| step.method)
        .collect();

    // Assert
    assert_eq!(run.output.state(), TimeOfImpactState::Touching);
    assert!(run.diagnostics.is_bounded());
    assert_eq!(methods, [RootMethod::Bisection, RootMethod::Secant]);
    assert!(run.diagnostics.branches.contains(&ToiBranch::RootConverged));
    assert!(
        run.diagnostics
            .branches
            .contains(&ToiBranch::DistanceTouching)
    );
}
