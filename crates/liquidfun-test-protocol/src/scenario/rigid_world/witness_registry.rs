use serde::{Deserialize, Serialize};

use super::types::RigidWorldActionKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidWorldWitnessFamily {
    NonCollidingBodyFixtureLifecycle,
    SingleContactLifecycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidWorldWitness {
    StaticBodyCreated,
    KinematicBodyCreated,
    DynamicBodyCreated,
    FixturesCreated,
    BodyInspected,
    FixtureInspected,
    BodyTransformChanged,
    BodyTypeChanged,
    BodyDeactivated,
    BodyReactivated,
    SensorEnabled,
    SensorDisabled,
    MaterialChanged,
    FilterChanged,
    DensityChangedWithoutMassReset,
    MassReset,
    CustomMassSet,
    StaticKinematicOverlapRejected,
    KinematicKinematicOverlapRejected,
    ZeroContactStep,
    FixtureDestroyed,
    BodyDestroyed,
    ContactCreated,
    ContactBegin,
    ContactPersisted,
    ManifoldActive,
    ContactSolved,
    WarmStartTransferred,
    SensorTouching,
    SensorWithoutManifold,
    FilterRemovedContact,
    FilterRecreatedContact,
    DeactivationDestroyedContact,
    ReactivationRecreatedContact,
    FixtureDestroyedContact,
    BodyCascadeEndOrdered,
}

const NON_COLLIDING_WITNESSES: [RigidWorldWitness; 22] = [
    RigidWorldWitness::StaticBodyCreated,
    RigidWorldWitness::KinematicBodyCreated,
    RigidWorldWitness::DynamicBodyCreated,
    RigidWorldWitness::FixturesCreated,
    RigidWorldWitness::BodyInspected,
    RigidWorldWitness::FixtureInspected,
    RigidWorldWitness::BodyTransformChanged,
    RigidWorldWitness::BodyTypeChanged,
    RigidWorldWitness::BodyDeactivated,
    RigidWorldWitness::BodyReactivated,
    RigidWorldWitness::SensorEnabled,
    RigidWorldWitness::SensorDisabled,
    RigidWorldWitness::MaterialChanged,
    RigidWorldWitness::FilterChanged,
    RigidWorldWitness::DensityChangedWithoutMassReset,
    RigidWorldWitness::MassReset,
    RigidWorldWitness::CustomMassSet,
    RigidWorldWitness::StaticKinematicOverlapRejected,
    RigidWorldWitness::KinematicKinematicOverlapRejected,
    RigidWorldWitness::ZeroContactStep,
    RigidWorldWitness::FixtureDestroyed,
    RigidWorldWitness::BodyDestroyed,
];

const SINGLE_CONTACT_WITNESSES: [RigidWorldWitness; 14] = [
    RigidWorldWitness::ContactCreated,
    RigidWorldWitness::ContactBegin,
    RigidWorldWitness::ContactPersisted,
    RigidWorldWitness::ManifoldActive,
    RigidWorldWitness::ContactSolved,
    RigidWorldWitness::WarmStartTransferred,
    RigidWorldWitness::SensorTouching,
    RigidWorldWitness::SensorWithoutManifold,
    RigidWorldWitness::FilterRemovedContact,
    RigidWorldWitness::FilterRecreatedContact,
    RigidWorldWitness::DeactivationDestroyedContact,
    RigidWorldWitness::ReactivationRecreatedContact,
    RigidWorldWitness::FixtureDestroyedContact,
    RigidWorldWitness::BodyCascadeEndOrdered,
];

const NON_COLLIDING_ACTIONS: [RigidWorldActionKind; 16] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::InspectBody,
    RigidWorldActionKind::InspectFixture,
    RigidWorldActionKind::SetBodyTransform,
    RigidWorldActionKind::SetBodyType,
    RigidWorldActionKind::SetBodyActive,
    RigidWorldActionKind::SetFixtureSensor,
    RigidWorldActionKind::SetFixtureMaterial,
    RigidWorldActionKind::SetFixtureFilter,
    RigidWorldActionKind::SetFixtureDensity,
    RigidWorldActionKind::ResetMassData,
    RigidWorldActionKind::SetCustomMassData,
    RigidWorldActionKind::Step,
    RigidWorldActionKind::DestroyFixture,
    RigidWorldActionKind::DestroyBody,
];

const SINGLE_CONTACT_ACTIONS: [RigidWorldActionKind; 8] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::SetBodyActive,
    RigidWorldActionKind::SetFixtureSensor,
    RigidWorldActionKind::SetFixtureFilter,
    RigidWorldActionKind::Step,
    RigidWorldActionKind::DestroyFixture,
    RigidWorldActionKind::DestroyBody,
];

impl RigidWorldWitnessFamily {
    pub const REQUIRED: [Self; 2] = [
        Self::NonCollidingBodyFixtureLifecycle,
        Self::SingleContactLifecycle,
    ];

    #[must_use]
    pub const fn required_witnesses(self) -> &'static [RigidWorldWitness] {
        match self {
            Self::NonCollidingBodyFixtureLifecycle => &NON_COLLIDING_WITNESSES,
            Self::SingleContactLifecycle => &SINGLE_CONTACT_WITNESSES,
        }
    }

    pub(super) const fn required_action_kinds(self) -> &'static [RigidWorldActionKind] {
        match self {
            Self::NonCollidingBodyFixtureLifecycle => &NON_COLLIDING_ACTIONS,
            Self::SingleContactLifecycle => &SINGLE_CONTACT_ACTIONS,
        }
    }
}

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
        )
    }
}
