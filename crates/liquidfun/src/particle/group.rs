//! Safe owned particle-group recipes and borrow-scoped group inspection.

#[allow(
    dead_code,
    reason = "consumed by the Phase 10 world-facing particle-group integration"
)]
pub(crate) mod sampling;

use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use crate::collision::Shape;
use crate::math::{Transform, Vec2};
use crate::{ParticleColor, ParticleFlags, ParticleGroupId, ParticleId};

const MAX_UPSTREAM_COUNT: usize = i32::MAX as usize;
const PRIVATE_GROUP_FLAG_MASK: u32 = 0x0018;

mod flags;
mod recipe;
mod source;
mod view;

pub use flags::*;
pub use recipe::*;
pub use source::*;
pub use view::*;

fn validate_count(count: usize) -> Result<(), ParticleGroupRecipeError> {
    if count > MAX_UPSTREAM_COUNT {
        return Err(ParticleGroupRecipeError::SourceCountOutOfRange { count });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
