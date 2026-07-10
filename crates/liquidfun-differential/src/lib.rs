//! Differential-testing support for the native Rust engine and C++ oracle.
//!
//! This unpublished crate will orchestrate validated protocol scenarios and
//! compare semantic traces. It currently establishes only the runner boundary.

#![forbid(unsafe_code)]

mod canonical;
mod comparator;
mod report;

pub use canonical::*;
pub use comparator::*;
pub use report::*;
