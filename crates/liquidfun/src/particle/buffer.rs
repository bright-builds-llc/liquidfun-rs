//! Safe owned particle-buffer transfer contracts.
//!
//! A bundle owns every supported consumer-supplied lane for its entire stay in
//! a particle system. Fixed bundles use an explicit declared limit and require
//! enough backing allocation up front; growable bundles use an explicit initial
//! allocation target and may grow until the system's checked maximum. Neither
//! mode derives semantic capacity from [`Vec::capacity`].

use std::error::Error;
use std::fmt;

use crate::math::Vec2;
use crate::{
    ArenaInsertError, DestructionRecord, ParticleColor, ParticleFlags, ParticleSystemDefError,
};

const MAX_PARTICLE_COUNT: usize = i32::MAX as usize;

/// The explicit growth contract attached to an owned particle-buffer bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleBufferMode {
    /// The supplied lanes never grow beyond this declared count.
    Fixed {
        /// Maximum number of semantic particle rows.
        capacity: usize,
    },
    /// The supplied lanes may grow beyond this initial allocation target.
    Growable {
        /// Allocation required from every supplied lane at adoption time.
        initial_capacity: usize,
    },
}

impl ParticleBufferMode {
    /// Returns the fixed limit or growable initial allocation target.
    #[must_use]
    pub const fn declared_count(self) -> usize {
        match self {
            Self::Fixed { capacity } => capacity,
            Self::Growable { initial_capacity } => initial_capacity,
        }
    }

    /// Returns whether this contract forbids lane growth.
    #[must_use]
    pub const fn is_fixed(self) -> bool {
        matches!(self, Self::Fixed { .. })
    }
}

/// Complete owned lanes supported by the safe external-buffer equivalent.
///
/// Positions, velocities, and flags are required. Colors remain optional with
/// the same lazy meaning as the pinned implementation. Group membership,
/// application values, identities, and derived solver state are intentionally
/// not consumer-owned lanes.
///
/// Ownership transfer prevents a lane alias from surviving adoption:
///
/// ```compile_fail
/// use liquidfun::math::Vec2;
/// use liquidfun::{ParticleBufferBundle, ParticleBufferLanes, ParticleFlags};
///
/// let lanes = ParticleBufferLanes::new(
///     Vec::<Vec2>::with_capacity(1),
///     Vec::<Vec2>::with_capacity(1),
///     Vec::<ParticleFlags>::with_capacity(1),
///     None,
/// );
/// let positions = lanes.positions();
/// let _bundle = ParticleBufferBundle::fixed(1, lanes).expect("complete fixed lanes");
/// assert!(positions.is_empty());
/// ```
#[derive(Debug, PartialEq)]
pub struct ParticleBufferLanes {
    pub(crate) positions: Vec<Vec2>,
    pub(crate) velocities: Vec<Vec2>,
    pub(crate) flags: Vec<ParticleFlags>,
    pub(crate) maybe_colors: Option<Vec<ParticleColor>>,
}

impl ParticleBufferLanes {
    /// Collects owned required lanes and an optional color lane for validation.
    #[must_use]
    pub const fn new(
        positions: Vec<Vec2>,
        velocities: Vec<Vec2>,
        flags: Vec<ParticleFlags>,
        maybe_colors: Option<Vec<ParticleColor>>,
    ) -> Self {
        Self {
            positions,
            velocities,
            flags,
            maybe_colors,
        }
    }

    /// Returns the current semantic position rows.
    #[must_use]
    pub fn positions(&self) -> &[Vec2] {
        &self.positions
    }

    /// Returns the current semantic velocity rows.
    #[must_use]
    pub fn velocities(&self) -> &[Vec2] {
        &self.velocities
    }

    /// Returns the current exact flag rows.
    #[must_use]
    pub fn flags(&self) -> &[ParticleFlags] {
        &self.flags
    }

    /// Returns the allocated color rows, when this optional lane exists.
    #[must_use]
    pub fn maybe_colors(&self) -> Option<&[ParticleColor]> {
        self.maybe_colors.as_deref()
    }

