//! Safe owned particle-group recipes and borrow-scoped group inspection.

#[allow(
    dead_code,
    reason = "consumed by the Phase 10 world-facing particle-group integration"
)]
pub(crate) mod sampling;

use std::error::Error;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use crate::collision::Shape;
use crate::math::{Transform, Vec2};
use crate::{ParticleColor, ParticleFlags, ParticleGroupId, ParticleId};

const MAX_UPSTREAM_COUNT: usize = i32::MAX as usize;
const PRIVATE_GROUP_FLAG_MASK: u32 = 0x0018;

/// Public particle-group behavior flags with the pinned upstream bit values.
///
/// Unknown public bits are retained by [`Self::from_bits_retain`] for the same
/// forward-compatible round-trip policy as [`ParticleFlags`]. The two
/// upstream-private lifecycle/cache bits are always removed and cannot cross
/// this public boundary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ParticleGroupFlags(u32);

impl ParticleGroupFlags {
    /// Prevent other particles from overlapping or leaking through the group.
    pub const SOLID: Self = Self(0x0001);
    /// Preserve the group's shape through rigid particle motion.
    pub const RIGID: Self = Self(0x0002);
    /// Retain the group identity when its final particle is removed.
    pub const CAN_BE_EMPTY: Self = Self(0x0004);

    /// Returns a value with no public group behavior enabled.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns the union of all named public group behaviors.
    #[must_use]
    pub const fn all() -> Self {
        Self(Self::SOLID.0 | Self::RIGID.0 | Self::CAN_BE_EMPTY.0)
    }

    /// Creates flags while retaining unknown public bits.
    ///
    /// Upstream-private lifecycle/cache bits are stripped even when present in
    /// the input, so they are neither constructible nor inspectable here.
    #[must_use]
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits & !PRIVATE_GROUP_FLAG_MASK)
    }

    /// Returns exact known and retained unknown public bits.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns whether no public or retained unknown bits are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns whether every bit in `other` is present.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for ParticleGroupFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_bits_retain(self.0 | rhs.0)
    }
}

impl BitOrAssign for ParticleGroupFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitAnd for ParticleGroupFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_bits_retain(self.0 & rhs.0)
    }
}

impl BitAndAssign for ParticleGroupFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

/// A non-empty owned union of shapes whose interiors are filled in source order.
///
/// Only circles and polygons have a fillable interior. The shape order is
/// retained and later sampling must evaluate the union in that order.
#[derive(Debug, Clone, PartialEq)]
pub struct FilledParticleGroupShapes {
    shapes: Box<[Shape]>,
}

impl FilledParticleGroupShapes {
    /// Creates a checked non-empty filled-shape union.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an empty or oversized collection, or when an
    /// edge or chain is supplied to an interior-filling source.
    pub fn new(shapes: Vec<Shape>) -> Result<Self, ParticleGroupRecipeError> {
        validate_count(shapes.len())?;
        if shapes.is_empty() {
            return Err(ParticleGroupRecipeError::EmptySource);
        }
        if shapes
            .iter()
            .any(|shape| matches!(shape, Shape::Edge(_) | Shape::Chain(_)))
        {
            return Err(ParticleGroupRecipeError::UnsupportedFilledShape);
        }
        Ok(Self {
            shapes: shapes.into_boxed_slice(),
        })
    }

    /// Returns the owned shapes in their fixed sampling order.
    #[must_use]
    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }
}

/// One owned edge or chain sampled as a source-ordered stroke.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleGroupStrokeShape {
    shape: Shape,
}

impl ParticleGroupStrokeShape {
    /// Creates a checked stroke source.
    ///
    /// # Errors
    ///
    /// Returns [`ParticleGroupRecipeError::UnsupportedStrokeShape`] when a
    /// circle or polygon is supplied because those shapes select filled
    /// sampling rather than stroke sampling.
    pub fn new(shape: Shape) -> Result<Self, ParticleGroupRecipeError> {
        if matches!(shape, Shape::Circle(_) | Shape::Polygon(_)) {
            return Err(ParticleGroupRecipeError::UnsupportedStrokeShape);
        }
        Ok(Self { shape })
    }

    /// Returns the edge or chain sampled by this source.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// Non-empty owned explicit particle positions in source order.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleGroupPositions {
    positions: Box<[Vec2]>,
}

