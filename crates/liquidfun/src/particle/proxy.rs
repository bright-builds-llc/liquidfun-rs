//! Checked spatial tags and source-ordered particle neighborhoods.

use crate::collision::Aabb;
use crate::{ParticleId, ParticleSystemId};

use super::ParticleSystemView;
use super::storage::ParticleIndex;

const X_TRUNC_BITS: u32 = 12;
const Y_TRUNC_BITS: u32 = 12;
const TAG_BITS: u32 = u32::BITS;
const Y_OFFSET: f32 = 2_048.0;
const Y_SHIFT: u32 = TAG_BITS - Y_TRUNC_BITS;
const X_SHIFT: u32 = TAG_BITS - Y_TRUNC_BITS - X_TRUNC_BITS;
const X_SCALE: f32 = 256.0;
const X_OFFSET: f32 = 524_288.0;
const Y_MASK: u32 = ((1_u32 << Y_TRUNC_BITS) - 1) << Y_SHIFT;
const X_MASK: u32 = !Y_MASK;
const RELATIVE_RIGHT: u32 = 1_u32 << X_SHIFT;
const RELATIVE_BOTTOM_LEFT: u32 = (1_u32 << Y_SHIFT).wrapping_sub(1_u32 << X_SHIFT);
const RELATIVE_BOTTOM_RIGHT: u32 = (1_u32 << Y_SHIFT) + (1_u32 << X_SHIFT);
const X_TAG_LIMIT: f32 = 1_048_576.0;
const Y_TAG_LIMIT: f32 = 4_096.0;

/// A failure while constructing checked particle spatial tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleProxyError {
    /// The particle diameter is not finite.
    NonFiniteDiameter,
    /// The particle diameter is zero or negative.
    NonPositiveDiameter,
    /// Derived diameter scaling cannot remain finite and nonzero.
    DiameterOutOfRange,
    /// A scaled particle position cannot be represented by the pinned tag layout.
    PositionOutOfTagRange,
}

/// One broad neighborhood candidate in pinned source enumeration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleNeighborPair {
    particles: [ParticleId; 2],
}

impl ParticleNeighborPair {
    /// Creates a semantic pair in source enumeration order.
    #[must_use]
    pub const fn new(first: ParticleId, second: ParticleId) -> Self {
        Self {
            particles: [first, second],
        }
    }

