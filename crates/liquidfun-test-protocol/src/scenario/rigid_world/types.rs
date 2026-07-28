use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::witness_registry::{RigidWorldWitness, RigidWorldWitnessFamily};
use super::{
    Phase9ParticleAction, Phase9ParticleDeclaration, Phase9ParticleSystemDeclaration,
    Phase10Operation,
};
use crate::{
    CodecError, FloatBits, ProtocolVersion, RequestId, ScenarioId, ScenarioSchemaVersion,
    ScenarioSource, Sha256Hex, ToleranceProfileVersion, TraceSchemaVersion, TransformBits,
    Vec2Bits,
};

mod action;
mod body;
mod directives;
mod joint;
mod timeline;

pub use action::*;
pub use body::*;
pub use directives::*;
pub use joint::*;
pub use timeline::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidWorldErrorKind {
    NoTimelines,
    DuplicateWitnessFamily,
    MissingWitnessFamily,
    DuplicateBodyId,
    DuplicateFixtureId,
    DuplicateActionId,
    DuplicateCheckpointId,
    DuplicateWitness,
    InvalidIdentifier,
    InvalidSource,
    InvalidGeometry,
    InvalidMaterial,
    InvalidOwner,
    UnknownBody,
    UnknownFixture,
    InvalidActionOrder,
    InvalidCheckpointOrder,
    CheckpointPhaseMismatch,
    ExpectedCountMismatch,
    MissingWitness,
    UnexpectedWitness,
    InvalidContactIdentity,
    InvalidBodyControl,
    InvalidStepConfiguration,
    InvalidQueryDirective,
    InvalidRayDirective,
    AggregateLimitExceeded,
    ResultTimelineMismatch,
    ResultCheckpointMismatch,
    ResultDeclarationOrderMismatch,
    ResultObservationMismatch,
    DuplicateJointId,
    DuplicateRopeId,
    UnknownJoint,
    UnknownRope,
    InvalidJointDefinition,
    InvalidJointDependency,
    InvalidRopeDefinition,
    InvalidContactDirective,
    InvalidParticleDefinition,
    InvalidParticleAction,
    InvalidParticleGroupDefinition,
    InvalidParticleGroupAction,
    InvalidParticleGroupResult,
}

pub(super) const fn validation(kind: RigidWorldErrorKind) -> RigidWorldDecodeError {
    RigidWorldDecodeError::Validation(kind)
}

pub(super) fn apply_lifecycle_action(
    action: &RigidWorldAction,
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
    live_bodies: &mut HashSet<ScenarioId>,
    live_fixtures: &mut HashSet<ScenarioId>,
) {
    match action {
        RigidWorldAction::CreateBody { body_id } => {
            live_bodies.insert(body_id.clone());
        }
        RigidWorldAction::CreateFixture { fixture_id } => {
            live_fixtures.insert(fixture_id.clone());
        }
        RigidWorldAction::DestroyFixture { fixture_id } => {
            live_fixtures.remove(fixture_id);
        }
        RigidWorldAction::DestroyBody { body_id } => {
            live_bodies.remove(body_id);
            live_fixtures.retain(|fixture_id| fixture_owners.get(fixture_id) != Some(body_id));
        }
        _ => {}
    }
}

fn joints_are_empty(joints: &[RigidJointDeclaration]) -> bool {
    joints.is_empty()
}

fn ropes_are_empty(ropes: &[RigidRopeDeclaration]) -> bool {
    ropes.is_empty()
}

fn particle_systems_are_empty(systems: &[Phase9ParticleSystemDeclaration]) -> bool {
    systems.is_empty()
}

fn particles_are_empty(particles: &[Phase9ParticleDeclaration]) -> bool {
    particles.is_empty()
}