    /// Clears every required and allocated optional semantic row while retaining allocations.
    ///
    /// This prepares returned lanes for a later adoption cycle without changing
    /// which optional lanes were supplied.
    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.flags.clear();
        if let Some(colors) = &mut self.maybe_colors {
            colors.clear();
        }
    }

    fn row_count(&self) -> Result<usize, ParticleBufferErrorKind> {
        let count = self.positions.len();
        let required_match = self.velocities.len() == count && self.flags.len() == count;
        let optional_match = self
            .maybe_colors
            .as_ref()
            .is_none_or(|colors| colors.len() == count);
        if !required_match || !optional_match {
            return Err(ParticleBufferErrorKind::LaneLengthMismatch);
        }
        Ok(count)
    }

    fn has_capacity(&self, required: usize) -> bool {
        self.positions.capacity() >= required
            && self.velocities.capacity() >= required
            && self.flags.capacity() >= required
            && self
                .maybe_colors
                .as_ref()
                .is_none_or(|colors| colors.capacity() >= required)
    }
}

/// A validated, uniquely owned particle-buffer bundle ready for adoption.
#[derive(Debug, PartialEq)]
pub struct ParticleBufferBundle {
    mode: ParticleBufferMode,
    lanes: ParticleBufferLanes,
}

impl ParticleBufferBundle {
    /// Validates empty lanes for a fixed declared capacity.
    ///
    /// # Errors
    ///
    /// Returns the original lanes with a typed reason when lengths are
    /// inconsistent, lanes already contain rows, capacity is invalid, or any
    /// supplied lane lacks the declared backing allocation.
    pub fn fixed(capacity: usize, lanes: ParticleBufferLanes) -> Result<Self, ParticleBufferError> {
        Self::validate(ParticleBufferMode::Fixed { capacity }, lanes)
    }

    /// Validates empty lanes for a growable initial allocation target.
    ///
    /// # Errors
    ///
    /// Returns the original lanes with a typed reason when lengths are
    /// inconsistent, lanes already contain rows, the target is out of range,
    /// or any supplied lane lacks the declared initial allocation.
    pub fn growable(
        initial_capacity: usize,
        lanes: ParticleBufferLanes,
    ) -> Result<Self, ParticleBufferError> {
        Self::validate(ParticleBufferMode::Growable { initial_capacity }, lanes)
    }

    fn validate(
        mode: ParticleBufferMode,
        lanes: ParticleBufferLanes,
    ) -> Result<Self, ParticleBufferError> {
        let declared_count = mode.declared_count();
        if declared_count > MAX_PARTICLE_COUNT {
            return Err(ParticleBufferError::new(
                ParticleBufferErrorKind::CapacityOutOfRange,
                lanes,
            ));
        }
        if mode.is_fixed() && declared_count == 0 {
            return Err(ParticleBufferError::new(
                ParticleBufferErrorKind::ZeroFixedCapacity,
                lanes,
            ));
        }
        let row_count = match lanes.row_count() {
            Ok(row_count) => row_count,
            Err(kind) => return Err(ParticleBufferError::new(kind, lanes)),
        };
        if row_count != 0 {
            return Err(ParticleBufferError::new(
                ParticleBufferErrorKind::NonEmptyAtAdoption,
                lanes,
            ));
        }
        if !lanes.has_capacity(declared_count) {
            return Err(ParticleBufferError::new(
                ParticleBufferErrorKind::InsufficientLaneCapacity {
                    required: declared_count,
                },
                lanes,
            ));
        }
        Ok(Self { mode, lanes })
    }

    /// Returns the explicit fixed/growable contract.
    #[must_use]
    pub const fn mode(&self) -> ParticleBufferMode {
        self.mode
    }

    /// Returns a borrow of the currently owned lanes.
    #[must_use]
    pub const fn lanes(&self) -> &ParticleBufferLanes {
        &self.lanes
    }

    /// Returns all uniquely owned lanes.
    #[must_use]
    pub fn into_lanes(self) -> ParticleBufferLanes {
        self.lanes
    }

    pub(crate) fn into_parts(self) -> (ParticleBufferMode, ParticleBufferLanes) {
        (self.mode, self.lanes)
    }

    pub(crate) const fn from_storage(mode: ParticleBufferMode, lanes: ParticleBufferLanes) -> Self {
        Self { mode, lanes }
    }
}

/// Stable validation categories for an owned particle-buffer bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleBufferErrorKind {
    /// Required or allocated optional lanes do not describe the same rows.
    LaneLengthMismatch,
    /// Construction-time adoption requires empty lanes.
    NonEmptyAtAdoption,
    /// A fixed bundle cannot declare zero capacity.
    ZeroFixedCapacity,
    /// The declared count cannot be represented by the pinned `int32` count.
    CapacityOutOfRange,
    /// At least one supplied lane cannot hold the declared initial rows without allocation.
    InsufficientLaneCapacity {
        /// Required initial backing allocation.
        required: usize,
    },
}

