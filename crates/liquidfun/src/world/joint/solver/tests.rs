use super::*;
use crate::world::joint::JointRuntime;
use crate::{BodyDef, JointDef, RevoluteJointDef, World};

#[test]
fn solver_body_lane_retains_semantic_identity_when_resolved() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");

    // Act
    let lane = SolverBodyLane::resolved(body, 3);

    // Assert
    assert_eq!(lane.body_id(), body);
    assert_eq!(lane.maybe_solver_index(), Some(3));
}

#[test]
fn staged_revolute_uses_the_live_typed_constraint() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let body_a = world
        .create_body(&BodyDef::default())
        .expect("body A should fit");
    let body_b = world
        .create_body(&BodyDef::default())
        .expect("body B should fit");
    let definition = RevoluteJointDef::new(body_a, body_b).expect("valid definition");
    let joint_id = world
        .create_joint(JointDef::from(definition))
        .expect("joint should fit");
    let record = world.joints.get(joint_id).expect("joint remains live");
    let JointRuntime::Revolute(runtime) = record.runtime else {
        panic!("revolute runtime should match its definition");
    };
    let input = JointConstraintInput::ordinary(
        joint_id,
        OrdinarySolverLanes::resolved(body_a, 0, body_b, 1),
        record.definition,
        record.runtime,
    );
    let bodies = [test_solver_body(), test_solver_body()];

    // Act
    let constraints = build_constraints(&[input], &bodies, 1.0 / 60.0, 1.0, true)
        .expect("live revolute staging should remain available");

    // Assert
    assert_eq!(constraints.len(), 1);
    let JointVelocityConstraint::Revolute(stage) = constraints[0] else {
        panic!("revolute must use the typed live constraint");
    };
    assert_eq!(stage.candidate.joint_id, joint_id);
    assert_eq!(stage.candidate.definition, definition);
    assert_eq!(
        stage.candidate.runtime.reaction_force(1.0),
        runtime.reaction_force(1.0)
    );
}

#[test]
fn unresolved_lane_is_rejected_before_typed_dispatch() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let body_a = world
        .create_body(&BodyDef::default())
        .expect("body A should fit");
    let body_b = world
        .create_body(&BodyDef::default())
        .expect("body B should fit");
    let definition = RevoluteJointDef::new(body_a, body_b).expect("valid definition");
    let joint_id = world
        .create_joint(definition.into())
        .expect("joint should fit");
    let record = world.joints.get(joint_id).expect("joint remains live");
    let input = JointConstraintInput::ordinary(
        joint_id,
        OrdinarySolverLanes::new(
            SolverBodyLane::resolved(body_a, 0),
            SolverBodyLane::unresolved(body_b),
        ),
        record.definition,
        record.runtime,
    );

    // Act
    let result = build_constraints(
        &[input],
        &[test_solver_body(), test_solver_body()],
        1.0 / 60.0,
        1.0,
        true,
    );

    // Assert
    assert!(matches!(
        result,
        Err(ContactSolveFailure::UnsupportedTopology)
    ));
}

#[test]
fn typed_solution_stages_the_complete_runtime() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let body_a = world
        .create_body(&BodyDef::default())
        .expect("body A should fit");
    let body_b = world
        .create_body(&BodyDef::default())
        .expect("body B should fit");
    let definition = RevoluteJointDef::new(body_a, body_b).expect("valid definition");
    let joint_id = world
        .create_joint(definition.into())
        .expect("joint should fit");
    let record = world.joints.get(joint_id).expect("joint remains live");
    let JointRuntime::Revolute(runtime) = record.runtime else {
        panic!("revolute runtime should match its definition");
    };
    // Act
    let solution = typed_solution(joint_id, JointRuntime::Revolute(runtime));

    // Assert
    assert_eq!(solution.joint_id, joint_id);
    assert!(matches!(
        solution.runtime,
        JointRuntime::Revolute(candidate_runtime)
            if candidate_runtime.reaction_force(1.0) == runtime.reaction_force(1.0)
    ));
}

