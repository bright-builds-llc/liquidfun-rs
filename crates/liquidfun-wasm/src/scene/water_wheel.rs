//! Unimplemented Water Wheel scene. A later plan owns the persistent world.

use super::BuiltScene;
use crate::session::SessionError;

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = presets;
    Err(SessionError::SceneUnimplemented)
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use liquidfun::math::Vec2;
    use liquidfun::{BodyType, JointDef, WorldObservationLimits};

    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::SessionCore;

    #[test]
    fn create_water_wheel_builds_hub_circle_and_paddle_segments() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::WaterWheel)
            .expect("Water Wheel should construct a native pinned wheel");
        let frame = capture(&session);

        // Assert
        assert!(
            frame.rigid_circles().len() >= 3,
            "hub circle reports at least x, y, radius"
        );
        assert!(
            frame.rigid_segments().len() >= 16,
            "at least four paddle segments report start/end floats"
        );
        assert!(session.rigid_shape_count() >= 5);
    }

    #[test]
    fn revolute_joint_is_created_without_a_motor() {
        // Arrange / Act
        let super::BuiltScene { world, .. } =
            super::build(&[]).expect("Water Wheel should construct");
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the revolute hub");
        let revolute: Vec<_> = observation
            .joints()
            .iter()
            .filter_map(|joint| match joint.snapshot().definition() {
                JointDef::Revolute(definition) => Some(definition),
                _ => None,
            })
            .collect();

        // Assert
        assert_eq!(revolute.len(), 1, "exactly one revolute pin should exist");
        assert!(
            !revolute[0].is_motor_enabled(),
            "D-09 forbids with_motor on the water-wheel hub"
        );
        assert_eq!(revolute[0].motor_speed().to_bits(), 0.0_f32.to_bits());
        assert_eq!(revolute[0].max_motor_torque().to_bits(), 0.0_f32.to_bits());
    }

    #[test]
    fn collect_segments_follow_a_forced_wheel_transform() {
        // Arrange
        let super::BuiltScene {
            mut world,
            hooks,
            ..
        } = super::build(&[]).expect("Water Wheel should construct");
        let before = flatten_segments(
            &hooks
                .collect_segments(&world)
                .expect("paddles should capture from the live pose"),
        );
        let wheel = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the wheel body")
            .bodies()
            .iter()
            .find(|body| body.snapshot().body_type() == BodyType::Dynamic)
            .expect("the paddle wheel is the dynamic body")
            .id();

        // Act
        world
            .set_body_transform(wheel, Vec2::new(0.0, 3.0), TAU / 8.0)
            .expect("test-only pose change proves capture uses the live transform");
        let after = flatten_segments(
            &hooks
                .collect_segments(&world)
                .expect("rotated paddles should still capture"),
        );

        // Assert
        assert!(after.len() >= 16);
        assert_ne!(
            before, after,
            "paddle segments must come from BodySnapshot::transform().apply, not constants"
        );
    }

    fn flatten_segments(segments: &[crate::scene::RigidSegment]) -> Vec<f32> {
        let mut values = Vec::with_capacity(segments.len().saturating_mul(4));
        for segment in segments {
            values.extend_from_slice(&[
                segment.start.x,
                segment.start.y,
                segment.end.x,
                segment.end.y,
            ]);
        }
        values
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Water Wheel should capture a frame"),
        )
    }
}
