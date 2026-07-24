//! Bounded native performance execution over sealed resolved scenarios.

mod native;
mod oracle;
mod report;

pub use native::*;
pub use oracle::{NativeBenchmarkAdapter, OracleBenchmarkAdapter, PairedBenchmarkAdapter};
pub use report::{
    BenchmarkAdapterOutput, PairedBenchmarkOutcome, PairedBenchmarkPlan, PairedEngineOrder,
    PairedHarnessFailure, PairedPerformanceError, PairedPerformanceErrorKind,
    PairedPerformanceReport, PairedPhysicsMismatch, PairedRawSample, RustChildProfileDiagnostic,
    run_paired_benchmark,
};
