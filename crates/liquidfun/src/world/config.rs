use std::error::Error;
use std::fmt;

use crate::math::Vec2;

use super::World;

const MAXIMUM_SOLVER_ITERATIONS: u32 = 1_024;

/// A checked timestep and solver-iteration configuration for one world step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepConfiguration {
    time_step: f32,
    velocity_iterations: u32,
    position_iterations: u32,
}

impl StepConfiguration {
    /// Maximum reviewed velocity-constraint passes accepted per step.
    pub const MAX_VELOCITY_ITERATIONS: u32 = MAXIMUM_SOLVER_ITERATIONS;
    /// Maximum reviewed position-constraint passes accepted per step.
    pub const MAX_POSITION_ITERATIONS: u32 = MAXIMUM_SOLVER_ITERATIONS;

    /// Creates checked timestep and iteration inputs before world effects begin.
    ///
    /// A zero timestep is valid and preserves the world's previous inverse
    /// timestep. Both iteration counts must be within the reviewed positive
    /// range.
    ///
    /// # Errors
    ///
    /// Returns a field-specific error for a non-finite or negative timestep,
    /// or an iteration count outside its reviewed range.
    pub const fn new(
        time_step: f32,
        velocity_iterations: u32,
        position_iterations: u32,
    ) -> Result<Self, StepConfigurationError> {
        if !time_step.is_finite() {
            return Err(StepConfigurationError::NonFiniteTimeStep);
        }
        if time_step < 0.0 {
            return Err(StepConfigurationError::NegativeTimeStep);
        }
        if velocity_iterations == 0 || velocity_iterations > Self::MAX_VELOCITY_ITERATIONS {
            return Err(StepConfigurationError::VelocityIterationsOutOfRange {
                requested: velocity_iterations,
                maximum: Self::MAX_VELOCITY_ITERATIONS,
            });
        }
        if position_iterations == 0 || position_iterations > Self::MAX_POSITION_ITERATIONS {
            return Err(StepConfigurationError::PositionIterationsOutOfRange {
                requested: position_iterations,
                maximum: Self::MAX_POSITION_ITERATIONS,
            });
        }
        Ok(Self {
            time_step,
            velocity_iterations,
            position_iterations,
        })
    }

    /// Returns the accepted timestep in seconds.
    #[must_use]
    pub const fn time_step(self) -> f32 {
        self.time_step
    }

    /// Returns the accepted velocity-constraint pass count.
    #[must_use]
    pub const fn velocity_iterations(self) -> u32 {
        self.velocity_iterations
    }

    /// Returns the accepted position-constraint pass count.
    #[must_use]
    pub const fn position_iterations(self) -> u32 {
        self.position_iterations
    }

    pub(super) fn timing(self, previous_inverse_time_step: f32) -> StepTiming {
        let inverse_time_step = if self.time_step > 0.0 {
            1.0 / self.time_step
        } else {
            0.0
        };
        StepTiming {
            inverse_time_step,
            time_step_ratio: previous_inverse_time_step * self.time_step,
        }
    }
}

/// A failure while constructing checked step inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StepConfigurationError {
    /// Timestep is NaN or infinite.
    NonFiniteTimeStep,
    /// Timestep is finite but negative.
    NegativeTimeStep,
    /// Velocity iterations are zero or exceed the reviewed maximum.
    VelocityIterationsOutOfRange {
        /// Rejected iteration count.
        requested: u32,
        /// Largest accepted iteration count.
        maximum: u32,
    },
    /// Position iterations are zero or exceed the reviewed maximum.
    PositionIterationsOutOfRange {
        /// Rejected iteration count.
        requested: u32,
        /// Largest accepted iteration count.
        maximum: u32,
    },
}

impl fmt::Display for StepConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteTimeStep => formatter.write_str("step timestep must be finite"),
            Self::NegativeTimeStep => formatter.write_str("step timestep must be non-negative"),
            Self::VelocityIterationsOutOfRange { requested, maximum } => write!(
                formatter,
                "velocity iterations must be within 1..={maximum}, got {requested}"
            ),
            Self::PositionIterationsOutOfRange { requested, maximum } => write!(
                formatter,
                "position iterations must be within 1..={maximum}, got {requested}"
            ),
        }
    }
}

impl Error for StepConfigurationError {}

/// Semantic completion state for one successful world step.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum StepCompletion {
    /// Discrete and enabled continuous work completed for this call.
    #[default]
    Complete,
    /// Coherent continuous work remains for a later sub-stepping call.
    ContinuousPending,
}

/// A failure while changing checked world-level configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorldConfigurationError {
    /// The x-coordinate of gravity is not finite.
    NonFiniteGravityX,
    /// The y-coordinate of gravity is not finite.
    NonFiniteGravityY,
    /// A prior hook panic poisoned coherent world operations.
    Poisoned,
    /// The world is inside an active step.
    Locked,
}

impl fmt::Display for WorldConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonFiniteGravityX => "world gravity.x must be finite",
            Self::NonFiniteGravityY => "world gravity.y must be finite",
            Self::Poisoned => "world is poisoned by a prior hook panic",
            Self::Locked => "world is locked by an active step",
        };
        formatter.write_str(message)
    }
}

impl Error for WorldConfigurationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WorldFlags(u8);

