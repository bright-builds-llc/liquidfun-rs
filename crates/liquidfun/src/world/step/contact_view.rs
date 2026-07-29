use super::{
    BodyId, ContactPointSnapshot, Error, FixtureId, ManagedContactSnapshot, ParticleBodyContact,
    ParticleContact, ParticleId, fmt,
};

/// A read-only view valid only for the duration of one hook call.
///
/// A hook cannot retain the view beyond its callback lifetime:
///
/// ```compile_fail
/// use liquidfun::{ContactView, StepHook};
///
/// struct RetainingHook {
///     retained: Option<ContactView<'static>>,
/// }
///
/// impl StepHook for RetainingHook {
///     fn observe(&mut self, contact: ContactView<'_>) {
///         self.retained = Some(contact);
///     }
/// }
/// ```
#[derive(Clone, Copy)]
pub struct ContactView<'step> {
    pub(super) contact: &'step ManagedContactSnapshot,
}

impl<'step> ContactView<'step> {
    /// Returns owned typed fixture identities without exposing contact storage.
    #[must_use]
    pub const fn fixtures(self) -> [FixtureId; 2] {
        self.contact.fixtures()
    }

    /// Returns typed body identities in oriented manager order.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        self.contact.bodies()
    }

    /// Returns shape-child coordinates in oriented manager order.
    #[must_use]
    pub const fn child_indices(self) -> [crate::collision::ChildIndex; 2] {
        self.contact.child_indices()
    }

    /// Returns whether the occurrence is currently touching.
    #[must_use]
    pub const fn is_touching(self) -> bool {
        self.contact.is_touching()
    }

    /// Returns whether the occurrence bypasses pre-solve and constraints.
    #[must_use]
    pub const fn is_sensor(self) -> bool {
        self.contact.is_sensor()
    }

    /// Returns the canonical manifold when this is a solid touching occurrence.
    #[must_use]
    pub const fn maybe_manifold(self) -> Option<&'step crate::collision::Manifold> {
        self.contact.maybe_manifold()
    }

    /// Returns warm-start points in canonical manifold order.
    #[must_use]
    pub fn points(self) -> &'step [ContactPointSnapshot] {
        self.contact.points()
    }

    /// Returns the creation-time mixed friction coefficient.
    #[must_use]
    pub const fn friction(self) -> f32 {
        self.contact.friction()
    }

    /// Returns the creation-time mixed restitution coefficient.
    #[must_use]
    pub const fn restitution(self) -> f32 {
        self.contact.restitution()
    }

    /// Returns the configured surface tangent speed.
    #[must_use]
    pub const fn tangent_speed(self) -> f32 {
        self.contact.tangent_speed()
    }
}

impl fmt::Debug for ContactView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ContactView")
            .field("fixtures", &self.fixtures())
            .field("child_indices", &self.child_indices())
            .field("touching", &self.is_touching())
            .field("sensor", &self.is_sensor())
            .finish_non_exhaustive()
    }
}

/// Narrow collision-filter result returned by a hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionDirective {
    /// Continue processing the occurrence.
    Collide,
    /// Ignore this occurrence before pre-solve processing.
    Ignore,
}

/// Borrow-scoped fixture-particle contact view for one synchronous decision.
#[derive(Clone, Copy)]
pub struct FixtureParticleView<'step> {
    pub(super) contact: &'step ParticleBodyContact,
}

impl FixtureParticleView<'_> {
    /// Returns the stable particle identity.
    #[must_use]
    pub const fn particle(self) -> ParticleId {
        self.contact.particle()
    }

    /// Returns the stable fixture identity.
    #[must_use]
    pub const fn fixture(self) -> FixtureId {
        self.contact.fixture()
    }

    /// Returns the stable body identity.
    #[must_use]
    pub const fn body(self) -> BodyId {
        self.contact.body()
    }

    /// Returns the proposed contact weight.
    #[must_use]
    pub const fn weight(self) -> f32 {
        self.contact.weight()
    }

    /// Returns the proposed contact normal.
    #[must_use]
    pub const fn normal(self) -> crate::math::Vec2 {
        self.contact.normal()
    }
}

/// Borrow-scoped particle-pair contact view for one synchronous decision.
#[derive(Clone, Copy)]
pub struct ParticlePairContactView<'step> {
    pub(super) contact: &'step ParticleContact,
}

