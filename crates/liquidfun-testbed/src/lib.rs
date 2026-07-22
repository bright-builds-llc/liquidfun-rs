//! Private renderer adapter and capability evidence for the `LiquidFun` testbed.

mod capability;

pub use capability::{
    CapabilityArtifact, CapabilityError, CapabilityOptions, CapabilityReport,
    REQUIRED_CAPABILITY_NAMES, run_capability_check,
};
