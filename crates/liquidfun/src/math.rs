//! Purpose-built scalar and vector math for `LiquidFun` compatibility.
//!
//! This module maps the selected upstream `b2Math.h` concepts into initialized,
//! safe Rust values. Arithmetic preserves the pinned source's operand order and
//! expression grouping where floating-point behavior is observable.
//!
//! Physics quantities use meters-kilograms-seconds (MKS). Angles are radians,
//! angular velocities are radians per second, and callers should perform any
//! rendering or pixel conversion outside the physics layer.
//!
//! Raw math values deliberately preserve every IEEE-754 `f32` bit pattern,
//! including signed zero, subnormals, infinities, and NaNs. Use [`is_valid`] or
//! the vector `is_valid` methods when a finite physics-domain value is required.

mod scalar;
/// Immutable constants translated from the selected upstream `b2Settings.h`.
pub mod settings;
mod vector;

pub use scalar::{
    abs, clamp, distance, distance_squared, inverse_sqrt, is_power_of_two, is_valid, max, min,
    next_power_of_two,
};
pub use vector::{Vec2, Vec3, Vec4};