impl ParticleGroupPositions {
    /// Creates checked explicit positions.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an empty or oversized collection or for the
    /// first non-finite position.
    pub fn new(positions: Vec<Vec2>) -> Result<Self, ParticleGroupRecipeError> {
        validate_count(positions.len())?;
        if positions.is_empty() {
            return Err(ParticleGroupRecipeError::EmptySource);
        }
        if let Some(index) = positions.iter().position(|position| !position.is_valid()) {
            return Err(ParticleGroupRecipeError::NonFinitePosition { index });
        }
        Ok(Self {
            positions: positions.into_boxed_slice(),
        })
    }

    /// Returns explicit positions in their fixed creation order.
    #[must_use]
    pub fn positions(&self) -> &[Vec2] {
        &self.positions
    }
}

/// Exactly one owned particle-group sampling source.
#[derive(Debug, Clone, PartialEq)]
pub enum ParticleGroupSource {
    /// Fill the union of one or more circles or polygons.
    FilledShapes(FilledParticleGroupShapes),
    /// Stroke one edge or chain in its child order.
    StrokeShape(ParticleGroupStrokeShape),
    /// Create one particle at each explicit position.
    Positions(ParticleGroupPositions),
}

impl ParticleGroupSource {
    /// Creates a non-empty filled-shape source.
    ///
    /// # Errors
    ///
    /// Returns a typed source validation error.
    pub fn filled_shapes(shapes: Vec<Shape>) -> Result<Self, ParticleGroupRecipeError> {
        FilledParticleGroupShapes::new(shapes).map(Self::FilledShapes)
    }

    /// Creates one checked edge or chain stroke source.
    ///
    /// # Errors
    ///
    /// Returns a typed source validation error.
    pub fn stroke_shape(shape: Shape) -> Result<Self, ParticleGroupRecipeError> {
        ParticleGroupStrokeShape::new(shape).map(Self::StrokeShape)
    }

    /// Creates a non-empty explicit-position source.
    ///
    /// # Errors
    ///
    /// Returns a typed source validation error.
    pub fn positions(positions: Vec<Vec2>) -> Result<Self, ParticleGroupRecipeError> {
        ParticleGroupPositions::new(positions).map(Self::Positions)
    }
}

/// Whether a recipe creates a distinct group or appends to a live group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleGroupDestination {
    /// Create a new group identity.
    New,
    /// Append the sampled particles to this checked opaque group identity.
    AppendTo(ParticleGroupId),
}

/// A failure while constructing a checked [`ParticleGroupRecipe`] or source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleGroupRecipeError {
    /// A source collection contains no shapes or positions.
    EmptySource,
    /// A source collection exceeds the pinned signed 32-bit count range.
    SourceCountOutOfRange {
        /// Supplied collection length.
        count: usize,
    },
    /// A filled source contains an edge or chain without an interior.
    UnsupportedFilledShape,
    /// A stroke source contains a circle or polygon.
    UnsupportedStrokeShape,
    /// An explicit position contains a non-finite coordinate.
    NonFinitePosition {
        /// Source-order position of the invalid value.
        index: usize,
    },
    /// The transform contains a non-finite translation or rotation component.
    NonFiniteTransform,
    /// The linear velocity contains a non-finite coordinate.
    NonFiniteLinearVelocity,
    /// The angular velocity is not finite.
    NonFiniteAngularVelocity,
    /// The group strength is not finite.
    NonFiniteStrength,
    /// The group strength is negative.
    NegativeStrength,
    /// The explicit stride is not finite.
    NonFiniteStride,
    /// The explicit stride is zero or negative.
    NonPositiveStride,
    /// The lifetime is not finite.
    NonFiniteLifetime,
}

impl fmt::Display for ParticleGroupRecipeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySource => formatter.write_str("particle-group source must not be empty"),
            Self::SourceCountOutOfRange { count } => write!(
                formatter,
                "particle-group source count {count} exceeds the pinned signed 32-bit range"
            ),
            Self::UnsupportedFilledShape => formatter
                .write_str("particle-group filled sources support only circles and polygons"),
            Self::UnsupportedStrokeShape => {
                formatter.write_str("particle-group stroke sources support only edges and chains")
            }
            Self::NonFinitePosition { index } => {
                write!(formatter, "particle-group position {index} must be finite")
            }
            Self::NonFiniteTransform => {
                formatter.write_str("particle-group transform must be finite")
            }
            Self::NonFiniteLinearVelocity => {
                formatter.write_str("particle-group linear velocity must be finite")
            }
            Self::NonFiniteAngularVelocity => {
                formatter.write_str("particle-group angular velocity must be finite")
            }
            Self::NonFiniteStrength => {
                formatter.write_str("particle-group strength must be finite")
            }
            Self::NegativeStrength => {
                formatter.write_str("particle-group strength must be non-negative")
            }
            Self::NonFiniteStride => formatter.write_str("particle-group stride must be finite"),
            Self::NonPositiveStride => {
                formatter.write_str("particle-group stride must be positive")
            }
            Self::NonFiniteLifetime => {
                formatter.write_str("particle-group lifetime must be finite")
            }
        }
    }
}

