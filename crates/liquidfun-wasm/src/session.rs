//! Native-testable ownership and stepping for one allowlisted scene.

use liquidfun::math::Vec2;
use liquidfun::{
    NoDecisionHook, ParticleSystemId, ParticleSystemSnapshot, StepConfiguration, StepLimits, World,
};

#[cfg(not(target_arch = "wasm32"))]
use liquidfun::DiagnosticStepProfile;

use self::escape::evict_escaped_particles;
use crate::frame::{FrameData, FrameDiagnostics};
use crate::scene::{
    BuiltScene, ControlEffect, SceneHooks, SceneId, build_scene, parse_pointer_kind,
};

pub(crate) const MAX_ADVANCE_STEPS: u32 = 4;
const MAX_FRAME_PARTICLES: usize = 16_384;
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
    last_failure_detail: String,
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
            .with_particle_iterations(crate::scene::particle_iterations(id))
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
            last_failure_detail: String::new(),
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
            if let Err(detail) =
                evict_escaped_particles(&mut self.world, self.particle_system, self.particle_radius)
            {
                self.last_failure_detail = detail;
                return Err(SessionError::StepFailed);
            }
            let stepped = self.world.step(
                self.step_configuration,
                &mut NoDecisionHook,
                self.step_limits,
            );
            if let Err(error) = stepped {
                self.last_failure_detail = error.to_string();
                return Err(SessionError::StepFailed);
            }
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
        evict_escaped_particles(&mut self.world, self.particle_system, self.particle_radius)
            .map_err(|detail| {
                self.last_failure_detail = detail;
                SessionError::StepFailed
            })?;
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

    pub(crate) fn failure_detail(&self) -> &str {
        &self.last_failure_detail
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

mod escape;

#[cfg(test)]
mod tests;
