//! Differential-testing support for the native Rust engine and C++ oracle.
//!
//! This unpublished crate will orchestrate validated protocol scenarios and
//! compare semantic traces. It currently establishes only the runner boundary.

#![forbid(unsafe_code)]

mod canonical;
mod collision_evidence;
mod collision_probe;
mod comparator;
mod failure_bundle;
mod fixtures;
mod math_probe;
mod minimizer;
mod phase4_evidence;
mod report;
mod runner;
mod rust_adapter;
mod supervisor;

pub use canonical::*;
pub use collision_evidence::*;
pub use collision_probe::*;
pub use comparator::*;
pub use failure_bundle::*;
pub use fixtures::*;
pub use math_probe::*;
pub use minimizer::*;
pub use phase4_evidence::*;
pub use report::*;
pub use runner::*;
pub use rust_adapter::*;
pub use supervisor::*;
