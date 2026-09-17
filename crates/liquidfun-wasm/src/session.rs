//! Native-testable ownership and stepping for one proof scene.

use liquidfun::{BodyId, NoDecisionHook, ParticleSystemId, StepConfiguration, StepLimits, World};

use crate::frame::FrameData;
use crate::scene::{PARTICLE_COUNT, PARTICLE_RADIUS, ProofScene, RigidSegment, build_proof_scene};

pub(crate) const MAX_ADVANCE_STEPS: u32 = 4;
const MAX_FRAME_PARTICLES: usize = 512;
const PARTICLE_POSITION_STRIDE: usize = 2;
const PARTICLE_COLOR_STRIDE: usize = 4;
const RIGID_SEGMENT_STRIDE: usize = 4;
const RIGID_CIRCLE_STRIDE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionError {
    SceneConstruction,
    StepCountOutOfRange,
    StepIndexExhausted,
    StepFailed,
    FrameCaptureFailed,
}

impl SessionError {
    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::SceneConstruction => "Rust/WASM scene construction failed",
            Self::StepCountOutOfRange => "Rust/WASM step count must be within 1 through 4",
            Self::StepIndexExhausted => "Rust/WASM step index exhausted",
            Self::StepFailed => "Rust/WASM simulation step failed",
            Self::FrameCaptureFailed => "Rust/WASM frame capture failed",
        }
    }
}

pub(crate) struct SessionCore {
    world: World,
    particle_system: ParticleSystemId,
    dynamic_circle: BodyId,
    basin_segments: [RigidSegment; 3],
    circle_radius: f32,
    particle_count: usize,
    step_configuration: StepConfiguration,
    step_limits: StepLimits,
    step_index: u32,
}

impl SessionCore {
    pub(crate) fn new() -> Result<Self, SessionError> {
        let ProofScene {
            world,
            particle_system,
            dynamic_circle,
            basin_segments,
            circle_radius,
        } = build_proof_scene().map_err(|_error| SessionError::SceneConstruction)?;
        let step_configuration = StepConfiguration::new(1.0 / 60.0, 8, 3)
            .map_err(|_error| SessionError::SceneConstruction)?
            .with_particle_iterations(2)
            .map_err(|_error| SessionError::SceneConstruction)?;

        Ok(Self {
            world,
            particle_system,
            dynamic_circle,
            basin_segments,
            circle_radius,
            particle_count: PARTICLE_COUNT,
            step_configuration,
            step_limits: StepLimits::default(),
            step_index: 0,
        })
    }

    pub(crate) fn advance(&mut self, step_count: u32) -> Result<(), SessionError> {
        if !(1..=MAX_ADVANCE_STEPS).contains(&step_count) {
            return Err(SessionError::StepCountOutOfRange);
        }
        let next_step_index = self
            .step_index
            .checked_add(step_count)
            .ok_or(SessionError::StepIndexExhausted)?;

        for _ in 0..step_count {
            self.world
                .step(
                    self.step_configuration,
                    &mut NoDecisionHook,
                    self.step_limits,
                )
                .map_err(|_error| SessionError::StepFailed)?;
        }

        self.step_index = next_step_index;
        Ok(())
    }

    pub(crate) fn capture_frame(&self) -> Result<FrameData, SessionError> {
        let (particle_positions, particle_colors, particle_radii) = {
            let view = self
                .world
                .particle_system_view(self.particle_system)
                .map_err(|_error| SessionError::FrameCaptureFailed)?;
            let particle_ids = view.particle_ids();
            let positions = view.positions();
            let maybe_colors = view.maybe_colors();
            let Some(colors) = maybe_colors else {
                return Err(SessionError::FrameCaptureFailed);
            };
            if particle_ids.len() != positions.len() || particle_ids.len() != colors.len() {
                return Err(SessionError::FrameCaptureFailed);
            }
            if particle_ids.len() > MAX_FRAME_PARTICLES {
                return Err(SessionError::FrameCaptureFailed);
            }

            let position_capacity = particle_ids
                .len()
                .checked_mul(PARTICLE_POSITION_STRIDE)
                .ok_or(SessionError::FrameCaptureFailed)?;
            let color_capacity = particle_ids
                .len()
                .checked_mul(PARTICLE_COLOR_STRIDE)
                .ok_or(SessionError::FrameCaptureFailed)?;
            let mut particle_positions = Vec::with_capacity(position_capacity);
            let mut particle_colors = Vec::with_capacity(color_capacity);
            for position in positions {
                particle_positions.extend_from_slice(&[position.x, position.y]);
            }
            for color in colors {
                particle_colors.extend_from_slice(&color.components());
            }
            let particle_radii = vec![PARTICLE_RADIUS; particle_ids.len()];
            (particle_positions, particle_colors, particle_radii)
        };

        let mut rigid_segments =
            Vec::with_capacity(self.basin_segments.len() * RIGID_SEGMENT_STRIDE);
        for segment in self.basin_segments {
            rigid_segments.extend_from_slice(&[
                segment.start.x,
                segment.start.y,
                segment.end.x,
                segment.end.y,
            ]);
        }
        let circle_position = self
            .world
            .body_snapshot(self.dynamic_circle)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .position();
        let mut rigid_circles = Vec::with_capacity(RIGID_CIRCLE_STRIDE);
        rigid_circles.extend_from_slice(&[
            circle_position.x,
            circle_position.y,
            self.circle_radius,
        ]);

        FrameData::new(
            self.step_index,
            particle_positions,
            particle_colors,
            particle_radii,
            rigid_segments,
            rigid_circles,
        )
        .map_err(|_error| SessionError::FrameCaptureFailed)
    }

