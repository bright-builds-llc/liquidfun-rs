//! Native-testable ownership and stepping for one allowlisted scene.

use liquidfun::math::Vec2;
use liquidfun::{
    NoDecisionHook, ParticleSystemId, ParticleSystemSnapshot, StepConfiguration, StepLimits, World,
};

#[cfg(not(target_arch = "wasm32"))]
use liquidfun::DiagnosticStepProfile;

use crate::frame::{FrameData, FrameDiagnostics};
use crate::scene::{
    BuiltScene, ControlEffect, SceneHooks, SceneId, build_scene, parse_pointer_kind,
};

pub(crate) const MAX_ADVANCE_STEPS: u32 = 4;
const MAX_FRAME_PARTICLES: usize = 10240;
const PARTICLE_POSITION_STRIDE: usize = 2;
const PARTICLE_COLOR_STRIDE: usize = 4;
const RIGID_SEGMENT_STRIDE: usize = 4;
const RIGID_CIRCLE_STRIDE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionError {
    SceneConstruction,
    UnknownScene,
    #[allow(dead_code)] // Retained for fail-closed unimplemented scene construction.
    SceneUnimplemented,
    UnknownControl,
    InvalidPointer,
    StepCountOutOfRange,
    StepIndexExhausted,
    StepFailed,
    FrameCaptureFailed,
    InvalidGravity,
}

impl SessionError {
    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::SceneConstruction => "Rust/WASM scene construction failed",
            Self::UnknownScene => "Rust/WASM scene id is not allowlisted",
            Self::SceneUnimplemented => "Rust/WASM scene is not implemented",
            Self::UnknownControl => "Rust/WASM control is not allowlisted",
            Self::InvalidPointer => "Rust/WASM pointer coordinates must be finite",
            Self::StepCountOutOfRange => "Rust/WASM step count must be within 1 through 4",
            Self::StepIndexExhausted => "Rust/WASM step index exhausted",
            Self::StepFailed => "Rust/WASM simulation step failed",
            Self::FrameCaptureFailed => "Rust/WASM frame capture failed",
            Self::InvalidGravity => "Rust/WASM gravity coordinates must be finite",
        }
    }
}

pub(crate) struct SessionCore {
    world: World,
    particle_system: ParticleSystemId,
    hooks: Box<dyn SceneHooks>,
    particle_radius: f32,
    #[cfg_attr(not(test), allow(dead_code))]
    presets: Vec<(String, String)>,
    #[cfg_attr(not(test), allow(dead_code))]
    scene_id: SceneId,
    particle_count: usize,
    step_configuration: StepConfiguration,
    step_limits: StepLimits,
    step_index: u32,
    authored_gravity: Vec2,
}

fn max_particle_speed(velocities: &[Vec2]) -> Result<f32, SessionError> {
    let mut max_squared = 0.0_f32;
    for velocity in velocities {
        if !velocity.is_valid() {
            return Err(SessionError::FrameCaptureFailed);
        }
        let squared = velocity.length_squared();
        if squared > max_squared {
            max_squared = squared;
        }
    }

    let speed = max_squared.sqrt();
    if !speed.is_finite() {
        return Err(SessionError::FrameCaptureFailed);
    }

    Ok(speed)
}

impl SessionCore {
    pub(crate) fn create(id: SceneId) -> Result<Self, SessionError> {
        Self::from_built(id, Vec::new())
    }

    fn from_built(id: SceneId, presets: Vec<(String, String)>) -> Result<Self, SessionError> {
        let BuiltScene {
            world,
            particle_system,
            particle_radius,
            hooks,
        } = build_scene(id, &presets)?;
        let particle_count = world
            .particle_system_snapshot(particle_system)
            .map_err(|_error| SessionError::SceneConstruction)?
            .particle_count();
        let step_configuration = StepConfiguration::new(1.0 / 60.0, 8, 3)
            .map_err(|_error| SessionError::SceneConstruction)?
            .with_particle_iterations(2)
            .map_err(|_error| SessionError::SceneConstruction)?;
        let authored_gravity = world.gravity();

        Ok(Self {
            world,
            particle_system,
            hooks,
            particle_radius,
            presets,
            scene_id: id,
            particle_count,
            step_configuration,
            step_limits: StepLimits::default(),
            step_index: 0,
            authored_gravity,
        })
    }

