//! Conservative spatial query pad for a moving fixture at particle iteration 0.
//!
//! At iteration 0 the CCD ray of particle `p` starts at
//! `S(p) = collision_start_from_previous_transform(p, ...)`, which maps `p`
//! through the previous body transform into the current one, and ends at
//! `E(p) = p + dt * v`. Static fixtures and later iterations have `S(p) = p`,
//! so the fixture AABB padded by the travel bound `motion` already holds every
//! particle whose travel box can overlap it. A moving fixture remaps the start,
//! and the remap grows with distance from the body origin. This module finds a
//! pad `r` for that case such that skipping every particle outside the fixture
//! AABB `A` padded by `r` never skips a hit.
//!
//! Soundness. `S` is affine, `S(p) = M p + t`. Polygons, edges and chains map
//! `T_cur ∘ T_prev⁻¹`, so `M = R_cur R_prevᵀ`. Circles also rotate by `R_prev`
//! and then `R_curᵀ` about the local center, so `M = k I` with
//! `k = |q_cur|² |q_prev|²` (1 up to rotation rounding). `E` satisfies
//! `|E(p) - p|_inf <= motion`.
//!
//! 1. Suppose the travel box of `[S(p), E(p)]` overlaps `A`. Some `q` in that
//!    box lies in `A`, and on each axis `|q_i - p_i|` is at most
//!    `max(|S_i(p) - p_i|, |E_i(p) - p_i|)`. So the inf-norm distance `δ` from
//!    `p` to `A` satisfies `δ <= max(f(p), motion)` with `f(p) = |S(p) - p|_inf`.
//! 1. Let `a` be `p` clamped into `A`, so `|p - a|_inf = δ`. `S(p) - p` is
//!    affine with linear part `M - I`, so `f(p) <= f(a) + L δ` with
//!    `L = ||M - I||_inf`. For polygons `L = |cos d - 1| + |sin d|`, read from
//!    the entries of `R_cur R_prevᵀ`; for circles `L = |k - 1|`.
//! 1. `f` is the norm of an affine map, so it is convex, and its maximum over the
//!    box `A` is attained at one of its 4 corners. Call that maximum `D0`, so
//!    `f(a) <= D0`.
//! 1. Hence `δ <= motion` or `δ (1 - L) <= D0`. **When `L < 1`**, every
//!    particle that can hit lies in `A` padded by `max(motion, D0 / (1 - L))`.
//!    When `L >= 1` no finite pad follows and the caller keeps the full scan.
//!
//! Floating point. The per-particle start is computed in f32, so `L` gets a
//! relative margin plus a rounding budget per meter of coordinate scale, and the
//! constant term gets a relative margin, an absolute margin and the same
//! rounding budget times the coordinate scale. The closed form is then
//! overshot by 5% and checked once more against the f32 start of the padded
//! box's corners; any failure returns `None`, which keeps the full scan. There
//! is no fixed-point iteration: `D(r) <= D0 + L r` grows with `r`, so iterating
//! `r = need(r)` creeps toward the fixed point without verifying.

use crate::collision::Aabb;
use crate::math::{Transform, Vec2};
use crate::particle::solver::boundary::collision_start_from_previous_transform;

/// Relative margin on the Lipschitz constant and the displacement bound.
const RELATIVE_MARGIN: f32 = 1.0e-3;
/// Absolute margin in meters, far above f32 rounding at playground coordinates.
const ABSOLUTE_MARGIN: f32 = 1.0e-3;
/// Rounding budget of one f32 start computation per meter of coordinate scale.
/// It is several times the handful of rounded operations in the start
/// computation, and it keeps the bound sound at large coordinates.
const ROUNDING_PER_METER: f32 = 64.0 * f32::EPSILON;
/// Overshoot of the closed-form pad so the corner check passes despite rounding.
const OVERSHOOT: f32 = 1.05;

