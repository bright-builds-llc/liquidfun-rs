//! Pure particle AABB and ray traversal over stable semantic identities.

use std::error::Error;
use std::fmt;

use crate::collision::{Aabb, RayCastInput};
use crate::math::Vec2;
use crate::{
    HandleError, ParticleId, ParticleSystemId, QueryDirective, RayCastDirective, RayCastFraction,
    World,
};

use super::{ParticleNeighborhood, ParticleProxyError, ParticleSystemView};

/// One borrow-scoped stable particle occurrence from an AABB query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleQueryOccurrence {
    system: ParticleSystemId,
    particle: ParticleId,
}

impl ParticleQueryOccurrence {
    /// Returns the owning particle system.
    #[must_use]
    pub const fn system(self) -> ParticleSystemId {
        self.system
    }

    /// Returns the stable particle identity.
    #[must_use]
    pub const fn particle(self) -> ParticleId {
        self.particle
    }
}

/// Owned semantic data for one particle ray hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleRayHit {
    system: ParticleSystemId,
    particle: ParticleId,
    point: Vec2,
    normal: Vec2,
    fraction: RayCastFraction,
}

impl ParticleRayHit {
    /// Returns the owning particle system.
    #[must_use]
    pub const fn system(self) -> ParticleSystemId {
        self.system
    }

    /// Returns the stable particle identity.
    #[must_use]
    pub const fn particle(self) -> ParticleId {
        self.particle
    }

    /// Returns the finite world-space intersection point.
    #[must_use]
    pub const fn point(self) -> Vec2 {
        self.point
    }

    /// Returns the finite outward particle normal.
    #[must_use]
    pub const fn normal(self) -> Vec2 {
        self.normal
    }

    /// Returns the checked normalized intersection fraction.
    #[must_use]
    pub const fn fraction(self) -> RayCastFraction {
        self.fraction
    }
}

/// A failure while querying one particle system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleQueryError {
    /// The particle-system handle is foreign, stale, or world access is poisoned.
    InvalidHandle(HandleError),
    /// Spatial proxy construction or checked bound expansion failed.
    InvalidProxyGeometry(ParticleProxyError),
}

impl fmt::Display for ParticleQueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid particle system: {error}"),
            Self::InvalidProxyGeometry(_error) => {
                formatter.write_str("particle query geometry is outside the checked proxy domain")
            }
        }
    }
}

impl Error for ParticleQueryError {}

impl From<HandleError> for ParticleQueryError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<ParticleProxyError> for ParticleQueryError {
    fn from(error: ParticleProxyError) -> Self {
        Self::InvalidProxyGeometry(error)
    }
}

/// A failure while ray casting one or more particle systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleRayCastError {
    /// The particle-system handle is foreign, stale, or world access is poisoned.
    InvalidHandle(HandleError),
    /// Spatial proxy construction or checked ray-bound expansion failed.
    InvalidProxyGeometry(ParticleProxyError),
    /// The supplied ray has no direction.
    DegenerateRay,
    /// Source-ordered ray arithmetic produced a non-finite value.
    NonFiniteDerivedGeometry,
    /// A callback directive attempted to widen the current interval.
    ClipOutsideCurrentInterval,
}

