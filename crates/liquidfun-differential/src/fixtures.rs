//! Confined staging, replay, review, and atomic promotion for differential evidence.
//!
//! `storage` owns `canonicalize`, `symlink_metadata`, `create_new`, and manifest `rename`
//! controls. `domain` owns `review_status` metadata, while `replay` binds any minimized
//! regression to its exact [`crate::FailureSignature`].

mod domain;
mod lifecycle;
mod replay;
mod storage;

pub use domain::{
    ArtifactCandidate, ArtifactKind, FixtureError, PromotionReceipt, ReviewMetadata, ReviewReceipt,
    StageRequest,
};
pub use lifecycle::{promote_candidate, review_candidate, stage_candidate};