/// Smallest verified pad r so every particle whose iteration-0 travel box can
/// overlap `aabb` lies inside `aabb` padded by r; None when it does not converge.
///
/// `motion` bounds `|E(p) - p|_inf` for every particle, as returned by the
/// caller's `max_particle_motion`. `None` means the caller must keep its full
/// particle scan: the rotation is too large (`L >= 1`), an input is not finite,
/// or the f32 corner check rejects the closed-form pad.
pub(super) fn moving_fixture_query_pad(
    aabb: Aabb,
    motion: f32,
    previous: Transform,
    current: Transform,
    body_local_center: Vec2,
    is_circle: bool,
) -> Option<f32> {
    if !motion.is_finite() || motion < 0.0 {
        return None;
    }
    let start = |point: Vec2| {
        collision_start_from_previous_transform(
            point,
            previous,
            current,
            body_local_center,
            is_circle,
            0,
        )
        .ok()
    };
    let lipschitz = linear_deviation(previous, current, is_circle) * (1.0 + RELATIVE_MARGIN)
        + ROUNDING_PER_METER;
    if lipschitz.is_nan() || lipschitz >= 1.0 {
        return None;
    }
    let slack = ABSOLUTE_MARGIN
        + ROUNDING_PER_METER * coordinate_scale(aabb, previous, current, body_local_center);
    let base = corner_displacement(aabb, start)?;
    let closed_form = (motion.max(base) * (1.0 + RELATIVE_MARGIN) + slack) / (1.0 - lipschitz);
    let pad = closed_form * OVERSHOOT + ABSOLUTE_MARGIN;
    if !pad.is_finite() {
        return None;
    }
    let padded = padded_aabb(aabb, pad)?;
    let need = motion.max(corner_displacement(padded, start)?) * (1.0 + RELATIVE_MARGIN)
        + slack
        + ROUNDING_PER_METER * pad;
    (need <= pad).then_some(pad)
}

/// `||M - I||_inf` for the linear part `M` of the iteration-0 start map.
fn linear_deviation(previous: Transform, current: Transform, is_circle: bool) -> f32 {
    let previous = previous.rotation();
    let current = current.rotation();
    if is_circle {
        let previous_squared =
            previous.cosine() * previous.cosine() + previous.sine() * previous.sine();
        let current_squared = current.cosine() * current.cosine() + current.sine() * current.sine();
        return (previous_squared * current_squared - 1.0).abs();
    }
    let cosine = current.cosine() * previous.cosine() + current.sine() * previous.sine();
    let sine = current.sine() * previous.cosine() - current.cosine() * previous.sine();
    (cosine - 1.0).abs() + sine.abs()
}

/// Largest inf-norm coordinate magnitude the start computation touches.
fn coordinate_scale(aabb: Aabb, previous: Transform, current: Transform, center: Vec2) -> f32 {
    let inf_norm = |vector: Vec2| vector.x.abs().max(vector.y.abs());
    inf_norm(aabb.lower_bound()).max(inf_norm(aabb.upper_bound()))
        + inf_norm(previous.position())
        + inf_norm(current.position())
        + inf_norm(center)
}

/// Largest `|S(c) - c|_inf` over the 4 corners `c` of `aabb`.
fn corner_displacement(aabb: Aabb, start: impl Fn(Vec2) -> Option<Vec2>) -> Option<f32> {
    let lower = aabb.lower_bound();
    let upper = aabb.upper_bound();
    let corners = [
        lower,
        Vec2::new(upper.x, lower.y),
        upper,
        Vec2::new(lower.x, upper.y),
    ];
    let mut largest = 0.0_f32;
    for corner in corners {
        let moved = start(corner)?;
        let displacement = (moved.x - corner.x).abs().max((moved.y - corner.y).abs());
        if !displacement.is_finite() {
            return None;
        }
        largest = largest.max(displacement);
    }
    Some(largest)
}

fn padded_aabb(aabb: Aabb, pad: f32) -> Option<Aabb> {
    let pad = Vec2::new(pad, pad);
    Aabb::new(aabb.lower_bound() - pad, aabb.upper_bound() + pad).ok()
}