impl fmt::Display for ParticleRayCastError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidHandle(error) => {
                return write!(formatter, "invalid particle system: {error}");
            }
            Self::InvalidProxyGeometry(_error) => {
                "particle ray geometry is outside the checked proxy domain"
            }
            Self::DegenerateRay => "particle ray must have a non-zero direction",
            Self::NonFiniteDerivedGeometry => {
                "particle ray arithmetic produced non-finite geometry"
            }
            Self::ClipOutsideCurrentInterval => {
                "particle ray directive cannot widen the current traversal interval"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for ParticleRayCastError {}

impl From<HandleError> for ParticleRayCastError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<ParticleProxyError> for ParticleRayCastError {
    fn from(error: ParticleProxyError) -> Self {
        Self::InvalidProxyGeometry(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ParticleRayTraversal {
    pub(crate) current_fraction: f32,
    pub(crate) terminated: bool,
}

impl World {
    /// Visits particles whose centers lie strictly inside `bounds`.
    ///
    /// Stable identities are reported in the system's source proxy order.
    /// General callback order is intentionally unspecified.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a foreign or stale system or checked proxy
    /// geometry outside the representable spatial tag domain.
    pub fn query_particle_system_aabb(
        &self,
        system: ParticleSystemId,
        bounds: Aabb,
        mut visitor: impl FnMut(&ParticleQueryOccurrence) -> QueryDirective,
    ) -> Result<(), ParticleQueryError> {
        let definition = self.particle_system_snapshot(system)?.definition();
        let view = self.particle_system_view(system)?;
        let diameter = 2.0 * definition.radius();
        query_aabb(&view, diameter, bounds, &mut visitor)?;
        Ok(())
    }

    /// Casts a checked ray against particles in one live system.
    ///
    /// Particles containing the ray start are excluded. Ignore and continue
    /// preserve the interval, terminate stops immediately, and clip narrows
    /// later candidates only after validation against the current interval.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid system, checked proxy or derived
    /// geometry failure, a degenerate ray, or a widening clip directive.
    pub fn ray_cast_particle_system(
        &self,
        system: ParticleSystemId,
        input: RayCastInput,
        mut visitor: impl FnMut(&ParticleRayHit) -> RayCastDirective,
    ) -> Result<(), ParticleRayCastError> {
        let definition = self.particle_system_snapshot(system)?.definition();
        let view = self.particle_system_view(system)?;
        let diameter = 2.0 * definition.radius();
        ray_cast(&view, diameter, input, input.max_fraction(), &mut visitor)?;
        Ok(())
    }
}

pub(crate) fn query_aabb(
    view: &ParticleSystemView<'_>,
    diameter: f32,
    bounds: Aabb,
    visitor: &mut impl FnMut(&ParticleQueryOccurrence) -> QueryDirective,
) -> Result<bool, ParticleQueryError> {
    let neighborhood = ParticleNeighborhood::from_view(view, diameter)?;
    let candidates = neighborhood.particle_candidates_in_bounds(bounds)?;
    let lower = bounds.lower_bound();
    let upper = bounds.upper_bound();

    for candidate in candidates {
        let Some(position) = position_for(view, candidate) else {
            unreachable!("particle proxy identities originate from the same immutable view")
        };
        if !(lower.x < position.x
            && position.x < upper.x
            && lower.y < position.y
            && position.y < upper.y)
        {
            continue;
        }
        let occurrence = ParticleQueryOccurrence {
            system: view.system(),
            particle: candidate,
        };
        if visitor(&occurrence) == QueryDirective::Terminate {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn ray_cast(
    view: &ParticleSystemView<'_>,
    diameter: f32,
    input: RayCastInput,
    current_fraction: f32,
    visitor: &mut impl FnMut(&ParticleRayHit) -> RayCastDirective,
) -> Result<ParticleRayTraversal, ParticleRayCastError> {
    let direction = input.end() - input.start();
    let direction_squared = direction.length_squared();
    if direction_squared == 0.0 {
        return Err(ParticleRayCastError::DegenerateRay);
    }
    if !direction_squared.is_finite() {
        return Err(ParticleRayCastError::NonFiniteDerivedGeometry);
    }

    let ray_bounds = Aabb::new(
        Vec2::new(
            input.start().x.min(input.end().x),
            input.start().y.min(input.end().y),
        ),
        Vec2::new(
            input.start().x.max(input.end().x),
            input.start().y.max(input.end().y),
        ),
    )
    .map_err(|_error| ParticleRayCastError::NonFiniteDerivedGeometry)?;
    let neighborhood = ParticleNeighborhood::from_view(view, diameter)?;
    let candidates = neighborhood.particle_candidates_in_bounds(ray_bounds)?;
    let squared_diameter = diameter * diameter;
    if !squared_diameter.is_finite() {
        return Err(ParticleRayCastError::NonFiniteDerivedGeometry);
    }

    let mut fraction = current_fraction.min(input.max_fraction());
    for candidate in candidates {
        let Some(position) = position_for(view, candidate) else {
            unreachable!("particle proxy identities originate from the same immutable view")
        };
        let relative_start = input.start() - position;
        let start_distance_squared = relative_start.length_squared();
        if !start_distance_squared.is_finite() {
            return Err(ParticleRayCastError::NonFiniteDerivedGeometry);
        }
        if start_distance_squared < squared_diameter {
            continue;
        }

        let projection = relative_start.dot(direction);
        let determinant = projection * projection
            - direction_squared * (start_distance_squared - squared_diameter);
        if !determinant.is_finite() {
            return Err(ParticleRayCastError::NonFiniteDerivedGeometry);
        }
        if determinant < 0.0 {
            continue;
        }
        let hit_fraction = (-projection - determinant.sqrt()) / direction_squared;
        if hit_fraction < 0.0 || hit_fraction > fraction {
            continue;
        }
        let mut normal = relative_start + hit_fraction * direction;
        let normal_length = normal.normalize();
        let point = input.start() + hit_fraction * direction;
        if !normal_length.is_finite() || normal_length == 0.0 || !point.is_valid() {
            return Err(ParticleRayCastError::NonFiniteDerivedGeometry);
        }
        let checked_fraction = RayCastFraction::new(hit_fraction)
            .map_err(|_error| ParticleRayCastError::NonFiniteDerivedGeometry)?;
        let hit = ParticleRayHit {
            system: view.system(),
            particle: candidate,
            point,
            normal,
            fraction: checked_fraction,
        };

        match visitor(&hit) {
            RayCastDirective::Ignore | RayCastDirective::Continue => {}
            RayCastDirective::Terminate => {
                return Ok(ParticleRayTraversal {
                    current_fraction: fraction,
                    terminated: true,
                });
            }
            RayCastDirective::Clip(clip) => {
                if clip.get() > fraction {
                    return Err(ParticleRayCastError::ClipOutsideCurrentInterval);
                }
                fraction = clip.get();
                if fraction == 0.0 {
                    return Ok(ParticleRayTraversal {
                        current_fraction: fraction,
                        terminated: true,
                    });
                }
            }
        }
    }

    Ok(ParticleRayTraversal {
        current_fraction: fraction,
        terminated: false,
    })
}

fn position_for(view: &ParticleSystemView<'_>, particle: ParticleId) -> Option<Vec2> {
    view.particle_ids()
        .iter()
        .position(|candidate| *candidate == particle)
        .map(|index| view.positions()[index])
}