#[test]
fn staged_constraints_preserve_source_input_order() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let body_a = world
        .create_body(&BodyDef::default())
        .expect("body A should fit");
    let body_b = world
        .create_body(&BodyDef::default())
        .expect("body B should fit");
    let first = world
        .create_joint(
            RevoluteJointDef::new(body_a, body_b)
                .expect("valid first joint")
                .into(),
        )
        .expect("first joint should fit");
    let second = world
        .create_joint(
            RevoluteJointDef::new(body_a, body_b)
                .expect("valid second joint")
                .into(),
        )
        .expect("second joint should fit");
    let inputs = [second, first].map(|joint_id| {
        let record = world.joints.get(joint_id).expect("joint remains live");
        JointConstraintInput::ordinary(
            joint_id,
            OrdinarySolverLanes::resolved(body_a, 0, body_b, 1),
            record.definition,
            record.runtime,
        )
    });
    let bodies = [test_solver_body(), test_solver_body()];

    // Act
    let constraints = build_constraints(&inputs, &bodies, 1.0 / 60.0, 1.0, true)
        .expect("source inputs should stage");

    // Assert
    let joint_ids = constraints
        .iter()
        .map(|constraint| {
            let JointVelocityConstraint::Revolute(stage) = constraint else {
                panic!("both inputs should be live revolute candidates");
            };
            stage.candidate.joint_id
        })
        .collect::<Vec<_>>();
    assert_eq!(joint_ids, vec![second, first]);
}

#[test]
fn gear_lanes_resolve_abcd_in_semantic_order_and_reject_absence() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let [body_a, body_b, body_c, body_d] = std::array::from_fn(|_| {
        world
            .create_body(&BodyDef::default())
            .expect("body should fit")
    });
    let resolved = GearSolverLanes::new(
        SolverBodyLane::resolved(body_a, 3),
        SolverBodyLane::resolved(body_b, 1),
        SolverBodyLane::resolved(body_c, 0),
        SolverBodyLane::resolved(body_d, 2),
    );
    let missing = GearSolverLanes::new(
        SolverBodyLane::resolved(body_a, 3),
        SolverBodyLane::resolved(body_b, 1),
        SolverBodyLane::unresolved(body_c),
        SolverBodyLane::resolved(body_d, 2),
    );
    let bodies = [
        test_solver_body(),
        test_solver_body(),
        test_solver_body(),
        test_solver_body(),
    ];

    // Act
    let indices = resolved.solver_indices(&bodies);
    let missing_result = missing.solver_indices(&bodies);

    // Assert
    assert_eq!(indices, Ok([3, 1, 0, 2]));
    assert_eq!(
        missing_result,
        Err(ContactSolveFailure::UnsupportedTopology)
    );
}

#[test]
fn gear_alias_scatter_combines_repeated_lane_deltas() {
    // Arrange
    let mut bodies = [test_solver_body(), test_solver_body(), test_solver_body()];
    let indices = [0, 1, 2, 2];
    let before = gear_solver_bodies(indices, &bodies).expect("gear bodies");
    let mut solved = before;
    solved[2].linear_velocity += Vec2::new(1.0, -2.0);
    solved[2].angular_velocity += 3.0;
    solved[3].linear_velocity += Vec2::new(4.0, 5.0);
    solved[3].angular_velocity -= 1.0;

    // Act
    store_gear_velocity_deltas(indices, &mut bodies, before, solved)
        .expect("aliased deltas should merge");

    // Assert
    assert_eq!(bodies[2].linear_velocity, Vec2::new(5.0, 3.0));
    assert_eq!(bodies[2].angular_velocity.to_bits(), 2.0_f32.to_bits());
}

fn test_solver_body() -> SolverBody {
    SolverBody {
        center: Vec2::ZERO,
        local_center: Vec2::ZERO,
        angle: 0.0,
        transform: crate::math::Transform::IDENTITY,
        linear_velocity: Vec2::ZERO,
        angular_velocity: 0.0,
        inverse_mass: 1.0,
        inverse_inertia: 1.0,
    }
}
