//! Private WebAssembly bridge for the `LiquidFun` browser playground.

#![forbid(unsafe_code)]

mod frame;
mod scene;
mod session;

pub use frame::ProofFrame;
