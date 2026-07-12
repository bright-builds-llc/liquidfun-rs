use std::error::Error;
use std::fmt;

use crate::collision::{FilterData, Shape};
use crate::{BodyId, HandleError};

use super::body::AggregateMassError;

/// A failure while deriving or extending fixture bounds for broad-phase storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixtureBoundsError {
    /// A checked shape and body transform produced a non-finite child AABB.
    NonFiniteDerivedBounds,
    /// Broad-phase fattening or displacement prediction overflowed finite coordinates.
    BroadPhaseOverflow,
}

impl fmt::Display for FixtureBoundsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonFiniteDerivedBounds => "fixture transform produced non-finite bounds",
            Self::BroadPhaseOverflow => "fixture broad-phase bounds overflowed",
        };
        formatter.write_str(message)
    }
}

impl Error for FixtureBoundsError {}

/// A failure while constructing a checked [`FixtureDef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixtureDefError {
    /// Density is not finite.
    NonFiniteDensity,
    /// Friction is not finite.
    NonFiniteFriction,
    /// Restitution is not finite.
    NonFiniteRestitution,
    /// Density is negative.
    NegativeDensity,
    /// Friction is negative.
    NegativeFriction,
    /// Restitution is negative.
    NegativeRestitution,
}

impl fmt::Display for FixtureDefError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonFiniteDensity => "fixture density must be finite",
            Self::NonFiniteFriction => "fixture friction must be finite",
            Self::NonFiniteRestitution => "fixture restitution must be finite",
            Self::NegativeDensity => "fixture density must be non-negative",
            Self::NegativeFriction => "fixture friction must be non-negative",
            Self::NegativeRestitution => "fixture restitution must be non-negative",
        };
        formatter.write_str(message)
    }
}

impl Error for FixtureDefError {}

/// A failure while changing checked material state on a world-owned fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixtureMutationError {
    /// The fixture identity does not resolve in this world.
    InvalidHandle(HandleError),
    /// The requested material value is invalid.
    InvalidValue(FixtureDefError),
    /// The requested density produces non-finite shape mass properties.
    InvalidDerivedMass,
}

impl fmt::Display for FixtureMutationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid fixture handle: {error}"),
            Self::InvalidValue(error) => write!(formatter, "invalid fixture value: {error}"),
            Self::InvalidDerivedMass => {
                formatter.write_str("fixture density produces invalid mass properties")
            }
        }
    }
}

impl Error for FixtureMutationError {}

impl From<HandleError> for FixtureMutationError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<FixtureDefError> for FixtureMutationError {
    fn from(error: FixtureDefError) -> Self {
        Self::InvalidValue(error)
    }
}

/// A failure while explicitly destroying a world-owned fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixtureDestructionError {
    /// The fixture identity does not resolve in this world.
    InvalidHandle(HandleError),
    /// The parent body's complete remaining-fixture aggregate is invalid.
    InvalidAggregateMass(AggregateMassError),
}

impl fmt::Display for FixtureDestructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid fixture handle: {error}"),
            Self::InvalidAggregateMass(error) => {
                write!(formatter, "invalid aggregate body mass: {error}")
            }
        }
    }
}

impl Error for FixtureDestructionError {}

impl From<HandleError> for FixtureDestructionError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<AggregateMassError> for FixtureDestructionError {
    fn from(error: AggregateMassError) -> Self {
        Self::InvalidAggregateMass(error)
    }
}

/// A reusable checked fixture definition with an owned immutable shape.
///
/// Density is kilograms per square meter. Friction and restitution are
/// dimensionless non-negative coefficients. Accepted values retain their exact
/// `f32` bit patterns and are never clamped. The definition exposes semantic
/// state only; broad-phase and other world-owned storage remain private.
///
/// Definition fields cannot be changed without constructing another checked
/// value:
///
/// ```compile_fail
/// use liquidfun::collision::{CircleShape, FilterData, Shape};
/// use liquidfun::math::Vec2;
/// use liquidfun::FixtureDef;
///
/// let shape = Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("valid circle"));
/// let mut definition = FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
///     .expect("valid fixture definition");
/// definition.density = 2.0;
/// ```
///
/// World-owned implementation storage is not exposed through a definition:
///
/// ```compile_fail
/// use liquidfun::collision::{CircleShape, FilterData, Shape};
/// use liquidfun::math::Vec2;
/// use liquidfun::FixtureDef;
///
/// let shape = Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("valid circle"));
/// let definition = FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
///     .expect("valid fixture definition");
/// let _storage = definition.proxies;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FixtureDef {
    shape: Shape,
    density: f32,
    friction: f32,
    restitution: f32,
    sensor: bool,
    filter_data: FilterData,
}

impl FixtureDef {
    /// Creates a reusable fixture definition with complete Phase 6 material state.
    ///
    /// # Errors
    ///
    /// Returns a field-specific error when a material coefficient is non-finite
    /// or negative.
    #[must_use = "fixture-definition construction can fail for invalid material values"]
    pub fn new(
        shape: Shape,
        density: f32,
        friction: f32,
        restitution: f32,
        sensor: bool,
        filter_data: FilterData,
    ) -> Result<Self, FixtureDefError> {
        validate_fixture_material(density, friction, restitution)?;
        Ok(Self {
            shape,
            density,
            friction,
            restitution,
            sensor,
            filter_data,
        })
    }

    /// Returns the immutable owned shape snapshot.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Returns density in kilograms per square meter.
    #[must_use]
    pub const fn density(&self) -> f32 {
        self.density
    }

    /// Returns the friction coefficient.
    #[must_use]
    pub const fn friction(&self) -> f32 {
        self.friction
    }