impl ParticlePairContactView<'_> {
    /// Returns both stable particle identities in source contact order.
    #[must_use]
    pub const fn particles(self) -> [ParticleId; 2] {
        self.contact.particles()
    }

    /// Returns the proposed pair-contact weight.
    #[must_use]
    pub const fn weight(self) -> f32 {
        self.contact.weight()
    }

    /// Returns the proposed pair-contact normal.
    #[must_use]
    pub const fn normal(self) -> crate::math::Vec2 {
        self.contact.normal()
    }
}

#[derive(Debug, Clone, Copy)]
pub(in crate::world) struct FixturePairSnapshot {
    fixtures: [FixtureId; 2],
    bodies: [BodyId; 2],
    child_indices: [crate::collision::ChildIndex; 2],
}

/// Owned evidence for one source-timed collision-filter decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionFilterEvent {
    fixtures: [FixtureId; 2],
    bodies: [BodyId; 2],
    child_indices: [crate::collision::ChildIndex; 2],
    decision: CollisionDirective,
}

impl CollisionFilterEvent {
    pub(super) const fn new(pair: FixturePairSnapshot, decision: CollisionDirective) -> Self {
        Self {
            fixtures: pair.fixtures,
            bodies: pair.bodies,
            child_indices: pair.child_indices,
            decision,
        }
    }

    /// Returns fixture identities in canonical pair order.
    #[must_use]
    pub const fn fixtures(self) -> [FixtureId; 2] {
        self.fixtures
    }

    /// Returns body identities in canonical pair order.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        self.bodies
    }

    /// Returns shape-child coordinates in canonical pair order.
    #[must_use]
    pub const fn child_indices(self) -> [crate::collision::ChildIndex; 2] {
        self.child_indices
    }

    /// Returns the exact decision made at the admission or refilter point.
    #[must_use]
    pub const fn decision(self) -> CollisionDirective {
        self.decision
    }
}

impl FixturePairSnapshot {
    pub(in crate::world) const fn new(
        fixtures: [FixtureId; 2],
        bodies: [BodyId; 2],
        child_indices: [crate::collision::ChildIndex; 2],
    ) -> Self {
        Self {
            fixtures,
            bodies,
            child_indices,
        }
    }
}

/// Borrow-scoped semantic fixture pair evaluated before contact admission.
///
/// The view deliberately contains no reusable contact identity because a
/// rejected admission does not create a contact.
///
/// ```compile_fail
/// use liquidfun::ContactId;
/// ```
#[derive(Clone, Copy)]
pub struct FixturePairView<'hook> {
    pair: &'hook FixturePairSnapshot,
}

impl<'hook> FixturePairView<'hook> {
    pub(in crate::world) const fn new(pair: &'hook FixturePairSnapshot) -> Self {
        Self { pair }
    }

    /// Returns fixture identities in canonical pair order.
    #[must_use]
    pub const fn fixtures(self) -> [FixtureId; 2] {
        self.pair.fixtures
    }

    /// Returns body identities in canonical pair order.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        self.pair.bodies
    }

    /// Returns shape-child coordinates in canonical pair order.
    #[must_use]
    pub const fn child_indices(self) -> [crate::collision::ChildIndex; 2] {
        self.pair.child_indices
    }
}

impl fmt::Debug for FixturePairView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FixturePairView")
            .field("fixtures", &self.fixtures())
            .field("bodies", &self.bodies())
            .field("child_indices", &self.child_indices())
            .finish()
    }
}

/// A validated contact-control value was rejected before hook application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContactControlError {
    /// The value was NaN or infinite.
    NonFinite,
    /// Friction or restitution was negative.
    Negative,
}

impl fmt::Display for ContactControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite => formatter.write_str("contact control must be finite"),
            Self::Negative => formatter.write_str("contact material control must be non-negative"),
        }
    }
}

impl Error for ContactControlError {}

/// Opaque validated material controls carried by [`PreSolveDirective`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(
    clippy::struct_field_names,
    reason = "maybe_ prefixes make the three optional source controls explicit"
)]
pub struct PreSolveControls {
    maybe_friction: Option<f32>,
    maybe_restitution: Option<f32>,
    maybe_tangent_speed: Option<f32>,
}

impl PreSolveControls {
    const EMPTY: Self = Self {
        maybe_friction: None,
        maybe_restitution: None,
        maybe_tangent_speed: None,
    };
}

/// Narrow pre-solve result returned by a hook.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[non_exhaustive]
pub enum PreSolveDirective {
    /// Keep the occurrence enabled.
    #[default]
    Enable,
    /// Disable the occurrence for this step.
    Disable,
    /// Keep the occurrence enabled and apply validated source-supported controls.
    Configure {
        /// Whether this occurrence remains enabled for the current update.
        enabled: bool,
        /// Validated source-supported material controls.
        controls: PreSolveControls,
    },
}