impl Error for ParticleGroupRecipeError {}

/// An owned, reusable, invariant-bearing particle-group creation recipe.
///
/// Source and destination are independent: exactly one source is always
/// present, while append targets cannot be confused with sampling geometry.
/// A non-positive lifetime selects the pinned infinite-lifetime behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleGroupRecipe<UserAssociation = ()> {
    source: ParticleGroupSource,
    destination: ParticleGroupDestination,
    particle_flags: ParticleFlags,
    group_flags: ParticleGroupFlags,
    transform: Transform,
    linear_velocity: Vec2,
    angular_velocity: f32,
    color: ParticleColor,
    strength: f32,
    maybe_stride: Option<f32>,
    lifetime: f32,
    maybe_user_association: Option<UserAssociation>,
}

impl ParticleGroupRecipe<()> {
    /// Creates a recipe from one checked source and an independent destination.
    #[must_use]
    pub fn new(source: ParticleGroupSource, destination: ParticleGroupDestination) -> Self {
        Self {
            source,
            destination,
            particle_flags: ParticleFlags::WATER,
            group_flags: ParticleGroupFlags::empty(),
            transform: Transform::IDENTITY,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            color: ParticleColor::ZERO,
            strength: 1.0,
            maybe_stride: None,
            lifetime: 0.0,
            maybe_user_association: None,
        }
    }
}

impl<UserAssociation> ParticleGroupRecipe<UserAssociation> {
    /// Returns a copy with exact known and retained unknown particle flags.
    #[must_use]
    pub const fn with_particle_flags(mut self, flags: ParticleFlags) -> Self {
        self.particle_flags = flags;
        self
    }

    /// Returns a copy with exact public and retained unknown group bits.
    #[must_use]
    pub const fn with_group_flags(mut self, flags: ParticleGroupFlags) -> Self {
        self.group_flags = flags;
        self
    }

    /// Returns a copy with a checked finite transform.
    ///
    /// # Errors
    ///
    /// Returns [`ParticleGroupRecipeError::NonFiniteTransform`] when any
    /// translation or rotation component is non-finite.
    pub fn with_transform(
        mut self,
        transform: Transform,
    ) -> Result<Self, ParticleGroupRecipeError> {
        let rotation = transform.rotation();
        if !transform.position().is_valid()
            || !rotation.sine().is_finite()
            || !rotation.cosine().is_finite()
        {
            return Err(ParticleGroupRecipeError::NonFiniteTransform);
        }
        self.transform = transform;
        Ok(self)
    }

    /// Returns a copy with checked finite linear velocity.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite component.
    pub fn with_linear_velocity(
        mut self,
        velocity: Vec2,
    ) -> Result<Self, ParticleGroupRecipeError> {
        if !velocity.is_valid() {
            return Err(ParticleGroupRecipeError::NonFiniteLinearVelocity);
        }
        self.linear_velocity = velocity;
        Ok(self)
    }

    /// Returns a copy with checked finite angular velocity in radians per second.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite value.
    pub fn with_angular_velocity(
        mut self,
        angular_velocity: f32,
    ) -> Result<Self, ParticleGroupRecipeError> {
        if !angular_velocity.is_finite() {
            return Err(ParticleGroupRecipeError::NonFiniteAngularVelocity);
        }
        self.angular_velocity = angular_velocity;
        Ok(self)
    }

    /// Returns a copy with an exact particle color.
    #[must_use]
    pub const fn with_color(mut self, color: ParticleColor) -> Self {
        self.color = color;
        self
    }

    /// Returns a copy with checked finite non-negative connection strength.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite or negative value.
    pub fn with_strength(self, strength: f32) -> Result<Self, ParticleGroupRecipeError> {
        if !strength.is_finite() {
            return Err(ParticleGroupRecipeError::NonFiniteStrength);
        }
        if strength < 0.0 {
            return Err(ParticleGroupRecipeError::NegativeStrength);
        }
        let mut recipe = self;
        recipe.strength = strength;
        Ok(recipe)
    }

