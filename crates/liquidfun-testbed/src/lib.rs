//! Private renderer adapter and capability evidence for the `LiquidFun` testbed.

pub mod app;
mod capability;
pub mod controller_adapter;
pub mod input;
pub mod screenshot;
pub mod theme;
pub mod ui;

pub use capability::{
    CapabilityArtifact, CapabilityError, CapabilityOptions, CapabilityReport,
    REQUIRED_CAPABILITY_NAMES, run_capability_check,
};