    /// Returns the restitution coefficient.
    #[must_use]
    pub const fn restitution(&self) -> f32 {
        self.restitution
    }

    /// Returns whether the fixture reports overlap without collision response.
    #[must_use]
    pub const fn is_sensor(&self) -> bool {
        self.sensor
    }

    /// Returns the complete collision-filter value.
    #[must_use]
    pub const fn filter_data(&self) -> FilterData {
        self.filter_data
    }

    /// Returns an owned semantic snapshot of this definition.
    #[must_use]
    pub fn snapshot(&self) -> FixtureSnapshot {
        FixtureSnapshot {
            shape: self.shape.clone(),
            density: self.density,
            friction: self.friction,
            restitution: self.restitution,
            sensor: self.sensor,
            filter_data: self.filter_data,
        }
    }

    pub(super) fn set_density(&mut self, density: f32) -> Result<(), FixtureDefError> {
        validate_density(density)?;
        self.density = density;
        Ok(())
    }

    pub(super) fn set_friction(&mut self, friction: f32) -> Result<(), FixtureDefError> {
        validate_friction(friction)?;
        self.friction = friction;
        Ok(())
    }

    pub(super) fn set_restitution(&mut self, restitution: f32) -> Result<(), FixtureDefError> {
        validate_restitution(restitution)?;
        self.restitution = restitution;
        Ok(())
    }

    pub(super) fn set_sensor(&mut self, sensor: bool) {
        self.sensor = sensor;
    }

    pub(super) fn set_filter_data(&mut self, filter_data: FilterData) {
        self.filter_data = filter_data;
    }
}

/// Owned semantic fixture state without mutable topology or world authority.
#[derive(Debug, Clone, PartialEq)]
pub struct FixtureSnapshot {
    shape: Shape,
    density: f32,
    friction: f32,
    restitution: f32,
    sensor: bool,
    filter_data: FilterData,
}

impl Eq for FixtureSnapshot {}

impl FixtureSnapshot {
    /// Returns the captured immutable shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Returns density in kilograms per square meter.
    #[must_use]
    pub const fn density(&self) -> f32 {
        self.density
    }

    /// Returns the captured friction coefficient.
    #[must_use]
    pub const fn friction(&self) -> f32 {
        self.friction
    }

    /// Returns the captured restitution coefficient.
    #[must_use]
    pub const fn restitution(&self) -> f32 {
        self.restitution
    }

    /// Returns whether the captured fixture is a sensor.
    #[must_use]
    pub const fn is_sensor(&self) -> bool {
        self.sensor
    }

    /// Returns the captured collision filter.
    #[must_use]
    pub const fn filter_data(&self) -> FilterData {
        self.filter_data
    }
}

/// Owned semantic state for one fixture attached to a world-owned body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldFixtureSnapshot {
    body: BodyId,
    definition: FixtureSnapshot,
    broad_phase_entry_count: usize,
}

impl WorldFixtureSnapshot {
    pub(super) fn from_definition(
        body: BodyId,
        definition: &FixtureDef,
        broad_phase_entry_count: usize,
    ) -> Self {
        Self {
            body,
            definition: definition.snapshot(),
            broad_phase_entry_count,
        }
    }

    /// Returns the fixture's owning body identity.
    #[must_use]
    pub const fn body(&self) -> BodyId {
        self.body
    }

    /// Returns the captured immutable shape.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        self.definition.shape()
    }

    /// Returns density in kilograms per square meter.
    #[must_use]
    pub const fn density(&self) -> f32 {
        self.definition.density()
    }

    /// Returns the captured friction coefficient.
    #[must_use]
    pub const fn friction(&self) -> f32 {
        self.definition.friction()
    }

    /// Returns the captured restitution coefficient.
    #[must_use]
    pub const fn restitution(&self) -> f32 {
        self.definition.restitution()
    }

    /// Returns whether the captured fixture is a sensor.
    #[must_use]
    pub const fn is_sensor(&self) -> bool {
        self.definition.is_sensor()
    }

    /// Returns the captured collision filter.
    #[must_use]
    pub const fn filter_data(&self) -> FilterData {
        self.definition.filter_data()
    }

    /// Returns the number of shape children currently participating in broad-phase discovery.
    #[must_use]
    pub const fn broad_phase_entry_count(&self) -> usize {
        self.broad_phase_entry_count
    }
}

fn validate_fixture_material(
    density: f32,
    friction: f32,
    restitution: f32,
) -> Result<(), FixtureDefError> {
    validate_density(density)?;
    validate_friction(friction)?;
    validate_restitution(restitution)?;
    Ok(())
}

fn validate_density(density: f32) -> Result<(), FixtureDefError> {
    if !density.is_finite() {
        return Err(FixtureDefError::NonFiniteDensity);
    }
    if density < 0.0 {
        return Err(FixtureDefError::NegativeDensity);
    }
    Ok(())
}

fn validate_friction(friction: f32) -> Result<(), FixtureDefError> {
    if !friction.is_finite() {
        return Err(FixtureDefError::NonFiniteFriction);
    }
    if friction < 0.0 {
        return Err(FixtureDefError::NegativeFriction);
    }
    Ok(())
}

fn validate_restitution(restitution: f32) -> Result<(), FixtureDefError> {
    if !restitution.is_finite() {
        return Err(FixtureDefError::NonFiniteRestitution);
    }
    if restitution < 0.0 {
        return Err(FixtureDefError::NegativeRestitution);
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn test_fixture_definition() -> FixtureDef {
    use crate::collision::shape::CircleShape;
    use crate::math::Vec2;

    let shape = Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid"));
    FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
        .expect("fixture definition should be valid")
}