    /// Returns a copy with checked positive particle sampling stride in meters.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite or non-positive value.
    pub fn with_stride(self, stride: f32) -> Result<Self, ParticleGroupRecipeError> {
        if !stride.is_finite() {
            return Err(ParticleGroupRecipeError::NonFiniteStride);
        }
        if stride <= 0.0 {
            return Err(ParticleGroupRecipeError::NonPositiveStride);
        }
        let mut recipe = self;
        recipe.maybe_stride = Some(stride);
        Ok(recipe)
    }

    /// Returns a copy using the particle system's pinned default stride.
    #[must_use]
    pub const fn with_default_stride(mut self) -> Self {
        self.maybe_stride = None;
        self
    }

    /// Returns a copy with checked finite lifetime in seconds.
    ///
    /// Values at or below zero retain their exact bits and select the pinned
    /// infinite-lifetime behavior.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite value.
    pub fn with_lifetime(self, lifetime: f32) -> Result<Self, ParticleGroupRecipeError> {
        if !lifetime.is_finite() {
            return Err(ParticleGroupRecipeError::NonFiniteLifetime);
        }
        let mut recipe = self;
        recipe.lifetime = lifetime;
        Ok(recipe)
    }

    /// Carries an application-owned association input with this recipe.
    #[must_use]
    pub fn with_user_association<NewAssociation>(
        self,
        user_association: NewAssociation,
    ) -> ParticleGroupRecipe<NewAssociation> {
        ParticleGroupRecipe {
            source: self.source,
            destination: self.destination,
            particle_flags: self.particle_flags,
            group_flags: self.group_flags,
            transform: self.transform,
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
            color: self.color,
            strength: self.strength,
            maybe_stride: self.maybe_stride,
            lifetime: self.lifetime,
            maybe_user_association: Some(user_association),
        }
    }

    /// Returns the single checked sampling source.
    #[must_use]
    pub const fn source(&self) -> &ParticleGroupSource {
        &self.source
    }

    /// Returns whether creation starts a group or appends to a target.
    #[must_use]
    pub const fn destination(&self) -> ParticleGroupDestination {
        self.destination
    }

    /// Returns exact particle behavior flags.
    #[must_use]
    pub const fn particle_flags(&self) -> ParticleFlags {
        self.particle_flags
    }

    /// Returns exact public and retained unknown group flag bits.
    #[must_use]
    pub const fn group_flags(&self) -> ParticleGroupFlags {
        self.group_flags
    }

    /// Returns the finite sampling transform.
    #[must_use]
    pub const fn transform(&self) -> Transform {
        self.transform
    }

    /// Returns finite linear velocity in meters per second.
    #[must_use]
    pub const fn linear_velocity(&self) -> Vec2 {
        self.linear_velocity
    }

    /// Returns finite angular velocity in radians per second.
    #[must_use]
    pub const fn angular_velocity(&self) -> f32 {
        self.angular_velocity
    }

    /// Returns the exact particle color.
    #[must_use]
    pub const fn color(&self) -> ParticleColor {
        self.color
    }

    /// Returns finite non-negative connection strength.
    #[must_use]
    pub const fn strength(&self) -> f32 {
        self.strength
    }

    /// Returns a positive explicit stride, or `None` for the pinned default.
    #[must_use]
    pub const fn maybe_stride(&self) -> Option<f32> {
        self.maybe_stride
    }

    /// Returns lifetime in seconds; values at or below zero mean infinite.
    #[must_use]
    pub const fn lifetime(&self) -> f32 {
        self.lifetime
    }

    /// Returns the typed application association input, when present.
    #[must_use]
    pub const fn maybe_user_association(&self) -> Option<&UserAssociation> {
        self.maybe_user_association.as_ref()
    }
}