    pub(crate) fn set_gravity(&mut self, x: f32, y: f32) -> Result<(), SessionError> {
        self.world
            .set_gravity(Vec2::new(x, y))
            .map_err(|_error| SessionError::InvalidGravity)
    }

    pub(crate) fn restore_authored_gravity(&mut self) -> Result<(), SessionError> {
        let gravity = self.authored_gravity;
        self.set_gravity(gravity.x, gravity.y)
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
            self.hooks
                .on_advance(&mut self.world, self.particle_system)?;
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

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn advance_profiled(&mut self) -> Result<DiagnosticStepProfile, SessionError> {
        let next_step_index = self
            .step_index
            .checked_add(1)
            .ok_or(SessionError::StepIndexExhausted)?;
        self.hooks
            .on_advance(&mut self.world, self.particle_system)?;
        let profile = self
            .world
            .step_profiled(
                self.step_configuration,
                &mut NoDecisionHook,
                self.step_limits,
            )
            .map(|(_report, profile)| profile)
            .map_err(|_error| SessionError::StepFailed)?;
        self.step_index = next_step_index;
        Ok(profile)
    }

    pub(crate) fn apply_control(&mut self, name: &str, value: &str) -> Result<bool, SessionError> {
        let effect =
            self.hooks
                .apply_control(&mut self.world, self.particle_system, name, value)?;
        match effect {
            ControlEffect::Live => Ok(false),
            ControlEffect::Recreated => {
                store_preset(&mut self.presets, name, value);
                *self = Self::from_built(self.scene_id, self.presets.clone())?;
                Ok(true)
            }
        }
    }

    pub(crate) fn apply_action(&mut self, name: &str) -> Result<(), SessionError> {
        self.hooks
            .apply_action(&mut self.world, self.particle_system, name)
    }

    pub(crate) fn apply_pointer(
        &mut self,
        kind: &str,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), SessionError> {
        let parsed = parse_pointer_kind(kind)?;
        if !world_x.is_finite() || !world_y.is_finite() {
            return Err(SessionError::InvalidPointer);
        }
        self.hooks.apply_pointer(
            &mut self.world,
            self.particle_system,
            parsed,
            world_x,
            world_y,
        )
    }

    pub(crate) fn capture_frame(&self) -> Result<FrameData, SessionError> {
        let (particle_positions, particle_colors, particle_radii, max_speed) = {
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
            let velocities = view.velocities();
            if velocities.len() != particle_ids.len() {
                return Err(SessionError::FrameCaptureFailed);
            }
            let max_speed = max_particle_speed(velocities)?;
            let particle_radii = vec![self.particle_radius; particle_ids.len()];
            (
                particle_positions,
                particle_colors,
                particle_radii,
                max_speed,
            )
        };
        let statistics = self
            .world
            .particle_system_statistics(self.particle_system)
            .map_err(|_error| SessionError::FrameCaptureFailed)?;
        let diagnostics = FrameDiagnostics {
            max_speed,
            stuck_candidate_count: statistics.stuck_candidates().len(),
            body_contact_count: statistics.body_contact_count(),
        };

        let segments = self.hooks.collect_segments(&self.world)?;
        let mut rigid_segments = Vec::with_capacity(
            segments
                .len()
                .checked_mul(RIGID_SEGMENT_STRIDE)
                .ok_or(SessionError::FrameCaptureFailed)?,
        );
        for segment in segments {
            rigid_segments.extend_from_slice(&[
                segment.start.x,
                segment.start.y,
                segment.end.x,
                segment.end.y,
            ]);
        }

        let circles = self.hooks.collect_circles(&self.world)?;
        let circle_count = circles.len();
        let mut rigid_circles = Vec::with_capacity(
            circle_count
                .checked_mul(RIGID_CIRCLE_STRIDE)
                .ok_or(SessionError::FrameCaptureFailed)?,
        );
        for (position, radius) in circles {
            rigid_circles.extend_from_slice(&[position.x, position.y, radius]);
        }

        let mut frame = FrameData::new(
            self.step_index,
            particle_positions,
            particle_colors,
            particle_radii,
            rigid_segments,
            rigid_circles,
            diagnostics,
        )
        .map_err(|_error| SessionError::FrameCaptureFailed)?;
        let mut labels = self.hooks.collect_circle_labels(&self.world)?;
        if labels.is_empty() {
            labels.resize(circle_count, String::new());
        }
        frame
            .set_circle_labels(labels)
            .map_err(|_error| SessionError::FrameCaptureFailed)?;
        Ok(frame)
    }

    pub(crate) const fn step_index(&self) -> u32 {
        self.step_index
    }

    pub(crate) const fn particle_count(&self) -> usize {
        self.particle_count
    }

    pub(crate) fn live_particle_count(&self) -> Result<usize, SessionError> {
        self.world
            .particle_system_snapshot(self.particle_system)
            .map_err(|_error| SessionError::SceneConstruction)
            .map(ParticleSystemSnapshot::particle_count)
    }

    pub(crate) fn rigid_shape_count(&self) -> usize {
        let segment_count = self
            .hooks
            .collect_segments(&self.world)
            .map_or(0, |segments| segments.len());
        let circle_count = self
            .hooks
            .collect_circles(&self.world)
            .map_or(0, |circles| circles.len());
        segment_count.saturating_add(circle_count)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn store_preset(presets: &mut Vec<(String, String)>, name: &str, value: &str) {
    if let Some(existing) = presets.iter_mut().find(|(key, _value)| key == name) {
        value.clone_into(&mut existing.1);
        return;
    }
    presets.push((name.to_owned(), value.to_owned()));
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::{SceneId, parse_scene_id};

    use super::*;

    fn new_session() -> SessionCore {
        SessionCore::create(SceneId::DamBreak).expect("fixed proof scene should construct")
    }

    #[test]
    fn parse_scene_id_maps_allowlisted_tokens() {
        // Arrange
        let tokens = [
            ("dam-break", SceneId::DamBreak),
            ("fountain", SceneId::Fountain),
            ("float-or-sink", SceneId::FloatOrSink),
            ("color-mixer", SceneId::ColorMixer),
            ("jelly-drop", SceneId::JellyDrop),
            ("water-wheel", SceneId::WaterWheel),
            ("particles", SceneId::Particles),
            ("liquid-timer", SceneId::LiquidTimer),
        ];

        for (raw, expected) in tokens {
            // Act
            let parsed = parse_scene_id(raw);

            // Assert
            assert_eq!(parsed, Ok(expected));
        }
    }

    #[test]
    fn parse_scene_id_rejects_unknown_tokens() {
        // Arrange
        let rejected = ["Dam-Break", "", "not-a-scene"];

        for raw in rejected {
            // Act
            let parsed = parse_scene_id(raw);

            // Assert
            assert_eq!(parsed, Err(SessionError::UnknownScene));
            assert_eq!(
                SessionError::UnknownScene.message(),
                "Rust/WASM scene id is not allowlisted"
            );
        }
    }

    #[test]
    fn capture_uses_the_same_particle_cap_as_copied_frames() {
        // Arrange / Act / Assert
        assert_eq!(MAX_FRAME_PARTICLES, 10240);
    }

    #[test]
    fn create_dam_break_matches_documented_medium_normal_world() {
        // Arrange
        let session =
            SessionCore::create(SceneId::DamBreak).expect("allowlisted Dam Break should construct");

        // Act
        let frame = capture(&session);

        // Assert
        let diagnostics = session.world.world_diagnostics();
        assert_eq!(session.world.gravity().x.to_bits(), 0.0_f32.to_bits());
        assert_eq!(session.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
        assert_eq!(diagnostics.body_count(), 2);
        assert_eq!(diagnostics.fixture_count(), 4);
        assert_eq!(session.particle_count(), 1920);
        assert_eq!(session.rigid_shape_count(), 4);
        assert_eq!(frame.particle_count(), 1920);
        assert_eq!(
            frame.particle_colors(),
            [77, 163, 255, 255].repeat(1920).into_boxed_slice()
        );
        assert_eq!(frame.max_speed().to_bits(), 0.0_f32.to_bits());
        assert_eq!(frame.stuck_candidate_count(), 0);
    }

    #[test]
    fn capture_reports_a_finite_speed_after_one_step() {
        // Arrange
        let mut session =
            SessionCore::create(SceneId::DamBreak).expect("allowlisted Dam Break should construct");

        // Act
        session.advance(1).expect("one step should succeed");
        let frame = capture(&session);

        // Assert
        assert!(frame.max_speed() > 0.0);
        assert!(frame.max_speed().is_finite());
        assert!(frame.stuck_candidate_count() <= frame.particle_count());
    }

    #[test]
    fn fountain_live_particle_count_grows_after_advance() {
        // Arrange
        let mut session =
            SessionCore::create(SceneId::Fountain).expect("allowlisted Fountain should construct");
        let start = session
            .live_particle_count()
            .expect("Fountain snapshot should succeed");

        // Act
        session
            .advance(1)
            .expect("Fountain emission advance should succeed");
        let end = session
            .live_particle_count()
            .expect("Fountain snapshot after emit should succeed");

        // Assert
        assert!(end > start);
        assert_eq!(session.particle_count(), start);
    }

    #[test]
    fn create_color_mixer_constructs_a_live_world() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::ColorMixer)
            .expect("allowlisted Color Mixer should construct");

        // Assert
        assert!((400..=2200).contains(&session.particle_count()));
    }

    #[test]
    fn dam_break_unknown_control_and_action_fail_closed() {
        // Arrange
        let mut session =
            SessionCore::create(SceneId::DamBreak).expect("allowlisted Dam Break should construct");

        // Act
        let control = session.apply_control("nope", "x");
        let action = session.apply_action("nope");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
        assert_eq!(
            SessionError::UnknownControl.message(),
            "Rust/WASM control is not allowlisted"
        );
        assert_eq!(session.particle_count(), 1920);
        assert_eq!(session.scene_id, SceneId::DamBreak);
        assert!(session.presets.is_empty());
    }

    #[test]
    fn store_preset_replaces_matching_name() {
        // Arrange
        let mut presets = vec![("water".to_owned(), "medium".to_owned())];

        // Act
        store_preset(&mut presets, "water", "large");
        store_preset(&mut presets, "gravity", "normal");

        // Assert
        assert_eq!(
            presets,
            vec![
                ("water".to_owned(), "large".to_owned()),
                ("gravity".to_owned(), "normal".to_owned()),
            ]
        );
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
        assert_eq!(system.definition().maximum_count(), Some(10240));
        assert_eq!(system.particle_count(), 1920);
        assert_eq!(frame.step_index(), 0);
        assert_eq!(frame.particle_count(), 1920);
        assert_eq!(frame.rigid_shape_count(), 4);
        assert_eq!(frame.rigid_segments().len(), 12);
        assert_eq!(frame.rigid_circles().len(), 3);
        assert_eq!(
            frame.particle_radii(),
            vec![0.063_245_55; 1920].into_boxed_slice()
        );
        assert_eq!(
            frame.particle_colors(),
            [77, 163, 255, 255].repeat(1920).into_boxed_slice()
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
        assert_eq!(frame.particle_positions().len(), 1920 * 2);
        assert_eq!(frame.particle_colors().len(), 1920 * 4);
        assert_eq!(frame.particle_radii().len(), 1920);
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

    #[test]
    fn set_gravity_overrides_until_authored_gravity_is_restored() {
        // Arrange
        let mut session = new_session();
        let built = session.world.gravity();

        // Act
        session
            .set_gravity(2.5, -1.5)
            .expect("finite gravity applies");
        let overridden = session.world.gravity();
        session
            .restore_authored_gravity()
            .expect("authored gravity restores");
        let restored = session.world.gravity();

        // Assert
        assert_eq!(overridden.x.to_bits(), 2.5_f32.to_bits());
        assert_eq!(overridden.y.to_bits(), (-1.5_f32).to_bits());
        assert_eq!(restored.x.to_bits(), built.x.to_bits());
        assert_eq!(restored.y.to_bits(), built.y.to_bits());
        assert!(session.set_gravity(f32::NAN, 0.0).is_err());
    }
}