    pub(crate) const fn step_index(&self) -> u32 {
        self.step_index
    }

    pub(crate) const fn particle_count(&self) -> usize {
        self.particle_count
    }

    pub(crate) const fn rigid_shape_count(&self) -> usize {
        self.basin_segments.len() + 1
    }
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;

    use super::*;

    fn new_session() -> SessionCore {
        SessionCore::new().expect("fixed proof scene should construct")
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("fixed proof scene should capture"),
        )
    }

    #[test]
    fn construction_creates_exact_bounded_colored_scene() {
        // Arrange
        let session = new_session();

        // Act
        let frame = capture(&session);

        // Assert
        let diagnostics = session.world.world_diagnostics();
        assert_eq!(session.world.gravity().x.to_bits(), 0.0_f32.to_bits());
        assert_eq!(session.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
        assert_eq!(diagnostics.body_count(), 2);
        assert_eq!(diagnostics.fixture_count(), 4);
        assert_eq!(session.world.particle_system_ids().len(), 1);
        let system = session
            .world
            .particle_system_snapshot(session.particle_system)
            .expect("proof particle system should remain live");
        assert_eq!(system.definition().maximum_count(), Some(512));
        assert_eq!(system.particle_count(), 192);
        assert_eq!(frame.step_index(), 0);
        assert_eq!(frame.particle_count(), 192);
        assert_eq!(frame.rigid_shape_count(), 4);
        assert_eq!(frame.rigid_segments().len(), 12);
        assert_eq!(frame.rigid_circles().len(), 3);
        assert_eq!(frame.particle_radii(), vec![0.2; 192].into_boxed_slice());
        assert_eq!(
            frame.particle_colors(),
            [57, 211, 199, 255].repeat(192).into_boxed_slice()
        );
    }

    #[test]
    fn advance_rejects_out_of_range_counts_without_effect() {
        // Arrange
        let rejected_counts = [0, MAX_ADVANCE_STEPS + 1];

        for rejected_count in rejected_counts {
            let mut session = new_session();
            let before = capture(&session).particle_positions();

            // Act
            let result = session.advance(rejected_count);

            // Assert
            assert_eq!(result, Err(SessionError::StepCountOutOfRange));
            assert_eq!(session.step_index(), 0);
            assert_eq!(capture(&session).particle_positions(), before);
        }
    }

    #[test]
    fn advance_increments_step_index_by_accepted_count() {
        // Arrange
        let mut session = new_session();

        // Act
        session
            .advance(MAX_ADVANCE_STEPS)
            .expect("bounded fixed steps should succeed");

        // Assert
        assert_eq!(session.step_index(), MAX_ADVANCE_STEPS);
    }

    #[test]
    fn captured_frame_lanes_are_aligned_finite_and_positive() {
        // Arrange
        let session = new_session();

        // Act
        let frame = capture(&session);

        // Assert
        assert_eq!(frame.particle_positions().len(), 192 * 2);
        assert_eq!(frame.particle_colors().len(), 192 * 4);
        assert_eq!(frame.particle_radii().len(), 192);
        assert!(
            frame
                .particle_positions()
                .iter()
                .all(|value| value.is_finite())
        );
        assert!(
            frame
                .particle_radii()
                .iter()
                .all(|radius| radius.is_finite() && *radius > 0.0)
        );
        assert!(frame.rigid_segments().iter().all(|value| value.is_finite()));
        assert!(frame.rigid_circles().iter().all(|value| value.is_finite()));
    }

    #[test]
    fn four_steps_move_particle_or_dynamic_circle_state() {
        // Arrange
        let mut session = new_session();
        let before = capture(&session);

        // Act
        session
            .advance(MAX_ADVANCE_STEPS)
            .expect("bounded fixed steps should succeed");
        let after = capture(&session);

        // Assert
        assert!(
            before.particle_positions() != after.particle_positions()
                || before.rigid_circles() != after.rigid_circles()
        );
    }

    #[test]
    fn step_index_overflow_is_rejected_before_world_effects() {
        // Arrange
        let mut session = new_session();
        session.step_index = u32::MAX;
        let before = capture(&session).particle_positions();

        // Act
        let result = session.advance(1);

        // Assert
        assert_eq!(result, Err(SessionError::StepIndexExhausted));
        assert_eq!(capture(&session).particle_positions(), before);
    }
}