impl PreSolveDirective {
    /// Returns this directive with a finite non-negative friction override.
    ///
    /// # Errors
    ///
    /// Rejects non-finite or negative values.
    pub fn with_friction(self, friction: f32) -> Result<Self, ContactControlError> {
        validate_material_control(friction)?;
        let (enabled, mut controls) = self.parts();
        controls.maybe_friction = Some(friction);
        Ok(Self::Configure { enabled, controls })
    }

    /// Returns this directive with a finite non-negative restitution override.
    ///
    /// # Errors
    ///
    /// Rejects non-finite or negative values.
    pub fn with_restitution(self, restitution: f32) -> Result<Self, ContactControlError> {
        validate_material_control(restitution)?;
        let (enabled, mut controls) = self.parts();
        controls.maybe_restitution = Some(restitution);
        Ok(Self::Configure { enabled, controls })
    }

    /// Returns this directive with a finite tangent-speed override.
    ///
    /// # Errors
    ///
    /// Rejects NaN and infinity.
    pub fn with_tangent_speed(self, tangent_speed: f32) -> Result<Self, ContactControlError> {
        if !tangent_speed.is_finite() {
            return Err(ContactControlError::NonFinite);
        }
        let (enabled, mut controls) = self.parts();
        controls.maybe_tangent_speed = Some(tangent_speed);
        Ok(Self::Configure { enabled, controls })
    }

    const fn parts(self) -> (bool, PreSolveControls) {
        match self {
            Self::Enable => (true, PreSolveControls::EMPTY),
            Self::Disable => (false, PreSolveControls::EMPTY),
            Self::Configure { enabled, controls } => (enabled, controls),
        }
    }

    pub(in crate::world) const fn enabled(self) -> bool {
        self.parts().0
    }

    pub(in crate::world) const fn material_controls(
        self,
    ) -> (Option<f32>, Option<f32>, Option<f32>) {
        let controls = self.parts().1;
        (
            controls.maybe_friction,
            controls.maybe_restitution,
            controls.maybe_tangent_speed,
        )
    }
}

fn validate_material_control(value: f32) -> Result<(), ContactControlError> {
    if !value.is_finite() {
        return Err(ContactControlError::NonFinite);
    }
    if value < 0.0 {
        return Err(ContactControlError::Negative);
    }
    Ok(())
}

/// Borrow-scoped semantic state available at the pinned pre-solve point.
#[derive(Clone, Copy)]
pub struct PreSolveView<'hook> {
    current: &'hook ManagedContactSnapshot,
    current_manifold: &'hook crate::collision::Manifold,
    maybe_previous_manifold: Option<&'hook crate::collision::Manifold>,
}

impl<'hook> PreSolveView<'hook> {
    pub(in crate::world) const fn new(
        current: &'hook ManagedContactSnapshot,
        maybe_previous_manifold: Option<&'hook crate::collision::Manifold>,
    ) -> Self {
        Self {
            current,
            current_manifold: current
                .maybe_manifold()
                .expect("pre-solve construction requires a touching solid manifold"),
            maybe_previous_manifold,
        }
    }

    /// Returns fixture identities in oriented manager order.
    #[must_use]
    pub const fn fixtures(self) -> [FixtureId; 2] {
        self.current.fixtures()
    }

    /// Returns body identities in oriented manager order.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        self.current.bodies()
    }

    /// Returns child indices in oriented manager order.
    #[must_use]
    pub const fn child_indices(self) -> [crate::collision::ChildIndex; 2] {
        self.current.child_indices()
    }

    /// Returns the current touching manifold.
    #[must_use]
    pub fn current_manifold(self) -> &'hook crate::collision::Manifold {
        self.current_manifold
    }

    /// Returns the owned semantic manifold captured before this update.
    #[must_use]
    pub const fn maybe_previous_manifold(self) -> Option<&'hook crate::collision::Manifold> {
        self.maybe_previous_manifold
    }

    /// Returns the current semantic contact through the legacy read-only view.
    #[must_use]
    pub const fn contact(self) -> ContactView<'hook> {
        ContactView {
            contact: self.current,
        }
    }
}

impl fmt::Debug for PreSolveView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreSolveView")
            .field("fixtures", &self.fixtures())
            .field("bodies", &self.bodies())
            .field("child_indices", &self.child_indices())
            .finish_non_exhaustive()
    }
}
