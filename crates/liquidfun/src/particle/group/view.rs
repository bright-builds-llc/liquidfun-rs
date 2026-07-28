use super::{ParticleGroupFlags, ParticleGroupId, ParticleId, Transform, Vec2};

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
