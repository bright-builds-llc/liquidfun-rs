//! Scripted magnetic pull: the gradient of one bounded polynomial potential
//! lifts liquid along the field axis (opposite gravity) into spike columns
//! centered on the magnet. It is not a magnetic field solve.
//!
//! For a particle at `p` and the magnet at `m`, with `r = p - m`,
//! `h = r . up` (height above the magnet) and `s = r . across` (offset across
//! the dish), the potential is `Psi = G * S * L * E(h) * R(s) * M(s)`:
//!
//! - `E` is the integral of the reach bump `E'(h) = (1 - (h/H)^2)^2`, so the
//!   lift has a softened core and falls smoothly to zero at the reach `H`.
//! - `R` is the envelope bump over the half width `W`: the pull fades toward
//!   the walls, and its slope draws liquid toward an off-center magnet.
//! - `M = (1 - 4t^2)^4` with `t` the offset from the nearest column center is
//!   a periodic bump that concentrates the lift into separate columns.
//!
//! The lift is `dPsi/dh` along `up`. The sideways pull is `dPsi/ds` with
//! `|E(h)|` in place of `E(h)`, so liquid on both sides of the magnet axis is
//! drawn toward it: with the odd `E`, liquid below a magnet held inside or
//! above the puddle was pushed away from the columns and no crown formed.
//! `|E|` levels off past the reach, so the sideways pull also fades from the
//! reach to twice the reach: a magnet far from the liquid gathers nothing
//! (D-02). The field is therefore conservative above the magnet and inside
//! the reach only. The acceleration is
//! capped at `CAP * G * S`, and drag damps it everywhere inside the envelope;
//! 60 s holds at full strength in five magnet positions show no energy growth,
//! so the spikes settle and hold (D-02). Only `+ - * /`, `abs`, `round` and
//! `signum` are used, so native and wasm builds compute identical bits.

use liquidfun::math::Vec2;

use crate::scene::particle_force::GravityFrame;

/// Shape and strength of the scripted pull. Every pulling term carries the
/// gravity magnitude `G`, so the spikes keep their shape on the gravity slider
/// and follow phone tilt (D-14).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct FieldParams {
    /// Spacing of the spike columns across the dish, in meters.
    pub(super) wavelength: f32,
    /// Half width `W` of the envelope that fades the pull toward the walls.
    pub(super) envelope_half_width: f32,
    /// Vertical reach `H` of the lift above and below the magnet.
    pub(super) reach: f32,
    /// Potential scale `L`: the lift is `G * S * L` at the magnet before the cap.
    pub(super) strength_scale: f32,
    /// Acceleration cap as a multiple of `G * S` (D-02: at most about 2 to 3).
    pub(super) cap: f32,
    /// Velocity damping rate, per second, inside the envelope while the
    /// magnet is on. Drag is damping, not a pull, so it is deliberately
    /// independent of the gravity magnitude: a per-second rate keeps the
    /// settling time the same across the gravity slider while every pulling
    /// term scales with `G` (D-14).
    pub(super) drag_per_second: f32,
}

/// The tuned pull for the shipped scene. At the default strength the cap is
/// above gravity, so the lift beats gravity only inside a contour that
/// narrows with height, which draws each column into a pointed spike. The
/// 48-01 cap of 1.5 kept the lift below gravity at the default, so the
/// columns were low rounded crests about 0.28 m tall; these values raise
/// them to about 0.6 m (quick 261016-u0s). The drag of 5 per second keeps
/// the fastest particle near 3 m/s in full-strength holds.
pub(super) const FIELD: FieldParams = FieldParams {
    wavelength: 0.65,
    envelope_half_width: 2.6,
    reach: 2.0,
    strength_scale: 3.5,
    cap: 2.5,
    drag_per_second: 5.0,
};

const _: () = assert!(FIELD.cap <= 3.0);

