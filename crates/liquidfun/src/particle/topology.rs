use std::error::Error;
use std::fmt;

#[allow(
    dead_code,
    reason = "consumed by the Phase 10 group join and reactive topology integration"
)]
pub(in crate::particle) mod constraints;
mod voronoi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::particle) struct VoronoiLimits {
    generators: usize,
    cells: usize,
    queue_tasks: usize,
    work: usize,
    nodes: usize,
}

impl VoronoiLimits {
    const fn new(
        maximum_generators: usize,
        maximum_cells: usize,
        maximum_queue_tasks: usize,
        maximum_work: usize,
        maximum_nodes: usize,
    ) -> Self {
        Self {
            generators: maximum_generators,
            cells: maximum_cells,
            queue_tasks: maximum_queue_tasks,
            work: maximum_work,
            nodes: maximum_nodes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::particle) enum VoronoiError {
    NonFiniteRadius,
    NonPositiveRadius,
    NonFiniteMargin,
    NegativeMargin,
    NonFiniteGenerator { ordinal: usize },
    GeneratorLimitExceeded { required: usize, limit: usize },
    AxisCountOutOfRange,
    ArithmeticOverflow,
    GridLimitExceeded { required: usize, limit: usize },
    QueueLimitExceeded { required: usize, limit: usize },
    WorkLimitExceeded { required: usize, limit: usize },
    NodeLimitExceeded { required: usize, limit: usize },
    NonFiniteDerivedGeometry,
    IncompleteDiagram,
    AllocationFailed,
}

impl fmt::Display for VoronoiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "bounded Voronoi generation failed: {self:?}")
    }
}

impl Error for VoronoiError {}