/// Borrow-scoped semantic inspection of one live particle group.
///
/// Member identities remain in source order and optional depth values align
/// one-to-one with them. Dense rows, mutable storage, internal flags, and
/// cached statistics do not cross this boundary.
///
/// Returned member borrows cannot escape the view's storage borrow:
///
/// ```compile_fail
/// use liquidfun::particle::ParticleGroupView;
/// use liquidfun::ParticleId;
///
/// fn escape(view: &ParticleGroupView<'_>) -> &'static [ParticleId] {
///     view.member_ids()
/// }
/// ```
///
/// Dense and mutable storage accessors are deliberately absent:
///
/// ```compile_fail
/// use liquidfun::particle::ParticleGroupView;
///
/// fn expose_dense_row(view: &ParticleGroupView<'_>) -> usize {
///     view.row()
/// }
/// ```
///
/// ```compile_fail
/// use liquidfun::particle::ParticleGroupView;
///
/// fn mutate_members(view: &mut ParticleGroupView<'_>) {
///     view.member_ids_mut().clear();
/// }
/// ```
#[derive(Debug)]
pub struct ParticleGroupView<'a> {
    state: ParticleGroupViewState,
    member_ids: &'a [ParticleId],
    maybe_depths: Option<&'a [f32]>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(
    dead_code,
    reason = "constructed by the Phase 10 world-facing particle-group integration"
)]
pub(crate) struct ParticleGroupViewState {
    pub(crate) id: ParticleGroupId,
    pub(crate) flags: ParticleGroupFlags,
    pub(crate) transform: Transform,
    pub(crate) center: Vec2,
    pub(crate) linear_velocity: Vec2,
    pub(crate) angular_velocity: f32,
    pub(crate) mass: f32,
    pub(crate) inertia: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "returned by the Phase 10 world-facing particle-group integration"
)]
pub(crate) enum ParticleGroupViewError {
    MisalignedDepth {
        member_count: usize,
        depth_count: usize,
    },
}

impl<'a> ParticleGroupView<'a> {
    #[allow(
        dead_code,
        reason = "called by the Phase 10 world-facing particle-group integration"
    )]
    pub(crate) fn new(
        mut state: ParticleGroupViewState,
        member_ids: &'a [ParticleId],
        maybe_depths: Option<&'a [f32]>,
    ) -> Result<Self, ParticleGroupViewError> {
        if let Some(depths) = maybe_depths
            && depths.len() != member_ids.len()
        {
            return Err(ParticleGroupViewError::MisalignedDepth {
                member_count: member_ids.len(),
                depth_count: depths.len(),
            });
        }
        if member_ids.is_empty() {
            state.center = Vec2::ZERO;
            state.linear_velocity = Vec2::ZERO;
            state.angular_velocity = 0.0;
            state.mass = 0.0;
            state.inertia = 0.0;
        }
        Ok(Self {
            state,
            member_ids,
            maybe_depths,
        })
    }

    /// Returns the stable world-scoped group identity.
    #[must_use]
    pub const fn id(&self) -> ParticleGroupId {
        self.state.id
    }

    /// Returns exact public and retained unknown group flag bits.
    #[must_use]
    pub const fn flags(&self) -> ParticleGroupFlags {
        self.state.flags
    }

    /// Returns the group's origin transform.
    #[must_use]
    pub const fn transform(&self) -> Transform {
        self.state.transform
    }

    /// Returns the group's origin position in meters.
    #[must_use]
    pub const fn position(&self) -> Vec2 {
        self.state.transform.position()
    }

    /// Returns the group's origin angle in radians.
    #[must_use]
    pub fn angle(&self) -> f32 {
        self.state.transform.rotation().angle()
    }

    /// Returns the current center of mass in meters.
    #[must_use]
    pub const fn center(&self) -> Vec2 {
        self.state.center
    }

    /// Returns the current center-of-mass velocity in meters per second.
    #[must_use]
    pub const fn linear_velocity(&self) -> Vec2 {
        self.state.linear_velocity
    }

    /// Returns the current angular velocity in radians per second.
    #[must_use]
    pub const fn angular_velocity(&self) -> f32 {
        self.state.angular_velocity
    }

    /// Returns total particle mass in kilograms.
    #[must_use]
    pub const fn mass(&self) -> f32 {
        self.state.mass
    }

    /// Returns moment of inertia about the center of mass.
    #[must_use]
    pub const fn inertia(&self) -> f32 {
        self.state.inertia
    }

    /// Returns the current member count.
    #[must_use]
    pub const fn member_count(&self) -> usize {
        self.member_ids.len()
    }

    /// Returns stable particle identities in source member order.
    #[must_use]
    pub const fn member_ids(&self) -> &[ParticleId] {
        self.member_ids
    }

    /// Returns depth values aligned with members when the depth lane applies.
    #[must_use]
    pub const fn maybe_depths(&self) -> Option<&[f32]> {
        self.maybe_depths
    }
}

fn validate_count(count: usize) -> Result<(), ParticleGroupRecipeError> {
    if count > MAX_UPSTREAM_COUNT {
        return Err(ParticleGroupRecipeError::SourceCountOutOfRange { count });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
