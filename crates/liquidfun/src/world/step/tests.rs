use crate::math::Vec2;
use crate::{BodyDef, BodyType, StepConfiguration, WakePolicy, World};

use super::StepCompletion;

#[test]
fn successful_continuous_pending_path_clears_forces() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("test body definition should be valid");
    let body = world
        .create_body(&definition)
        .expect("test body should fit");
    world
        .apply_body_force_to_center(body, Vec2::new(f32::MAX, 0.0), WakePolicy::Wake)
        .expect("first finite force should be accepted");
    let configuration =
        StepConfiguration::new(1.0 / 60.0, 8, 3).expect("test step configuration should be valid");
    let timing = world.prepare_step_timing(configuration);

    // Act
    let completion = world.finish_successful_step(timing, StepCompletion::ContinuousPending);

    // Assert
    assert_eq!(completion, StepCompletion::ContinuousPending);
    world
        .apply_body_force_to_center(body, Vec2::new(f32::MAX, 0.0), WakePolicy::Wake)
        .expect("successful pending path should clear the force accumulator");
}
