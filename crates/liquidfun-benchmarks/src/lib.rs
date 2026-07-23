//! Private complete-matrix benchmark support.
//!
//! Criterion remains a Rust-only diagnostic consumer. Paired samples use the same sealed case
//! preparation and explicit measured-region executor without treating Criterion's adaptive loop
//! as cross-engine evidence.

#![forbid(unsafe_code)]

mod paired;

pub use paired::*;