    /// Returns the stable particle identities in source enumeration order.
    #[must_use]
    pub const fn particles(self) -> [ParticleId; 2] {
        self.particles
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Proxy {
    particle: ParticleId,
    row: ParticleIndex,
    tag: u32,
}

/// Owned, source-ordered spatial candidates for one particle-system snapshot.
///
/// Dense rows and packed tags remain private. All returned particles retain
/// their stable world- and system-scoped identities.
///
/// ```compile_fail
/// use liquidfun::{ParticleNeighborhood, World};
/// fn must_not_expose_rows(neighborhood: &ParticleNeighborhood) {
///     let _ = neighborhood.pair_rows();
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleNeighborhood {
    system: ParticleSystemId,
    diameter: f32,
    proxies: Vec<Proxy>,
    pairs: Vec<ParticleNeighborPair>,
    pair_rows: Vec<[ParticleIndex; 2]>,
}

impl ParticleNeighborhood {
    /// Builds checked spatial proxies and broad neighborhood candidates.
    ///
    /// Equal packed tags retain the borrow-scoped row order. The pinned C++
    /// comparator treats equal tags as equivalent; retaining row order gives
    /// Rust a deterministic representative without making the tag public.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid diameter or a position outside the
    /// finite 12-bit-by-12-bit tag domain.
    pub fn from_view(
        view: &ParticleSystemView<'_>,
        diameter: f32,
    ) -> Result<Self, ParticleProxyError> {
        if !diameter.is_finite() {
            return Err(ParticleProxyError::NonFiniteDiameter);
        }
        if diameter <= 0.0 {
            return Err(ParticleProxyError::NonPositiveDiameter);
        }

        let inverse_diameter = 1.0 / diameter;
        let squared_diameter = diameter * diameter;
        if !inverse_diameter.is_finite()
            || inverse_diameter <= 0.0
            || !squared_diameter.is_finite()
            || squared_diameter <= 0.0
        {
            return Err(ParticleProxyError::DiameterOutOfRange);
        }
        let mut proxies = Vec::with_capacity(view.particle_ids().len());
        for (row, (particle, position)) in view
            .particle_ids()
            .iter()
            .copied()
            .zip(view.positions().iter().copied())
            .enumerate()
        {
            let tag = checked_tag(inverse_diameter * position.x, inverse_diameter * position.y)?;
            proxies.push(Proxy {
                particle,
                row: ParticleIndex(row),
                tag,
            });
        }
        proxies.sort_by_key(|proxy| proxy.tag);
        let (pairs, pair_rows) = enumerate_pairs(&proxies);

        Ok(Self {
            system: view.system(),
            diameter,
            proxies,
            pairs,
            pair_rows,
        })
    }

    /// Returns the owning particle system.
    #[must_use]
    pub const fn system(&self) -> ParticleSystemId {
        self.system
    }

    /// Returns broad candidate pairs in source enumeration order.
    #[must_use]
    pub fn pairs(&self) -> &[ParticleNeighborPair] {
        &self.pairs
    }

    pub(in crate::particle) fn pair_rows(&self) -> &[[ParticleIndex; 2]] {
        &self.pair_rows
    }

    pub(crate) const fn diameter(&self) -> f32 {
        self.diameter
    }

    /// Returns the source-expanded proxy candidates for an AABB.
    ///
    /// This is deliberately the broad candidate set: callers that need exact
    /// containment must apply their narrow geometry predicate afterward.
    ///
    /// # Errors
    ///
    /// Returns [`ParticleProxyError::PositionOutOfTagRange`] when expanding the
    /// bounds by one particle diameter leaves the checked tag domain.
    pub fn particle_candidates_in_bounds(
        &self,
        bounds: Aabb,
    ) -> Result<Vec<ParticleId>, ParticleProxyError> {
        let inverse_diameter = 1.0 / self.diameter;
        let lower = bounds.lower_bound();
        let upper = bounds.upper_bound();
        let lower_tag = checked_tag(
            inverse_diameter * lower.x - 1.0,
            inverse_diameter * lower.y - 1.0,
        )?;
        let upper_tag = checked_tag(
            inverse_diameter * upper.x + 1.0,
            inverse_diameter * upper.y + 1.0,
        )?;
        let first = self.proxies.partition_point(|proxy| proxy.tag < lower_tag);
        let last = self.proxies.partition_point(|proxy| proxy.tag <= upper_tag);
        let x_lower = lower_tag & X_MASK;
        let x_upper = upper_tag & X_MASK;

        Ok(self.proxies[first..last]
            .iter()
            .filter(|proxy| {
                let x_tag = proxy.tag & X_MASK;
                x_tag >= x_lower && x_tag <= x_upper
            })
            .map(|proxy| proxy.particle)
            .collect())
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "checked nonnegative finite ranges make these casts the pinned truncation toward zero"
)]
fn checked_tag(x: f32, y: f32) -> Result<u32, ParticleProxyError> {
    let scaled_x = X_SCALE * x + X_OFFSET;
    let offset_y = y + Y_OFFSET;
    if !x.is_finite()
        || !y.is_finite()
        || !(0.0..X_TAG_LIMIT).contains(&scaled_x)
        || !(0.0..Y_TAG_LIMIT).contains(&offset_y)
    {
        return Err(ParticleProxyError::PositionOutOfTagRange);
    }
    Ok(((offset_y as u32) << Y_SHIFT) + scaled_x as u32)
}

fn visit_pairs(proxies: &[Proxy], mut visit: impl FnMut(&Proxy, &Proxy)) {
    let mut below_start = 0;
    for (a_index, a) in proxies.iter().enumerate() {
        let right_tag = a.tag.wrapping_add(RELATIVE_RIGHT);
        for b in &proxies[a_index + 1..] {
            if right_tag < b.tag {
                break;
            }
            visit(a, b);
        }

        let bottom_left_tag = a.tag.wrapping_add(RELATIVE_BOTTOM_LEFT);
        while below_start < proxies.len() && proxies[below_start].tag < bottom_left_tag {
            below_start += 1;
        }
        let bottom_right_tag = a.tag.wrapping_add(RELATIVE_BOTTOM_RIGHT);
        for b in &proxies[below_start..] {
            if bottom_right_tag < b.tag {
                break;
            }
            visit(a, b);
        }
    }
}

fn enumerate_pairs(proxies: &[Proxy]) -> (Vec<ParticleNeighborPair>, Vec<[ParticleIndex; 2]>) {
    let mut count = 0_usize;
    visit_pairs(proxies, |_, _| count += 1);
    let mut pairs = Vec::with_capacity(count);
    let mut pair_rows = Vec::with_capacity(count);
    visit_pairs(proxies, |a, b| {
        pairs.push(ParticleNeighborPair::new(a.particle, b.particle));
        pair_rows.push([a.row, b.row]);
    });
    (pairs, pair_rows)
}

#[cfg(test)]
mod row_carry {
    use super::*;
    use crate::math::Vec2;
    use crate::{ParticleDef, World};

    #[test]
    fn neighborhood_pair_rows_align_with_public_particle_ids() {
        // Arrange
        let mut world = World::new().expect("test world key remains available");
        let system = world
            .create_particle_system()
            .expect("particle system should fit");
        let create_at = |world: &mut World, position: Vec2| {
            let definition = ParticleDef::default()
                .with_position(position)
                .expect("test position should be finite");
            let _ = world
                .create_particle_with_def(system, None, &definition)
                .expect("particle should fit")
                .created_particle();
        };
        create_at(&mut world, Vec2::new(0.0, 0.0));
        create_at(&mut world, Vec2::new(0.5, 0.0));
        create_at(&mut world, Vec2::new(2.0, 0.0));
        let view = world
            .particle_system_view(system)
            .expect("particle system should remain live");

        // Act
        let neighborhood = ParticleNeighborhood::from_view(&view, 1.0)
            .expect("finite in-range positions should build proxies");

        // Assert
        assert_eq!(neighborhood.pair_rows().len(), neighborhood.pairs().len());
        for (i, pair) in neighborhood.pairs().iter().enumerate() {
            let rows = neighborhood.pair_rows()[i];
            assert_eq!(view.particle_ids()[rows[0].0], pair.particles()[0]);
            assert_eq!(view.particle_ids()[rows[1].0], pair.particles()[1]);
        }
    }
}