/// Bounded C1 bump: `(1 - x^2)^2` inside `|x| < 1`, else 0.
fn bump(x: f32) -> f32 {
    if x.abs() >= 1.0 {
        return 0.0;
    }
    let q = 1.0 - x * x;
    q * q
}

/// Integral of `bump(h / reach)` over `h`: odd and bounded by `8 * reach / 15`.
fn bump_integral(h: f32, reach: f32) -> f32 {
    let x = h / reach;
    if x.abs() >= 1.0 {
        return x.signum() * reach * 8.0 / 15.0;
    }
    let x3 = x * x * x;
    reach * (x - 2.0 * x3 / 3.0 + x3 * x * x / 5.0)
}

/// Fade of the sideways pull: 1 inside the reach, falling smoothly to 0 at
/// twice the reach.
fn far_fade(h: f32, reach: f32) -> f32 {
    let x = (h.abs() - reach) / reach;
    if x <= 0.0 {
        return 1.0;
    }
    bump(x)
}

/// Column modulation `b^4` with `b = 1 - 4t^2` and `t = s/wavelength -
/// round(s/wavelength)`; returns `(M, dM/ds)`.
fn columns(s: f32, wavelength: f32) -> (f32, f32) {
    let u = s / wavelength;
    let t = u - u.round();
    let b = 1.0 - 4.0 * t * t;
    let b2 = b * b;
    (b2 * b2, 4.0 * b2 * b * (-8.0 * t) / wavelength)
}

/// Envelope `R(s)` and its slope `R'(s)` over the half width.
fn envelope_and_slope(params: &FieldParams, s: f32) -> (f32, f32) {
    let half_width = params.envelope_half_width;
    let y = s / half_width;
    if y.abs() >= 1.0 {
        return (0.0, 0.0);
    }
    let q = 1.0 - y * y;
    (q * q, 2.0 * q * (-2.0 * s / (half_width * half_width)))
}

/// Offset of `position` from `magnet` in the gravity frame: `(h, s)`.
fn frame_offset(frame: GravityFrame, magnet: Vec2, position: Vec2) -> (f32, f32) {
    let offset = position - magnet;
    let h = offset.x * frame.up.x + offset.y * frame.up.y;
    let s = offset.x * frame.across.x + offset.y * frame.across.y;
    (h, s)
}

/// Envelope weight `R(s)` in `0..=1` at `position`; drag scales with it.
pub(super) fn envelope(
    params: &FieldParams,
    frame: GravityFrame,
    magnet: Vec2,
    position: Vec2,
) -> f32 {
    let (_h, s) = frame_offset(frame, magnet, position);
    envelope_and_slope(params, s).0
}

/// Scripted acceleration (without drag) at `position` for a magnet at
/// `magnet` and a strength in `0..=1`.
pub(super) fn magnet_acceleration(
    params: &FieldParams,
    frame: GravityFrame,
    strength: f32,
    magnet: Vec2,
    position: Vec2,
) -> Vec2 {
    let (h, s) = frame_offset(frame, magnet, position);
    let (envelope, envelope_slope) = envelope_and_slope(params, s);
    let (modulation, modulation_slope) = columns(s, params.wavelength);
    let weight = envelope * modulation;
    let weight_slope = envelope_slope * modulation + envelope * modulation_slope;
    let scale = frame.magnitude * strength * params.strength_scale;
    let lift = scale * bump(h / params.reach) * weight;
    // `|E(h)|`: below the magnet the sideways pull still points toward it.
    // The fade is exactly 1 inside the reach, so the pull there is unchanged.
    let lateral =
        scale * bump_integral(h, params.reach).abs() * far_fade(h, params.reach) * weight_slope;
    let acceleration = frame.up * lift + frame.across * lateral;
    let cap = params.cap * frame.magnitude * strength;
    let length = acceleration.length();
    if length <= cap {
        return acceleration;
    }
    acceleration * (cap / length)
}
