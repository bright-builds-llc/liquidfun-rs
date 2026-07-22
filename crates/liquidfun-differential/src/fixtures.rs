//! Confined staging, replay, review, and atomic promotion for differential evidence.
//!
//! `storage` owns `canonicalize`, `symlink_metadata`, `create_new`, and manifest `rename`
//! controls. `domain` owns `review_status` metadata, while `replay` binds any minimized
//! regression to its exact [`crate::FailureSignature`].

mod domain;
mod lifecycle;
mod replay;
#[path = "rigid_fixtures.rs"]
mod rigid;
mod storage;

pub use domain::{
    ArtifactCandidate, ArtifactKind, FixtureError, PromotionReceipt, ReviewMetadata, ReviewReceipt,
    StageRequest,
};
pub use lifecycle::{promote_candidate, review_candidate, stage_candidate};
pub use replay::{
    CatalogRegressionError, CatalogRegressionErrorKind, CatalogRegressionReplay,
    CatalogRegressionReplayEntry, replay_catalog_failure_bundle, replay_catalog_regressions,
};
pub use rigid::{RIGID_FIXTURE_SCENARIO_ID, stage_rigid_candidate};
