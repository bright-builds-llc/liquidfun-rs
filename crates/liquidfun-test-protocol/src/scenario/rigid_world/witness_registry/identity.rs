use super::RigidWorldWitness;

impl RigidWorldWitness {
    #[must_use]
    pub const fn requires_contact_identity(self) -> bool {
        matches!(
            self,
            Self::ContactCreated
                | Self::ContactBegin
                | Self::ContactPersisted
                | Self::ManifoldActive
                | Self::ContactSolved
                | Self::WarmStartTransferred
                | Self::SensorTouching
                | Self::SensorWithoutManifold
                | Self::FilterRemovedContact
                | Self::FilterRecreatedContact
                | Self::DeactivationDestroyedContact
                | Self::ReactivationRecreatedContact
                | Self::FixtureDestroyedContact
                | Self::BodyCascadeEndOrdered
                | Self::MultiContactIslandSolved
                | Self::IslandTraversalOrdered
                | Self::WarmStartApplied
                | Self::WarmStartDisabledThenStored
                | Self::ContactWokeIsland
                | Self::ContinuousPhysicsPreventedTunneling
                | Self::BulletStateSelectedContinuousContact
                | Self::ContinuousTransitionsOrdered
        )
    }
}