impl WorldFlags {
    const WARM_STARTING: u8 = 1 << 0;
    const CONTINUOUS_PHYSICS: u8 = 1 << 1;
    const SUB_STEPPING: u8 = 1 << 2;
    const AUTOMATIC_FORCE_CLEARING: u8 = 1 << 3;

    const fn contains(self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    fn set(&mut self, flag: u8, enabled: bool) {
        if enabled {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct WorldConfiguration {
    gravity: Vec2,
    previous_inverse_time_step: f32,
    flags: WorldFlags,
}

impl Default for WorldConfiguration {
    fn default() -> Self {
        Self {
            gravity: Vec2::ZERO,
            previous_inverse_time_step: 0.0,
            flags: WorldFlags(
                WorldFlags::WARM_STARTING
                    | WorldFlags::CONTINUOUS_PHYSICS
                    | WorldFlags::AUTOMATIC_FORCE_CLEARING,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct StepTiming {
    inverse_time_step: f32,
    time_step_ratio: f32,
}

impl StepTiming {
    pub(super) const fn time_step_ratio(self) -> f32 {
        self.time_step_ratio
    }
}

impl World {
    /// Returns world gravity in meters per second squared.
    #[must_use]
    pub const fn gravity(&self) -> Vec2 {
        self.configuration.gravity
    }

    /// Replaces world gravity after validating both coordinates.
    ///
    /// This control does not wake bodies.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for a non-finite coordinate, poisoned
    /// world, or active step lock.
    pub fn set_gravity(&mut self, gravity: Vec2) -> Result<(), WorldConfigurationError> {
        self.ensure_configuration_mutable()?;
        if !gravity.x.is_finite() {
            return Err(WorldConfigurationError::NonFiniteGravityX);
        }
        if !gravity.y.is_finite() {
            return Err(WorldConfigurationError::NonFiniteGravityY);
        }
        self.configuration.gravity = gravity;
        Ok(())
    }

    /// Returns whether solver warm starting is enabled.
    #[must_use]
    pub const fn is_warm_starting_enabled(&self) -> bool {
        self.configuration.flags.contains(WorldFlags::WARM_STARTING)
    }

    /// Enables or disables solver warm starting.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for a poisoned or locked world.
    pub fn set_warm_starting_enabled(
        &mut self,
        enabled: bool,
    ) -> Result<(), WorldConfigurationError> {
        self.ensure_configuration_mutable()?;
        self.configuration
            .flags
            .set(WorldFlags::WARM_STARTING, enabled);
        Ok(())
    }

    /// Returns whether continuous physics is enabled.
    #[must_use]
    pub const fn is_continuous_physics_enabled(&self) -> bool {
        self.configuration
            .flags
            .contains(WorldFlags::CONTINUOUS_PHYSICS)
    }

    /// Enables or disables continuous physics.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for a poisoned or locked world.
    pub fn set_continuous_physics_enabled(
        &mut self,
        enabled: bool,
    ) -> Result<(), WorldConfigurationError> {
        self.ensure_configuration_mutable()?;
        self.configuration
            .flags
            .set(WorldFlags::CONTINUOUS_PHYSICS, enabled);
        Ok(())
    }

    /// Returns whether one accepted continuous event pauses for sub-stepping.
    #[must_use]
    pub const fn is_sub_stepping_enabled(&self) -> bool {
        self.configuration.flags.contains(WorldFlags::SUB_STEPPING)
    }

    /// Enables or disables single-event continuous sub-stepping.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for a poisoned or locked world.
    pub fn set_sub_stepping_enabled(
        &mut self,
        enabled: bool,
    ) -> Result<(), WorldConfigurationError> {
        self.ensure_configuration_mutable()?;
        self.configuration
            .flags
            .set(WorldFlags::SUB_STEPPING, enabled);
        Ok(())
    }

    /// Returns whether successful steps clear accumulated body forces.
    #[must_use]
    pub const fn is_automatic_force_clearing_enabled(&self) -> bool {
        self.configuration
            .flags
            .contains(WorldFlags::AUTOMATIC_FORCE_CLEARING)
    }

    /// Enables or disables automatic force clearing after successful steps.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for a poisoned or locked world.
    pub fn set_automatic_force_clearing_enabled(
        &mut self,
        enabled: bool,
    ) -> Result<(), WorldConfigurationError> {
        self.ensure_configuration_mutable()?;
        self.configuration
            .flags
            .set(WorldFlags::AUTOMATIC_FORCE_CLEARING, enabled);
        Ok(())
    }

    pub(super) fn prepare_step_timing(&self, configuration: StepConfiguration) -> StepTiming {
        configuration.timing(self.configuration.previous_inverse_time_step)
    }

    pub(super) fn commit_step_timing(&mut self, timing: StepTiming) {
        if timing.inverse_time_step > 0.0 {
            self.configuration.previous_inverse_time_step = timing.inverse_time_step;
        }
    }

    pub(super) fn ensure_configuration_mutable(&self) -> Result<(), WorldConfigurationError> {
        if self.is_poisoned() {
            return Err(WorldConfigurationError::Poisoned);
        }
        if self.is_locked() {
            return Err(WorldConfigurationError::Locked);
        }
        Ok(())
    }
}
