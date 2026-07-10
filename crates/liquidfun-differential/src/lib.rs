//! Differential-testing support for the native Rust engine and C++ oracle.
//!
//! This unpublished crate will orchestrate validated protocol scenarios and
//! compare semantic traces. It currently establishes only the runner boundary.

#![forbid(unsafe_code)]

mod canonical;
mod comparator;
mod failure_bundle;
mod fixtures;
mod minimizer;
mod report;
mod runner;
mod rust_adapter;
mod supervisor;

pub use canonical::*;
pub use comparator::*;
pub use failure_bundle::*;
pub use fixtures::*;
pub use minimizer::*;
pub use report::*;
pub use runner::*;
pub use rust_adapter::*;
pub use supervisor::*;