impl fmt::Display for ParticleBufferErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LaneLengthMismatch => formatter.write_str("particle buffer lane lengths differ"),
            Self::NonEmptyAtAdoption => {
                formatter.write_str("particle buffers must be empty when adopted")
            }
            Self::ZeroFixedCapacity => {
                formatter.write_str("fixed particle buffer capacity must be positive")
            }
            Self::CapacityOutOfRange => {
                formatter.write_str("particle buffer capacity exceeds the pinned count range")
            }
            Self::InsufficientLaneCapacity { required } => write!(
                formatter,
                "a supplied particle lane has less than the required capacity of {required}"
            ),
        }
    }
}

/// Validation failure that returns ownership of every supplied lane.
#[derive(Debug, PartialEq)]
pub struct ParticleBufferError {
    kind: ParticleBufferErrorKind,
    lanes: ParticleBufferLanes,
}

impl ParticleBufferError {
    const fn new(kind: ParticleBufferErrorKind, lanes: ParticleBufferLanes) -> Self {
        Self { kind, lanes }
    }

    /// Returns the stable validation category.
    #[must_use]
    pub const fn kind(&self) -> ParticleBufferErrorKind {
        self.kind
    }

    /// Returns ownership of all rejected lanes.
    #[must_use]
    pub fn into_lanes(self) -> ParticleBufferLanes {
        self.lanes
    }
}

impl fmt::Display for ParticleBufferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(formatter)
    }
}

impl Error for ParticleBufferError {}

/// Why a validated bundle could not be adopted by a world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleBufferAdoptionErrorKind {
    /// The bundle's fixed capacity conflicts with the system maximum.
    Definition(ParticleSystemDefError),
    /// The world could not allocate the particle-system owner.
    World(ArenaInsertError),
}

/// Failed world adoption with the still-owned validated bundle attached.
#[derive(Debug, PartialEq)]
pub struct ParticleBufferAdoptionError {
    kind: ParticleBufferAdoptionErrorKind,
    bundle: Box<ParticleBufferBundle>,
}

impl ParticleBufferAdoptionError {
    pub(crate) fn new(kind: ParticleBufferAdoptionErrorKind, bundle: ParticleBufferBundle) -> Self {
        Self {
            kind,
            bundle: Box::new(bundle),
        }
    }

    /// Returns the stable adoption category.
    #[must_use]
    pub const fn kind(&self) -> ParticleBufferAdoptionErrorKind {
        self.kind
    }

    /// Returns ownership of the validated bundle after the no-effect failure.
    #[must_use]
    pub fn into_bundle(self) -> ParticleBufferBundle {
        *self.bundle
    }
}

impl fmt::Display for ParticleBufferAdoptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ParticleBufferAdoptionErrorKind::Definition(error) => error.fmt(formatter),
            ParticleBufferAdoptionErrorKind::World(error) => error.fmt(formatter),
        }
    }
}

impl Error for ParticleBufferAdoptionError {}

/// Owned destruction evidence and particle lanes returned from system teardown.
#[derive(Debug, PartialEq)]
pub struct ParticleBufferTeardown {
    records: Vec<DestructionRecord>,
    bundle: ParticleBufferBundle,
}

impl ParticleBufferTeardown {
    pub(crate) const fn new(records: Vec<DestructionRecord>, bundle: ParticleBufferBundle) -> Self {
        Self { records, bundle }
    }

    /// Returns the complete ordered particle-system destruction evidence.
    #[must_use]
    pub fn records(&self) -> &[DestructionRecord] {
        &self.records
    }

    /// Returns the fixed/growable contract used by the destroyed system.
    #[must_use]
    pub const fn mode(&self) -> ParticleBufferMode {
        self.bundle.mode()
    }

    /// Returns the final semantic contents in every owned supplied lane.
    #[must_use]
    pub fn into_lanes(self) -> ParticleBufferLanes {
        self.bundle.into_lanes()
    }

    /// Returns destruction evidence and the complete validated buffer bundle.
    #[must_use]
    pub fn into_parts(self) -> (Vec<DestructionRecord>, ParticleBufferBundle) {
        (self.records, self.bundle)
    }
}
