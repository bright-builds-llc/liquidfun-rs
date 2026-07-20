//! Phase 9 rigid-world non-regression through the complete particle solver.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleDef, ParticleFlags};
use liquidfun::{
    BodyDef, BodySnapshot, BodyType, ContactView, DistanceJointDef, FixtureDef,
    JointSpecificSnapshot, PreSolveDirective, StepConfiguration, StepHook, StepLimits, World,
};

#[derive(Default)]
struct RecordingHook {
    observations: usize,
}

impl StepHook for RecordingHook {
    fn pre_solve(&mut self, _contact: ContactView<'_>) -> PreSolveDirective {
        PreSolveDirective::Disable
    }

    fn observe(&mut self, _contact: ContactView<'_>) {
        self.observations += 1;
    }
}

fn body_bits(snapshot: BodySnapshot) -> [u32; 8] {
    [
        snapshot.position().x.to_bits(),
        snapshot.position().y.to_bits(),
        snapshot.angle().to_bits(),
        snapshot.linear_velocity().x.to_bits(),
        snapshot.linear_velocity().y.to_bits(),
        snapshot.angular_velocity().to_bits(),
        snapshot.mass().to_bits(),
        snapshot.rotational_inertia().to_bits(),
    ]
}

fn populated_world(
    with_inactive_particles: bool,
) -> (
    World,
    [liquidfun::BodyId; 2],
    liquidfun::JointId,
    Option<liquidfun::ParticleId>,
) {
    let mut world = World::new().expect("world key remains available");
    let first = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
                .expect("body definition is valid"),
        )
        .expect("first body fits");
    let second = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(0.75, 0.0), 0.0, true)
                .expect("body definition is valid"),
        )
        .expect("second body fits");
    let fixture = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle is valid")),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("fixture is valid");
    world.create_fixture(first, &fixture).expect("fixture fits");
    world
        .create_fixture(second, &fixture)
        .expect("fixture fits");
    let joint = world
        .create_joint(
            DistanceJointDef::new(first, second)
                .expect("distance joint is valid")
                .into(),
        )
        .expect("joint fits");
    let maybe_particle = if with_inactive_particles {
        let system = world.create_particle_system().expect("system fits");
        Some(
            world
                .create_particle_with_def(
                    system,
                    None,
                    &ParticleDef::default()
                        .with_position(Vec2::new(100.0, 100.0))
                        .expect("position is finite")
                        .with_flags(ParticleFlags::WATER),
                )
                .expect("particle fits")
                .created_particle(),
        )
    } else {
        None
    };
    (world, [first, second], joint, maybe_particle)
}

#[test]
fn inactive_phase10_particle_behaviors_preserve_phase9_rigid_world_semantics() {
    // Arrange
    let (mut baseline, baseline_bodies, baseline_joint, _) = populated_world(false);
    let (mut candidate, candidate_bodies, candidate_joint, maybe_particle) = populated_world(true);
    let configuration = StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("configuration is valid")
        .with_particle_iterations(2)
        .expect("iteration count is valid");
    let mut baseline_hook = RecordingHook::default();
    let mut candidate_hook = RecordingHook::default();

    // Act
    let baseline_report = baseline
        .step(configuration, &mut baseline_hook, StepLimits::default())
        .expect("baseline step succeeds");
    let candidate_report = candidate
        .step(configuration, &mut candidate_hook, StepLimits::default())
        .expect("candidate step succeeds");

    // Assert
    assert_eq!(baseline_report.phases(), candidate_report.phases());
    assert_eq!(
        baseline_report.events().len(),
        candidate_report.events().len()
    );
    assert_eq!(
        baseline_report.contact_solves().len(),
        candidate_report.contact_solves().len()
    );
    assert_eq!(baseline_hook.observations, candidate_hook.observations);
    assert_eq!(
        baseline_bodies
            .map(|body| body_bits(baseline.body_snapshot(body).expect("body remains live"))),
        candidate_bodies
            .map(|body| body_bits(candidate.body_snapshot(body).expect("body remains live"))),
    );
    let JointSpecificSnapshot::Distance(baseline_distance) = baseline
        .joint_snapshot(baseline_joint)
        .expect("joint remains live")
        .specific()
    else {
        panic!("distance snapshot expected");
    };
    let JointSpecificSnapshot::Distance(candidate_distance) = candidate
        .joint_snapshot(candidate_joint)
        .expect("joint remains live")
        .specific()
    else {
        panic!("distance snapshot expected");
    };
    assert_eq!(
        baseline_distance.current_length().to_bits(),
        candidate_distance.current_length().to_bits()
    );
    let particle = maybe_particle.expect("candidate includes one particle");
    let particle = candidate
        .particle_snapshot(particle)
        .expect("water particle remains live");
    assert!(particle.position().is_valid());
    assert!(particle.velocity().is_valid());
    assert_eq!(particle.flags(), ParticleFlags::WATER);
}
